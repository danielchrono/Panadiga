"""Offline CI unit tests for the non-destructive P7501 bench ecosystem."""

from __future__ import annotations

from pathlib import Path

import pytest

from scripts.bench.decoders import (
    parse_am34_uart_log,
    parse_busdiag_output,
    parse_pincheck_output,
    parse_status_output,
    read_captured_log,
)
from scripts.bench.protocol import (
    BenchSafetyError,
    BenchSession,
    MockBenchTransport,
    validate_safe_bench_command,
)
from scripts.bench.runner import main as bench_main, run_smoke_suite


def test_bench_session_enforces_safety_and_isolate() -> None:
    """Reject mutating commands and guarantee ISOLATE runs when leaving BenchSession."""
    mock = MockBenchTransport()
    with BenchSession(mock) as session:
        assert "PONG" in session.send("PING")
        with pytest.raises(BenchSafetyError):
            session.send("FLASH_ERASE 0x807F0000")
        with pytest.raises(BenchSafetyError):
            session.send("STATUS\nERASE")
    assert mock.history[-1] == "ISOLATE"


def test_gp26_sticky_low_detection_and_uart_decoders() -> None:
    """Detect sticky-LOW GP26 bug cleanly and parse COM10 electrical/boot telemetry."""
    smoke = run_smoke_suite()
    assert smoke["smoke_ok"] is True
    assert smoke["gp26_bug_detected"] is True
    assert smoke["gp26_recovered"] is True

    mock = MockBenchTransport()
    with BenchSession(mock) as session:
        pins = parse_pincheck_output(session.send("PINCHECK"))
        bus = parse_busdiag_output(session.send("BUSDIAG"))
        st = parse_status_output(session.send("STATUS"))
        jig = parse_am34_uart_log(session.send("JIGBOOT"))

    assert st.vdd_volts == pytest.approx(2.70)
    assert len(pins) == 4
    assert pins[0].hiz_volts == pytest.approx(3.25)
    assert bus.gp2_sole_active_tx is True
    assert bus.xreset_rise_us == 20
    assert jig.ddr_check_ok is True
    assert jig.loader_version == "ver0.61 [06/03/06]"
    assert jig.startup_reason == "70"
    assert -91006 in jig.sdcd_errors
    assert jig.sense_packets[0].vendor_mecha_code == 0x83A7


def test_utf16le_log_decoding_and_cli_smoke(tmp_path: Path) -> None:
    """Verify UTF-16LE capture decoding and CLI smoke-suite exit status."""
    sample = tmp_path / "normal_utf16.log"
    sample.write_bytes("DDR CHECK: OK\r\n***** normal mode *****\r\nBGO driver init failed[1].\r\n".encode("utf-16"))
    decoded = parse_am34_uart_log(read_captured_log(sample))
    assert decoded.ddr_check_ok is True
    assert decoded.boot_mode == "normal mode"
    assert decoded.bgo_failed_code == 1
    assert validate_safe_bench_command("PIOLOGIC 4000 JIG:350") == "PIOLOGIC 4000 JIG:350"
    assert validate_safe_bench_command("PANARSP 6000 JIG:350 PROBESEQ") == "PANARSP 6000 JIG:350 PROBESEQ"
    assert bench_main(["smoke-suite"]) == 0
