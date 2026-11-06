#!/usr/bin/env python3
"""
Panasonic DMR-EH55 NVRAM Cleaner for Spansion S29JL064H (8MB / TSOP-48)
Author: Antigravity Pair-Programming
Usage:
    python clean_spansion_nvram.py <input_dump.bin> [output_clean.bin]

This script:
1. Validates the raw 8MB dump (exactly 8,388,608 bytes).
2. Extracts the corrupted NVRAM/EEMUL block (0x007F0000 - 0x007FFFFF, 64KB) to nvram_corrupt_backup.bin.
3. Generates a clean firmware binary with the NVRAM sector erased to 0xFF.
4. Verifies that the OS/Bootloader area (0x00000000 - 0x007EFFFF) is 100% untouched.
5. Prints full cryptographic hashes (MD5, SHA-256) for auditability.
"""

import sys
import os
import hashlib

FLASH_SIZE = 8 * 1024 * 1024  # 8,388,608 bytes (64 Mbit)
NVRAM_START = 0x007F0000
NVRAM_END   = 0x00800000      # 64 KB block
NVRAM_SIZE  = NVRAM_END - NVRAM_START

def calc_hashes(data: bytes):
    md5 = hashlib.md5(data).hexdigest()
    sha256 = hashlib.sha256(data).hexdigest()
    return md5, sha256

def main():
    if len(sys.argv) < 2:
        print("Usage: python clean_spansion_nvram.py <input_dump.bin> [output_clean.bin]")
        print("\nExample:")
        print("  python clean_spansion_nvram.py DMR_EH55_ORIGINAL.bin DMR_EH55_CLEAN.bin")
        sys.exit(1)

    input_file = sys.argv[1]
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.abspath(os.path.join(script_dir, ".."))

    if not os.path.exists(input_file):
        alt = os.path.join(project_root, "firmware_and_images", input_file)
        if os.path.exists(alt):
            input_file = alt
        else:
            print(f"[ERROR] Input file not found: {input_file}")
            sys.exit(1)

    if len(sys.argv) >= 3:
        output_file = sys.argv[2]
    else:
        base, ext = os.path.splitext(input_file)
        output_file = f"{base}_CLEAN{ext}"

    with open(input_file, "rb") as f:
        data = bytearray(f.read())

    if len(data) != FLASH_SIZE:
        print(f"[ERROR] Invalid flash dump size: {len(data)} bytes.")
        print(f"        Expected Spansion S29JL064H size: {FLASH_SIZE} bytes (8MB).")
        sys.exit(1)

    orig_md5, orig_sha = calc_hashes(data)
    print("=" * 70)
    print("PANASONIC DMR-EH55 FLASH PATCHER - SPANSION S29JL064H")
    print("=" * 70)
    print(f"Input file:       {input_file}")
    print(f"Size:             {len(data):,} bytes (8.00 MB)")
    print(f"Original MD5:     {orig_md5}")
    print(f"Original SHA-256: {orig_sha}")
    print("-" * 70)

    # Extract NVRAM block
    nvram_data = bytes(data[NVRAM_START:NVRAM_END])
    backup_nvram_path = os.path.join(os.path.dirname(output_file) or ".", "nvram_corrupt_backup.bin")
    with open(backup_nvram_path, "wb") as f:
        f.write(nvram_data)
    print(f"[OK] Corrupted NVRAM sector (0x{NVRAM_START:06X} - 0x{NVRAM_END-1:06X}) extracted to:")
    print(f"     -> {backup_nvram_path} ({len(nvram_data):,} bytes)")

    # Analyze corrupt NVRAM content
    non_ff = sum(1 for b in nvram_data if b != 0xFF)
    non_zero = sum(1 for b in nvram_data if b != 0x00)
    print(f"     Non-FF bytes in NVRAM:  {non_ff:,} / {NVRAM_SIZE:,} ({non_ff/NVRAM_SIZE*100:.1f}%)")
    print(f"     Non-zero bytes:         {non_zero:,} / {NVRAM_SIZE:,} ({non_zero/NVRAM_SIZE*100:.1f}%)")
    print("     Detalhamento por Micro-Setor de 8 KB (SA134 a SA141):")
    for idx in range(8):
        s_off = idx * 0x2000
        sec = nvram_data[s_off : s_off + 0x2000]
        used = sum(1 for b in sec if b != 0xFF)
        print(f"       SA{134+idx} (0x{NVRAM_START+s_off:06X}): {used:4d}/8192 bytes ativos | Header: {sec[:8].hex().upper()}")

    # Clean NVRAM sector with 0xFF
    for i in range(NVRAM_START, NVRAM_END):
        data[i] = 0xFF

    # Verify untouched OS area
    with open(input_file, "rb") as f:
        orig_bytes = f.read(NVRAM_START)
    if bytes(data[:NVRAM_START]) != orig_bytes:
        print("[CRITICAL ERROR] Bootloader/OS area was inadvertently modified! Aborting.")
        sys.exit(1)
    print("[OK] Verified: System OS & Bootloader (first 8,323,072 bytes) are 100% IDENTICAL.")

    # Write clean image
    with open(output_file, "wb") as f:
        f.write(data)

    clean_md5, clean_sha = calc_hashes(data)
    print("-" * 70)
    print(f"[SUCCESS] Clean firmware binary saved to:")
    print(f"     -> {output_file}")
    print(f"Clean MD5:        {clean_md5}")
    print(f"Clean SHA-256:    {clean_sha}")
    print("=" * 70)
    print("READY TO FLASH BACK TO SPANSION S29JL064H VIA XGECU T48 / TL866II!")
    print("When booted, the Panasonic CPU will initialize clean factory parameters.")
    print("=" * 70)

if __name__ == "__main__":
    main()
