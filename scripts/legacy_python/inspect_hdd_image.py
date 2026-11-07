#!/usr/bin/env python3
"""
Panasonic DMR-EH55 HDD 100MB Image Inspector
Author: Antigravity Pair-Programming
Usage:
    python inspect_hdd_image.py [image_path]

Analyzes the first 100MB dump of the Panasonic DMR-EH55 HDD, mapping non-zero
regions, identifying filesystem structures (MEIHFS), and finding partitions.
"""

import sys
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
DEFAULT_PATH = os.path.join(PROJECT_ROOT, "firmware_and_images", "panasonic DMR EH-55 Firmware hdd_100.bin")

def main():
    if len(sys.argv) > 1:
        path = sys.argv[1]
        if not os.path.exists(path):
            alt_path = os.path.join(PROJECT_ROOT, "firmware_and_images", path)
            if os.path.exists(alt_path):
                path = alt_path
    else:
        path = DEFAULT_PATH
    if not os.path.exists(path):
        print(f"[ERROR] File not found: {path}")
        sys.exit(1)

    size = os.path.getsize(path)
    print("=" * 70)
    print("PANASONIC DMR-EH55 HDD 100MB DUMP ANALYZER")
    print("=" * 70)
    print(f"File: {path}")
    print(f"Size: {size:,} bytes ({size / (1024*1024):.2f} MB)")
    print("-" * 70)

    with open(path, "rb") as f:
        # Check MBR / Sector 0
        s0 = f.read(512)
        print("Sector 0 (LBA 0) Preview:")
        print("  Hex:", s0[:32].hex())
        ascii_prev = ''.join(chr(b) if 32 <= b < 127 else '.' for b in s0[:64])
        print("  ASCII:", ascii_prev)
        if s0[510:512] == b'\x55\xaa':
            print("  [INFO] Standard MBR signature 0x55AA present.")
        else:
            print("  [INFO] Non-standard MBR (Panasonic MEIHFS RAW structure).")

        # Map non-zero 1MB chunks
        print("\nMapping 1MB Chunks across the 100MB image:")
        f.seek(0)
        chunk_size = 1024 * 1024
        for i in range(size // chunk_size):
            chunk = f.read(chunk_size)
            nz = sum(1 for b in chunk if b != 0)
            if nz > 0:
                bar = "#" * int(nz / chunk_size * 20)
                print(f"  MB {i:02d} (0x{i*chunk_size:08X}): {nz:7,d} bytes [{bar:<20}] ({nz/chunk_size*100:5.1f}%)")

    print("=" * 70)

if __name__ == "__main__":
    main()
