"""
PANADIGA LIVE HARDWARE AUTOMATION & TEST SUITE
Panasonic DMR-EH55 (Matsushita AM34 / UniPhier MN2DS0016)
Pico Probe on COM10 (P7501 TO WRITE JIGU)
"""

import sys
import time
import argparse
import serial
from typing import Optional

PORT = "COM10"
BAUD = 115200

def get_serial(port: str = PORT) -> serial.Serial:
    ser = serial.Serial(port, BAUD, timeout=0.2)
    ser.dtr = True
    ser.rts = True
    time.sleep(0.05)
    ser.reset_input_buffer()
    return ser

def send_command(ser: serial.Serial, cmd: str, wait_timeout: float = 5.0) -> str:
    ser.reset_input_buffer()
    ser.write((cmd.strip() + "\n").encode("ascii"))
    t0 = time.time()
    out = b""
    cmd_bytes = cmd.strip().encode("ascii")
    while time.time() - t0 < wait_timeout:
        chunk = ser.read(2048)
        if chunk:
            out += chunk
            # Check if prompt returned after command
            if b"DWIRE32A>" in out.split(cmd_bytes, 1)[-1]:
                break
    return out.decode("latin1", errors="replace")

def test_pincheck(ser: serial.Serial) -> bool:
    print("\n========================================================")
    print(" [TEST 1] VERIFICAÇÃO ELÉTRICA & TENSÕES (PINCHECK)")
    print("========================================================")
    resp = send_command(ser, "PINCHECK", wait_timeout=3.0)
    print(resp.strip())
    
    # Check VDD
    vdd_ok = "VDD: 0.0" not in resp and "0.00V" not in resp
    if not vdd_ok:
        print("\n[ALERTA CRÍTICO] VDD do alvo está em 0V! Verifique se a Panasonic está ligada na tomada.")
    else:
        print("\n[OK] Barramento energizado e detectado com sucesso.")
    return vdd_ok

def test_normal_boot(ser: serial.Serial, duration_sec: int = 15) -> str:
    print(f"\n========================================================")
    print(f" [TEST 2] CAPTURA DE BOOT MODO NORMAL ({duration_sec}s)")
    print(f"========================================================")
    resp = send_command(ser, f"BOOTCAP {duration_sec * 1000}", wait_timeout=duration_sec + 4.0)
    print(resp.strip())
    return resp

def test_jigboot(ser: serial.Serial, duration_sec: int = 15) -> str:
    print(f"\n========================================================")
    print(f" [TEST 3] CAPTURA MODO DE FÁBRICA / JIGBOOT ({duration_sec}s)")
    print(f"========================================================")
    resp = send_command(ser, f"JIGBOOT {duration_sec * 1000}", wait_timeout=duration_sec + 4.0)
    print(resp.strip())
    return resp

def test_fmuscan(ser: serial.Serial, mode: int = 1) -> str:
    print(f"\n========================================================")
    print(f" [TEST 4] SWEEP DE HANDSHAKE FMU SERIAL (MODE {mode})")
    print(f"========================================================")
    resp = send_command(ser, f"FMUSCAN {mode}", wait_timeout=8.0)
    print(resp.strip())
    return resp

def interactive_monitor(ser: serial.Serial):
    print("\n========================================================")
    print(" [MODO INTERATIVO] MONITOR CONTÍNUO COM CONTROLE LIVE")
    print(" Comandos rápidos:")
    print("   [b] Normal Boot (BOOTCAP 10s)")
    print("   [j] Factory Boot (JIGBOOT 12s)")
    print("   [f] FMU Handshake Sweep")
    print("   [p] Pincheck & Tensões")
    print("   [m] Monitor Passivo (15s)")
    print("   [q] Sair")
    print("========================================================")
    
    while True:
        try:
            choice = input("\nEscolha uma ação [b/j/f/p/m/q]: ").strip().lower()
            if choice == "q":
                break
            elif choice == "b":
                test_normal_boot(ser, 12)
            elif choice == "j":
                test_jigboot(ser, 15)
            elif choice == "f":
                test_fmuscan(ser, 1)
            elif choice == "p":
                test_pincheck(ser)
            elif choice == "m":
                print("Monitorando UART por 15s...")
                resp = send_command(ser, "UARTMON 15000", wait_timeout=17.0)
                print(resp.strip())
            else:
                print("Opção inválida.")
        except KeyboardInterrupt:
            break

def main():
    parser = argparse.ArgumentParser(description="Panadiga Hardware Automation Suite")
    parser.add_argument("--test", choices=["pincheck", "normal", "jig", "fmu", "all", "interactive"], default="all")
    parser.add_argument("--port", default=PORT, help="Porta serial do Pico")
    args = parser.parse_args()

    try:
        ser = get_serial(args.port)
    except Exception as e:
        print(f"[ERRO] Não foi possível abrir {args.port}: {e}")
        sys.exit(1)

    print(f"[CONECTADO] Raspberry Pi Pico em {args.port} (115200 bps)")

    try:
        if args.test == "pincheck":
            test_pincheck(ser)
        elif args.test == "normal":
            test_normal_boot(ser, 20)
        elif args.test == "jig":
            test_jigboot(ser, 20)
        elif args.test == "fmu":
            test_fmuscan(ser, 1)
        elif args.test == "interactive":
            interactive_monitor(ser)
        elif args.test == "all":
            ok = test_pincheck(ser)
            if ok:
                test_jigboot(ser, 15)
                test_fmuscan(ser, 1)
    finally:
        ser.close()
        print("\n[DESCONECTADO] Sessão serial encerrada com segurança.")

if __name__ == "__main__":
    main()
