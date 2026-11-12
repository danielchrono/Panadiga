#!/usr/bin/env python3
"""
Merge pre-built RP2040 (pico) and RP2350 (pico2) UF2 binaries into a single
universal multi-family UF2 (`panasonic_dwire32a_unified.uf2`).

Layout guarantees safe drag-and-drop and automated copy flashing on BOTH
RP2040 (`RPI-RP2`) and RP2350 (`RP2350`) USB bootroms:
  1. RP2350-E10 absolute workaround block(s) (`0xE48BFF57`) first.
  2. RP2040 blocks `0 .. N-2` (`0xE48BFF56`).
  3. RP2350 ARM-S blocks `0 .. M-2` (`0xE48BFF59`).
  4. Final completion blocks (`rp2040[N-1]`, `rp2350_arm[M-1]`) grouped in the
     trailing 1 KB so neither MCU reboots mid-transfer during OS USB writes.
"""

from __future__ import annotations

import argparse
import hashlib
import struct
from pathlib import Path

UF2_MAGIC_START0 = 0x0A324655
UF2_MAGIC_START1 = 0x9E5D5157
UF2_MAGIC_END = 0x0AB16F30
UF2_FLAG_FAMILY_ID_PRESENT = 0x00002000
UF2_BLOCK_SIZE = 512

FAMILY_RP2040 = 0xE48BFF56
FAMILY_RP2XXX_ABS = 0xE48BFF57
FAMILY_RP2350_ARM_S = 0xE48BFF59


def _parse_uf2_blocks(path: Path) -> list[tuple[int, bytes]]:
    raw = path.read_bytes()
    if len(raw) == 0 or (len(raw) % UF2_BLOCK_SIZE) != 0:
        raise ValueError(f"Invalid UF2 size in {path}: {len(raw)} bytes")

    blocks: list[tuple[int, bytes]] = []
    for idx in range(0, len(raw), UF2_BLOCK_SIZE):
        blk = raw[idx : idx + UF2_BLOCK_SIZE]
        m0, m1, flags, _addr, _psize, _bno, _nb, family_id = struct.unpack_from(
            "<8I", blk, 0
        )
        (mend,) = struct.unpack_from("<I", blk, UF2_BLOCK_SIZE - 4)
        if m0 != UF2_MAGIC_START0 or m1 != UF2_MAGIC_START1 or mend != UF2_MAGIC_END:
            raise ValueError(f"Corrupt UF2 magic in {path} at block {idx // UF2_BLOCK_SIZE}")
        if (flags & UF2_FLAG_FAMILY_ID_PRESENT) == 0:
            raise ValueError(f"Missing family ID flag in {path} at block {idx // UF2_BLOCK_SIZE}")
        blocks.append((family_id, blk))
    return blocks


def merge_unified_uf2(rp2040_path: Path, rp2350_path: Path, out_path: Path) -> tuple[int, str]:
    """Combine RP2040 and RP2350 UF2 files into one unified multi-family UF2."""
    rp2040_all = _parse_uf2_blocks(rp2040_path)
    rp2350_all = _parse_uf2_blocks(rp2350_path)

    rp2040_blocks = [blk for fid, blk in rp2040_all if fid == FAMILY_RP2040]
    rp2350_abs_blocks = [blk for fid, blk in rp2350_all if fid == FAMILY_RP2XXX_ABS]
    rp2350_arm_blocks = [blk for fid, blk in rp2350_all if fid != FAMILY_RP2XXX_ABS]

    if not rp2040_blocks:
        raise ValueError(f"No RP2040 blocks (0x{FAMILY_RP2040:08X}) found in {rp2040_path}")
    if not rp2350_arm_blocks:
        raise ValueError(f"No RP2350 executable blocks found in {rp2350_path}")

    merged: list[bytes] = []
    # RP2350-E10 erratum requires the 0xE48BFF57 absolute block before non-abs blocks.
    merged.extend(rp2350_abs_blocks)
    if len(rp2040_blocks) > 1:
        merged.extend(rp2040_blocks[:-1])
    if len(rp2350_arm_blocks) > 1:
        merged.extend(rp2350_arm_blocks[:-1])
    # Place final completion block of each family at EOF to avoid premature USB disconnect.
    merged.append(rp2040_blocks[-1])
    merged.append(rp2350_arm_blocks[-1])

    payload = b"".join(merged)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(payload)
    digest = hashlib.sha256(payload).hexdigest().upper()
    return len(merged), digest


def main() -> int:
    parser = argparse.ArgumentParser(description="Merge RP2040 and RP2350 UF2 images into a unified UF2.")
    parser.add_argument("--rp2040", type=Path, required=True, help="Path to panasonic_dwire32a_rp2040.uf2")
    parser.add_argument("--rp2350", type=Path, required=True, help="Path to panasonic_dwire32a_rp2350.uf2")
    parser.add_argument("--out", type=Path, required=True, help="Output panasonic_dwire32a_unified.uf2")
    args = parser.parse_args()

    total_blocks, sha256_hex = merge_unified_uf2(args.rp2040, args.rp2350, args.out)
    size_kb = (total_blocks * UF2_BLOCK_SIZE) / 1024.0
    print(f"[SUCCESS] GENERATED UNIFIED UF2: {args.out.name}")
    print(f"  Path    : {args.out.resolve()}")
    print(f"  Blocks  : {total_blocks} ({total_blocks * UF2_BLOCK_SIZE} bytes / {size_kb:.2f} KB)")
    print(f"  SHA-256 : {sha256_hex}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
