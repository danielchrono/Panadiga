"""Non-destructive CLI runner for Pico P7501 bench diagnostics and smoke tests."""

from __future__ import annotations

import argparse
from dataclasses import asdict
import json
from pathlib import Path
from typing import Any

from scripts.bench.decoders import (
    parse_am34_uart_log,
    parse_busdiag_output,
    parse_pincheck_output,
    parse_status_output,
    read_captured_log,
)
from scripts.bench.protocol import BenchSession, MockBenchTransport, SerialTransport, TransportProtocol


def _make_transport(port: str | None, simulate_bug: bool = False) -> TransportProtocol:
    if port:
        return SerialTransport(port)
    return MockBenchTransport(simulate_gp26_sticky_low=simulate_bug)


def run_health(port: str | None) -> dict[str, Any]:
    """Execute read-only electrical health verification (`PING`, `STATUS`, `PINCHECK`, `BUSDIAG`)."""
    with BenchSession(_make_transport(port)) as session:
        ping = session.send("PING", timeout_s=2.0)
        status = parse_status_output(session.send("STATUS", timeout_s=2.0))
        pins = parse_pincheck_output(session.send("PINCHECK", timeout_s=3.0))
        bus = parse_busdiag_output(session.send("BUSDIAG", timeout_s=3.0))
    return {
        "pong": "PONG" in ping,
        "status": {
            "vdd_volts": status.vdd_volts,
            "vdd_status": status.vdd_status,
            "gp26_sticky_low_suspected": status.gp26_sticky_low_suspected,
        },
        "pincheck_rows": [asdict(r) for r in pins],
        "busdiag": asdict(bus),
    }


def run_capture(command: str, port: str | None, out_path: Path | None) -> dict[str, Any]:
    """Execute `BOOTCAP`, `JIGBOOT`, or `UARTMON`, decode AM34 output, and isolate bus."""
    with BenchSession(_make_transport(port)) as session:
        raw_log = session.send(command, timeout_s=6.0)
    if out_path is not None:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(raw_log, encoding="utf-8")
    decoded = parse_am34_uart_log(raw_log)
    return {"command": command, "bytes": len(raw_log), "decoded": asdict(decoded)}


def run_smoke_suite() -> dict[str, Any]:
    """Run offline self-check verifying health, sticky-LOW bug detection, and JIGBOOT decoding."""
    health_res = run_health(None)
    jig_res = run_capture("JIGBOOT", None, None)

    bug_tr = MockBenchTransport(simulate_gp26_sticky_low=True)
    with BenchSession(bug_tr) as session:
        _ = session.send("UARTSEND VER")
        bug_status = parse_status_output(session.send("STATUS"))
        _ = session.send("UARTPROBE 8")
        fixed_status = parse_status_output(session.send("STATUS"))

    assert health_res["pong"] is True
    assert bug_status.gp26_sticky_low_suspected is True
    assert fixed_status.gp26_sticky_low_suspected is False
    assert jig_res["decoded"]["boot_mode"] == "serial boot mode"

    return {
        "smoke_ok": True,
        "health_vdd": health_res["status"]["vdd_volts"],
        "gp26_bug_detected": bug_status.gp26_sticky_low_suspected,
        "gp26_recovered": not fixed_status.gp26_sticky_low_suspected,
        "jigboot_notes": jig_res["decoded"]["notes"],
    }


def main(argv: list[str] | None = None) -> int:
    """CLI entrypoint for `python -m scripts.bench.runner`."""
    parser = argparse.ArgumentParser(description="Panadiga P7501 Non-Destructive Bench Runner")
    sub = parser.add_subparsers(dest="subcommand", required=True)

    p_health = sub.add_parser("health", help="Run PING/STATUS/PINCHECK/BUSDIAG/ISOLATE")
    p_health.add_argument("--port", default=None, help="Serial port (e.g. COM10); omit for --mock")
    p_health.add_argument("--mock", action="store_true", help="Force offline deterministic mock transport")

    p_cap = sub.add_parser("capture", help="Run BOOTCAP/JIGBOOT/UARTMON and decode UART stream")
    p_cap.add_argument("cmd", choices=["BOOTCAP", "JIGBOOT", "UARTMON"])
    p_cap.add_argument("--port", default=None)
    p_cap.add_argument("--mock", action="store_true")
    p_cap.add_argument("--out", type=Path, default=None)

    p_dec = sub.add_parser("decode-log", help="Decode an existing captured UART log file")
    p_dec.add_argument("path", type=Path)

    sub.add_parser("smoke-suite", help="Run offline end-to-end bench verification suite")

    args = parser.parse_args(argv)
    result: dict[str, Any]
    if args.subcommand == "health":
        result = run_health(None if args.mock else args.port)
    elif args.subcommand == "capture":
        result = run_capture(args.cmd, None if args.mock else args.port, args.out)
    elif args.subcommand == "decode-log":
        text = read_captured_log(args.path)
        result = asdict(parse_am34_uart_log(text))
    else:
        result = run_smoke_suite()

    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
