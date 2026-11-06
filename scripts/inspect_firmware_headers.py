import struct
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))

HDD_PATH = os.path.join(PROJECT_ROOT, "firmware_and_images", "panasonic DMR EH-55 Firmware hdd_100.bin")
FRM_PATH = os.path.join(PROJECT_ROOT, "firmware_and_images", "PANA_DVD.FRM")

def hex_dump(data, start_offset=0, length=128):
    lines = []
    for i in range(0, min(length, len(data)), 16):
        chunk = data[i:i+16]
        hex_str = " ".join(f"{b:02X}" for b in chunk)
        ascii_str = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        lines.append(f"{start_offset+i:08X}  {hex_str:<48}  |{ascii_str}|")
    return "\n".join(lines)

with open(HDD_PATH, "rb") as f:
    f.seek(0x02000000)
    hdr32 = f.read(256)
    f.seek(0x04000000)
    hdr64 = f.read(256)

print("=== HDD Block at 0x02000000 (32MB) ===")
print(hex_dump(hdr32, 0x02000000, 128))

print("\n=== HDD Block at 0x04000000 (64MB) ===")
print(hex_dump(hdr64, 0x04000000, 128))

with open(FRM_PATH, "rb") as f:
    frm_hdr = f.read(256)

print("\n=== PANA_DVD.FRM (first 128 bytes) ===")
print(hex_dump(frm_hdr, 0x00000000, 128))
