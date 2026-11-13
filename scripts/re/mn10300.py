"""Matsushita MN103S / AM34 instruction cross-reference helpers.

Encodes the empirically verified MN10300 / MN103S PC-relative encoding rule:
relative displacement is computed from the address of the current instruction
(`target_va = instruction_va + signed_disp`), not the following instruction.
"""

from __future__ import annotations

from dataclasses import dataclass
import struct
from typing import Final

# Opcode -> (mnemonic, struct_fmt, instruction_len)
_REL_BRANCH_TABLE: Final[dict[int, tuple[str, str, int]]] = {
    0xCA: ("bra8", "<b", 2),
    0xCC: ("jmp16", "<h", 3),
    0xCD: ("call16", "<h", 3),
    0xDC: ("jmp32", "<i", 5),
    0xDD: ("call32", "<i", 5),
}


@dataclass(frozen=True)
class InstructionRef:
    """Decoded MN103S relative call or jump instruction."""

    va: int
    opcode: int
    mnemonic: str
    displacement: int
    target_va: int


@dataclass(frozen=True)
class ImmediateLoadRef:
    """Decoded 6-byte `0xFC <sub> <imm32>` extended immediate load."""

    va: int
    sub_opcode: int
    value: int
    c_string: str | None


def _extract_ascii_or_cp932(buf: bytes, offset: int, max_len: int = 80) -> str | None:
    """Decode a printable CP932 C-string at `offset`, or return None."""
    if offset < 0 or offset >= len(buf):
        return None
    end = buf.find(b"\x00", offset)
    if end == -1 or end - offset > max_len:
        end = min(len(buf), offset + max_len)
    raw = buf[offset:end]
    if not raw:
        return None
    try:
        decoded = raw.decode("cp932")
    except UnicodeDecodeError:
        return None
    if all((0x20 <= ord(ch) < 0xFF00) or ch in "\r\n\t" for ch in decoded):
        return decoded
    return None


def decode_branch_call_refs(
    buf: bytes,
    image_base: int,
    va_start: int | None = None,
    va_end: int | None = None,
    max_abs_disp: int = 0x200000,
) -> list[InstructionRef]:
    """Scan a binary window for MN103S relative branches/calls targeting `[image_base, image_end)`."""
    lo = 0 if va_start is None else max(0, va_start - image_base)
    hi = len(buf) if va_end is None else min(len(buf), va_end - image_base)
    upper_va = image_base + len(buf)
    refs: list[InstructionRef] = []

    for off in range(lo, hi):
        op = buf[off]
        spec = _REL_BRANCH_TABLE.get(op)
        if spec is None:
            continue
        mnemonic, fmt, insn_len = spec
        if off + insn_len > len(buf):
            continue
        disp = int(struct.unpack_from(fmt, buf, off + 1)[0])
        if disp == 0 or abs(disp) > max_abs_disp:
            continue
        va = image_base + off
        target = va + disp
        if image_base <= target < upper_va:
            refs.append(InstructionRef(va, op, mnemonic, disp, target))
    return refs


def find_callers_to(
    buf: bytes,
    image_base: int,
    targets: set[int] | list[int] | tuple[int, ...],
) -> list[InstructionRef]:
    """Return every MN103S branch or call referencing any virtual address in `targets`."""
    target_set = set(targets)
    return [ref for ref in decode_branch_call_refs(buf, image_base) if ref.target_va in target_set]


def find_fc_immediate_loads(
    buf: bytes,
    image_base: int,
    va_start: int,
    va_end: int,
) -> list[ImmediateLoadRef]:
    """Extract 6-byte `0xFC <sub_op> <u32>` immediate/pointer loads in a VA range."""
    lo = max(0, va_start - image_base)
    hi = min(len(buf) - 6, va_end - image_base)
    upper_va = image_base + len(buf)
    results: list[ImmediateLoadRef] = []
    for off in range(lo, hi + 1):
        if buf[off] != 0xFC:
            continue
        sub = buf[off + 1]
        val = int(struct.unpack_from("<I", buf, off + 2)[0])
        cstr = _extract_ascii_or_cp932(buf, val - image_base) if image_base <= val < upper_va else None
        results.append(ImmediateLoadRef(image_base + off, sub, val, cstr))
    return results


def find_pointer_xrefs(buf: bytes, image_base: int, target_va: int) -> list[int]:
    """Find every 32-bit little-endian literal reference to `target_va` inside `buf`."""
    needle = struct.pack("<I", target_va)
    hits: list[int] = []
    pos = 0
    while True:
        idx = buf.find(needle, pos)
        if idx == -1:
            break
        hits.append(image_base + idx)
        pos = idx + 1
    return hits
