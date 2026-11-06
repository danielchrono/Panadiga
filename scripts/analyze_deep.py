#!/usr/bin/env python3
import os
import re
import math
from collections import Counter

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))

HDD_PATH = os.path.join(PROJECT_ROOT, "firmware_and_images", "panasonic DMR EH-55 Firmware hdd_100.bin")
FRM_PATH = os.path.join(PROJECT_ROOT, "firmware_and_images", "PANA_DVD.FRM")

def scan_hdd():
    print("=== Scanning HDD Image ===")
    if not os.path.exists(HDD_PATH):
        print("HDD file not found")
        return

    with open(HDD_PATH, "rb") as f:
        size = os.path.getsize(HDD_PATH)
        print(f"Total HDD Dump Size: {size:,} bytes")
        
        # Scan for strings
        f.seek(0)
        data = f.read()
        
        keywords = [b"panasonic", b"diga", b"firmware", b"boot", b"dvd", b"version", b"error", b"system", b"copyright", b"spansion", b"mei", b"matsushita", b"dmr"]
        matches = list(re.finditer(rb"[A-Za-z0-9_\-\./ ]{6,}", data))
        found_kw = 0
        for m in matches:
            s = m.group()
            low = s.lower()
            for kw in keywords:
                if kw in low:
                    print(f"  [HDD String] 0x{m.start():08X}: {s.decode('latin1', errors='replace')} (Matched: {kw.decode()})")
                    found_kw += 1
                    break
            if found_kw >= 30:
                print("  [HDD String] ... more than 30 matches, truncating ...")
                break
        print(f"Total keyword matches: {found_kw}")

def scan_frm():
    print("\n=== Scanning PANA_DVD.FRM ===")
    if not os.path.exists(FRM_PATH):
        print("FRM file not found")
        return

    with open(FRM_PATH, "rb") as f:
        size = os.path.getsize(FRM_PATH)
        data = f.read()
        print(f"Total FRM Size: {size:,} bytes")
        
        keywords = [b"panasonic", b"diga", b"firmware", b"boot", b"dvd", b"version", b"error", b"system", b"copyright", b"spansion", b"mei", b"matsushita", b"dmr"]
        matches = list(re.finditer(rb"[A-Za-z0-9_\-\./ ]{6,}", data))
        found_kw = 0
        for m in matches:
            s = m.group()
            low = s.lower()
            for kw in keywords:
                if kw in low:
                    print(f"  [FRM String] 0x{m.start():08X}: {s.decode('latin1', errors='replace')} (Matched: {kw.decode()})")
                    found_kw += 1
                    break
            if found_kw >= 30:
                print("  [FRM String] ... more than 30 matches, truncating ...")
                break
        print(f"Total keyword matches: {found_kw}")

if __name__ == "__main__":
    scan_hdd()
    scan_frm()
