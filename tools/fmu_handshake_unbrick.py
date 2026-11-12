#!/usr/bin/env python3
"""
Panasonic DMR-EH55 — Handshake Serial via FMU em P7501 (Caminho 1)
==================================================================
Executa automaticamente pelo Raspberry Pi Pico (COM10):
  1. GP26 (SW_UART) = LOW durante o pulso de XRESET (GP5) para entrar em
     `***** serial boot mode *****`.
  2. Assim que o AM34 entra em `serial boot mode`, sobe GP26 = HIGH e mantém
     GP3 (NSW5V) = HIGH (3.3V) para polarizar a base de Q7514 antes de o `fmu`
     abrir a janela `wait 1 second.`.
  3. Varre os modos FMUSCAN (2, 3, 0) e as sequências estruturadas de
     sincronismo do `fmu` via FMUTEST sem provocar UART Overrun (OER) no AM34.
"""

import os
import shutil
import sys
import time
import serial
import serial.tools.list_ports

UF2_PATH = r"C:\Users\danie\Documents\Panadiga\dist\panasonic_dwire32a_rp2040.uf2"


def find_pico_port(timeout_s: float = 30.0) -> str | None:
    t0 = time.time()
    while time.time() - t0 < timeout_s:
        for p in serial.tools.list_ports.comports():
            if "2E8A" in (p.hwid or "").upper() or p.device.upper() == "COM10":
                return p.device
        time.sleep(0.5)
    return None


def send_cmd(ser: serial.Serial, cmd: str, timeout_s: float = 6.5) -> str:
    ser.reset_input_buffer()
    ser.write((cmd.strip() + "\r\n").encode("ascii"))
    ser.flush()
    t0 = time.time()
    buf = b""
    while time.time() - t0 < timeout_s:
        n = ser.in_waiting
        chunk = ser.read(n if n > 0 else 1)
        if chunk:
            buf += chunk
            if b"DWIRE32A>" in buf:
                break
    return buf.decode("utf-8", errors="replace")


def auto_update_uf2_if_needed(port: str) -> str:
    with serial.Serial(port, 115200, timeout=0.5) as ser:
        out = send_cmd(ser, "FMUTEST 2 0D", timeout_s=6.5)
        if "Unknown command" not in out:
            print("[+] Firmware do Pico já suporta FMUTEST!")
            print(out)
            return port

        print("[*] Atualizando firmware do Pico com suporte a FMUTEST via BOOTSEL...")
        try:
            ser.write(b"BOOTSEL\r\n")
            ser.flush()
        except Exception:
            pass

    # Aguarda unidade RPI-RP2 montar
    t0 = time.time()
    drive_root = None
    while time.time() - t0 < 15.0:
        for letter in "DEFGHIJKLMNOPQRSTUVWXYZ":
            cand = f"{letter}:\\"
            if os.path.exists(os.path.join(cand, "INFO_UF2.TXT")):
                drive_root = cand
                break
        if drive_root:
            break
        time.sleep(0.4)

    if not drive_root:
        raise RuntimeError("Unidade RPI-RP2 não encontrada após BOOTSEL.")

    dst = os.path.join(drive_root, "panasonic_dwire32a_rp2040.uf2")
    shutil.copyfile(UF2_PATH, dst)
    print(f"[+] UF2 copiado para {dst}. Aguardando reconexão serial...")
    time.sleep(2.0)
    new_port = find_pico_port(timeout_s=15.0)
    if not new_port:
        raise RuntimeError("Pico não reconectou na porta serial após gravação do UF2.")
    return new_port


def main() -> int:
    print("======================================================================")
    print(" PANASONIC DMR-EH55 — HANDSHAKE SERIAL FMU EM P7501 (VIA PICO)        ")
    print("======================================================================")
    port = find_pico_port(timeout_s=5.0)
    if not port:
        print("[!] Pico (COM10 / VID:2E8A) não detectado no barramento USB no momento.")
        print("    Aguardando até 60s você reconectar o cabo USB do Pico...")
        port = find_pico_port(timeout_s=60.0)
        if not port:
            print("[ERRO] Pico não encontrado em nenhuma porta COM.")
            return 1

    print(f"[+] Pico detectado em {port}!")

    with serial.Serial(port, 115200, timeout=0.5) as ser:
        print("\n--- [1/3] Verificando PINCHECK e STATUS ---")
        print(send_cmd(ser, "STATUS", timeout_s=3.0))
        print(send_cmd(ser, "PINCHECK", timeout_s=3.0))

        print("\n--- [2/3] Executando FMUSCAN (Modos 2, 3 e 0) ---")
        for mode in (2, 3, 0):
            print(f"\n>>> Testando FMUSCAN {mode} <<<")
            res = send_cmd(ser, f"FMUSCAN {mode}", timeout_s=7.0)
            print(res)

    port = auto_update_uf2_if_needed(port)

    # Sequências candidatas de Handshake do FMU Matsushita (sem causar UART Overrun)
    candidates = [
        ("ENQ (0x05)", 2, "05"),
        ("STX (0x02)", 2, "02"),
        ("ACK (0x06)", 2, "06"),
        ("CR (\\r)", 2, "0D"),
        ("CRLF (\\r\\n)", 2, "0D0A"),
        ("SPACE (0x20)", 2, "20"),
        ("ESC (0x1B)", 2, "1B"),
        ("Sync 0x55 0xAA", 2, "55AA"),
        ("ASCII 'fvu\\r'", 2, "6676750D"),
        ("ASCII 'FVU\\r'", 2, "4656550D"),
        ("ASCII 'fmu\\r'", 2, "666D750D"),
        ("Motorola S-Rec S0", 2, "533030333030303046430D"),
        ("Repeat ENQ (0x05) @ 50ms", 12, "05"),
        ("Repeat CR (0x0D) @ 50ms", 12, "0D"),
        ("Repeat 'fvu\\r' @ 50ms", 12, "6676750D"),
    ]

    with serial.Serial(port, 115200, timeout=0.5) as ser:
        print("\n--- [3/3] Executando Handshake Estruturado FMUTEST (Q7514 Polarizado) ---")
        for label, mode, hex_payload in candidates:
            print(f"\n>>> [{label}] -> FMUTEST {mode} {hex_payload} <<<")
            res = send_cmd(ser, f"FMUTEST {mode} {hex_payload}", timeout_s=7.0)
            print(res)

    return 0


if __name__ == "__main__":
    sys.exit(main())
