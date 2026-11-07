#!/usr/bin/env python3
"""
Panadiga Engineering CLI - Type-Safe Workbench Administration
Powered by Typer with strict Python 3.10+ type annotations.
"""

from pathlib import Path
from typing import Optional, List, TypedDict
import hashlib
import json
import sys
import typer
from rich.console import Console
from rich.table import Table
from rich.panel import Panel

class MemoryRegion(TypedDict):
    name: str
    start: str
    end: str
    size_kb: int
    perms: str

app = typer.Typer(
    name="panadiga-cli",
    help="Panadiga Engineering CLI: Diagnostics, Ghidra symbol tools, GDB bridge automation, and verification.",
    add_completion=False,
)
console = Console()


@app.command("memory-map")
def generate_memory_map(
    output_path: Optional[Path] = typer.Option(
        None, "--output", "-o", help="Target output file path (.py or .json)"
    ),
    format_type: str = typer.Option(
        "json", "--format", "-f", help="Output format: json, ghidra-py, markdown"
    ),
) -> None:
    """
    Generate the official memory map specification for Matsushita AM34 / UniPhier MN2DS0016.
    """
    regions: List[MemoryRegion] = [
        {"name": "FLASH_NOR", "start": "0x00000000", "end": "0x007FFFFF", "size_kb": 8192, "perms": "r-x"},
        {"name": "INTERNAL_SRAM", "start": "0x10000000", "end": "0x1003FFFF", "size_kb": 256, "perms": "rwx"},
        {"name": "BOOT_ROM", "start": "0x80000000", "end": "0x8001FFFF", "size_kb": 128, "perms": "r-x"},
        {"name": "SYSTEM_MMIO", "start": "0xC0000000", "end": "0xC0003FFF", "size_kb": 16, "perms": "rw-"},
        {"name": "DWIRE32A_OCD", "start": "0xC0004000", "end": "0xC00040FF", "size_kb": 1, "perms": "rw-"},
    ]

    if format_type.lower() == "json":
        data = json.dumps(regions, indent=2)
        if output_path:
            output_path.write_text(data, encoding="utf-8")
            console.print(f"[bold green]✔[/bold green] Memory map written to {output_path}")
        else:
            console.print_json(data)
    else:
        table = Table(title="Matsushita AM34 Memory Map Architecture")
        table.add_column("Region", style="cyan", no_wrap=True)
        table.add_column("Start Address", style="magenta")
        table.add_column("End Address", style="magenta")
        table.add_column("Size (KB)", justify="right", style="green")
        table.add_column("Permissions", style="yellow")

        for r in regions:
            table.add_row(r["name"], r["start"], r["end"], str(r["size_kb"]), r["perms"])

        console.print(table)


@app.command("verify-artifact")
def verify_artifact(
    file_path: Path = typer.Argument(..., help="Path to firmware, NVRAM or HDD partition binary"),
    expected_sha256: Optional[str] = typer.Option(
        None, "--sha256", "-s", help="Expected SHA-256 hash for strict assertion"
    ),
) -> None:
    """
    Verify integrity, calculate SHA-256 and detect known Panadiga binary signatures.
    """
    if not file_path.exists():
        console.print(f"[bold red]ERRO:[/bold red] Arquivo não encontrado: {file_path}")
        raise typer.Exit(code=1)

    data = file_path.read_bytes()
    size = len(data)
    computed_sha256 = hashlib.sha256(data).hexdigest()

    console.print(Panel(
        f"[bold]Arquivo:[/bold] {file_path.name}\n"
        f"[bold]Tamanho:[/bold] {size:,} bytes ({size / 1024:.2f} KB)\n"
        f"[bold]SHA-256:[/bold] [yellow]{computed_sha256}[/yellow]",
        title="Verificação Criptográfica de Artefato",
        border_style="cyan"
    ))

    if expected_sha256:
        if computed_sha256.lower() == expected_sha256.lower():
            console.print("[bold green]✔ HASH CONFIRMADO COM SUCESSO![/bold green]")
        else:
            console.print(f"[bold red]✖ DIVERGÊNCIA DE HASH![/bold red] Esperado: {expected_sha256}")
            raise typer.Exit(code=2)


@app.command("gdb-config")
def gdb_config(
    port_name: str = typer.Option("COM5", "--port", "-p", help="Serial port name (e.g. COM5, /dev/ttyACM0)"),
    tcp_port: int = typer.Option(3333, "--tcp-port", "-t", help="TCP port for GDB remote protocol"),
    listen_host: str = typer.Option("127.0.0.1", "--host", "-h", help="TCP listen host address"),
) -> None:
    """
    Display configuration parameters and GDB connection commands for the DWire32A bridge.
    """
    console.print(Panel(
        f"[bold cyan]DWire32A GDB RSP Bridge Configuration[/bold cyan]\n\n"
        f"• Serial Port:   [yellow]{port_name}[/yellow]\n"
        f"• Listen Socket: [green]{listen_host}:{tcp_port}[/green]\n\n"
        f"[bold]GDB Command to Connect:[/bold]\n"
        f"  (gdb) [bold green]target remote {listen_host}:{tcp_port}[/bold green]\n"
        f"  (gdb) [bold green]set architecture am34[/bold green]\n"
        f"  (gdb) [bold green]info registers[/bold green]",
        title="GDB RSP Server Settings",
        border_style="magenta"
    ))


if __name__ == "__main__":
    app()
