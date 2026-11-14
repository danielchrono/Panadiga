"""Non-destructive transport and session management for Pico P7501 bench probing."""

from __future__ import annotations

from dataclasses import dataclass, field
import importlib
import time
from typing import Final, Protocol, cast


SAFE_BENCH_COMMANDS: Final[tuple[str, ...]] = (
    "PING",
    "HELP",
    "BANNER",
    "STATUS",
    "VDD",
    "LED",
    "PINCHECK",
    "BUSDIAG",
    "SNIFF",
    "ISOLATE",
    "UARTMON",
    "BOOTCAP",
    "JIGBOOT",
    "UARTPROBE",
    "UARTSEND",
    "BAUD",
    "BAUDSCAN",
    "UARTSTREAM",
    "HEXCAP",
    "UARTTXHEX",
    "WAITTX",
    "PWRTRACE",
)

_FORBIDDEN_WRITE_TOKENS: Final[tuple[str, ...]] = (
    "ERASE",
    "WRITE",
    "FLASH",
    "PROGRAM",
    "WRITEMEM",
)

_PROMPT_MARKERS: Final[tuple[str, ...]] = ("P7501>", "PANADIGA>")


class BenchSafetyError(RuntimeError):
    """Raised when a command violates non-destructive bench safety rules."""


class BenchProtocolError(RuntimeError):
    """Raised on serial transport or prompt-framing failures."""


class _SerialPortLike(Protocol):
    """Minimal subset of pyserial.Serial used by SerialTransport."""

    timeout: float | None

    def write(self, data: bytes) -> int | None: ...
    def read(self, size: int = 1) -> bytes: ...
    def reset_input_buffer(self) -> None: ...
    def close(self) -> None: ...


class TransportProtocol(Protocol):
    """Abstract line-based command transport for live or mocked Pico probes."""

    def exchange(self, command: str, timeout_s: float = 4.0) -> str:
        """Send one validated bench command and collect output until prompt or timeout."""
        ...

    def close(self) -> None:
        """Release underlying communication resources."""
        ...


def validate_safe_bench_command(command: str) -> str:
    """Validate that `command` belongs to the read-only/diagnostic bench allowlist."""
    normalized = command.strip()
    if not normalized:
        raise BenchSafetyError("Empty bench command is not permitted")
    if "\n" in normalized or "\r" in normalized:
        raise BenchSafetyError("Multiline commands are rejected to prevent command injection")

    head = normalized.split()[0].upper()
    if head not in SAFE_BENCH_COMMANDS:
        raise BenchSafetyError(f"Command '{head}' is outside SAFE_BENCH_COMMANDS")

    upper_full = normalized.upper()
    for token in _FORBIDDEN_WRITE_TOKENS:
        if token in upper_full:
            raise BenchSafetyError(f"Forbidden mutating token '{token}' in bench command")
    return normalized


class SerialTransport:
    """PySerial-backed transport for a live Pico connected over USB CDC (e.g. COM10)."""

    def __init__(self, port: str, baudrate: int = 115200) -> None:
        serial_mod = importlib.import_module("serial")
        serial_cls = getattr(serial_mod, "Serial")
        self._port_name = port
        self._ser = cast(_SerialPortLike, serial_cls(port, baudrate=baudrate, timeout=0.1))
        self._ser.reset_input_buffer()

    def exchange(self, command: str, timeout_s: float = 4.0) -> str:
        """Transmit one validated command over USB CDC and read until prompt or deadline."""
        cmd = validate_safe_bench_command(command)
        self._ser.reset_input_buffer()
        self._ser.write((cmd + "\r\n").encode("ascii"))

        chunks: list[str] = []
        deadline = time.monotonic() + timeout_s
        while time.monotonic() < deadline:
            raw = self._ser.read(4096)
            if raw:
                text = raw.decode("utf-8", errors="replace")
                chunks.append(text)
                joined = "".join(chunks)
                if any(marker in joined for marker in _PROMPT_MARKERS):
                    return joined
            else:
                time.sleep(0.02)
        return "".join(chunks)

    def close(self) -> None:
        """Close the USB CDC serial handle."""
        self._ser.close()


def _empty_str_list() -> list[str]:
    return []


@dataclass
class MockBenchTransport:
    """Deterministic offline simulator matching empirical COM10 DMR-EH55 telemetry."""

    vdd_volts: float = 2.70
    simulate_gp26_sticky_low: bool = False
    target_is_normal_bgo_brick: bool = True
    sd_status_code: int = -91006
    history: list[str] = field(default_factory=_empty_str_list)
    _gp26_latched_low: bool = field(init=False, default=False)

    def exchange(self, command: str, timeout_s: float = 4.0) -> str:
        """Simulate Pico P7501 command execution offline for CI and regression tests."""
        del timeout_s
        cmd = validate_safe_bench_command(command)
        self.history.append(cmd)
        parts = cmd.split()
        head = parts[0].upper()

        if head == "PING":
            return "PING\r\nPONG\r\nP7501> "
        if head in ("ISOLATE", "UARTPROBE"):
            if self.simulate_gp26_sticky_low and head == "UARTPROBE":
                self._gp26_latched_low = False
            if head == "ISOLATE":
                return (
                    "ISOLATE\r\n"
                    "ISOLATE: OK (All 6 pins in High-Impedance Hi-Z mode; unified JST ZR-6 plug is electrically invisible)\r\n"
                    "P7501> "
                )
            return "=== UARTPROBE MODE 8 CAPTURE (0 bytes) ===\r\nUARTPROBE: OK\r\nP7501> "
        if head in ("STATUS", "VDD"):
            vdd = 0.02 if self._gp26_latched_low else self.vdd_volts
            state = "LOW" if vdd < 1.5 else "OK"
            return (
                f"{head}\r\n"
                f"VDD: {vdd:.2f}V ({state})\r\n"
                "  [STATUS] Lines static: GP2=1, GP3=1, GP4=1, GP5=1.\r\n"
                "P7501> "
            )
        if head == "PINCHECK":
            return self._render_pincheck()
        if head == "BUSDIAG":
            return (
                "BUSDIAG\r\n"
                "GP2 (Pin6 MAX TXD / UART RX): 872 transitions (250 ms)\r\n"
                "GP3 (Pin5 NSW5V / Static Hi-Z): 0 transitions (5.15V rail idle)\r\n"
                "GP4 (Pin3 MAX RXD / UART TX): 0 transitions (R7570 10k idle HIGH)\r\n"
                "GP5 (Pin2 XRESET): STRONG HIGH, XRESET Release Rise Time (Hi-Z) : 20 us\r\n"
                "BUSDIAG: OK\r\n"
                "P7501> "
            )
        if head in ("BOOTCAP", "JIGBOOT", "UARTMON", "UARTSEND"):
            if self.simulate_gp26_sticky_low:
                self._gp26_latched_low = True
            return self._render_capture(head)
        return f"{cmd}\r\nOK\r\nP7501> "

    def _render_pincheck(self) -> str:
        if self._gp26_latched_low:
            gp26 = "GP26 (Pin1 SW_UART) | 0.02V | 0.02V | 0.02V | STICKY LOW BUG"
        else:
            gp26 = "GP26 (Pin1 SW_UART) | 3.25V | 2.69V | 3.27V | Weak pull-up (Q7510 B-C clamp)"
        return (
            "PINCHECK\r\n"
            "=== P7501 6-PIN ELECTRICAL MATRIX (PINCHECK) ===\r\n"
            "Pin               | Hi-Z | Pull-Down | Pull-Up | Electrical State\r\n"
            f"{gp26}\r\n"
            "GP3 (Pin5 NSW5V   ) | 3.29V | 3.28V | 3.30V | Hi-Z Sense ONLY (Static rail ~5.15V upstream)\r\n"
            "GP4 (Pin3 MAX RXD ) | 3.28V | 2.91V | 3.29V | Idle HIGH (R7570 10k / Q7514)\r\n"
            "GP5 (Pin2 XRESET  ) | 3.29V | 2.85V | 3.30V | STRONG HIGH (Open-Drain control)\r\n"
            "PINCHECK: OK\r\n"
            "P7501> "
        )

    def _render_capture(self, head: str) -> str:
        if head == "JIGBOOT":
            return (
                "JIGBOOT\r\n"
                "DDR CHECK: OK\r\n"
                "Chip select = 1\r\n"
                "***** Loader ver0.61 [06/03/06] *****\r\n"
                "***** serial boot mode *****\r\n"
                "!!! start fmu to read & exec fvu. !!!\r\n"
                "wait 1 second.\r\n"
                "*** timer correspond initialize ***\r\n"
                "startup reason = [70]\r\n"
                "Main hung-up time changed 255[s].\r\n"
                "SD init start...\r\n"
                f"[SDCD]SetBusWidth:SD Status ERROR {self.sd_status_code}\r\n"
                "SD init failed[-90002].\r\n"
                "*** CD diag boot mode ***\r\n"
                "[RTSD] ATAPI Software Reset \r\n"
                "X(80062900)X(80023a00)** startup drive **\r\n"
                "X(80020401)59,[RTSD] === DRIVE ACCESS ERROR === ( 80040904 )\r\n"
                "*** CDB  [00 00 00 00 00 00 00 00 00 00 00 00] \r\n"
                "*** SENSE[70 00 04 00 00 00 00 0A 00 00 83 A7 09 04 00 00 00 00] \r\n"
                "X(80040904)0,Drive start failed[-2147219196].\r\n"
                f"{head}: OK\r\n"
                "P7501> "
            )
        bgo_line = "BGO driver init failed[1].\r\n" if self.target_is_normal_bgo_brick else "BGO driver init OK\r\n"
        return (
            f"{head}\r\n"
            "DDR CHECK: OK\r\n"
            "***** Loader ver0.61 [06/03/06] *****\r\n"
            "***** normal mode *****\r\n"
            f"{bgo_line}"
            f"{head}: OK\r\n"
            "P7501> "
        )

    def close(self) -> None:
        """No-op close for mock transport."""


class BenchSession:
    """Context manager guaranteeing bus Hi-Z isolation (`ISOLATE`) upon exit."""

    def __init__(self, transport: TransportProtocol) -> None:
        self._transport = transport

    def __enter__(self) -> BenchSession:
        return self

    def send(self, command: str, timeout_s: float = 4.0) -> str:
        """Execute a validated safe bench command."""
        return self._transport.exchange(command, timeout_s=timeout_s)

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        del exc_type, exc, tb
        try:
            self._transport.exchange("ISOLATE", timeout_s=2.0)
        finally:
            self._transport.close()
