"""Unit tests for Matsushita MN103S and PANA_DVD.FRM static reverse-engineering helpers."""

from __future__ import annotations

from pathlib import Path
import struct
import zlib

from scripts.re.mn10300 import (
    decode_branch_call_refs,
    find_callers_to,
    find_fc_immediate_loads,
    find_pointer_xrefs,
)
from scripts.re.pana_frm import (
    adler32_panasonic,
    decompress_drv_payload,
    derive_round_key,
    feistel_decrypt,
    feistel_encrypt,
    find_extrhead_containers,
    pack_extrhead,
    parse_atapi_cdb_dispatch,
    parse_decrypted_frm_toc,
    parse_drv_cli_table,
    parse_drv_error_table,
    parse_sense_lut,
    reconstruct_master_key,
    unpack_extrhead,
)


def test_mn10300_pc_relative_rule_cur_insn() -> None:
    """Verify `target_va = instruction_va + signed_disp` for 16-bit and 32-bit calls."""
    # Image loaded at 0x44000000: printf at +0x10 (0x44000010), call32 at +0x169 (0x44000169)
    # Disp = 0x10 - 0x169 = -0x159
    buf = bytearray(0x200)
    buf[0x169] = 0xDD
    struct.pack_into("<i", buf, 0x16A, -0x159)
    # Also place call16 at +0x100 targeting +0x80 -> disp = -0x80
    buf[0x100] = 0xCD
    struct.pack_into("<h", buf, 0x101, -0x80)
    # Place FC CC cstring load pointing to +0x20 ("Loader ver0.61")
    buf[0x20:0x2F] = b"Loader ver0.61\x00"
    buf[0x40] = 0xFC
    buf[0x41] = 0xCC
    struct.pack_into("<I", buf, 0x42, 0x44000020)
    # Place literal 32-bit pointer xref to 0x44000010
    struct.pack_into("<I", buf, 0x1C0, 0x44000010)

    raw = bytes(buf)
    refs = decode_branch_call_refs(raw, 0x44000000)
    assert any(r.va == 0x44000169 and r.target_va == 0x44000010 and r.mnemonic == "call32" for r in refs)
    callers = find_callers_to(raw, 0x44000000, [0x44000080])
    assert [(c.va, c.mnemonic) for c in callers] == [(0x44000100, "call16")]

    loads = find_fc_immediate_loads(raw, 0x44000000, 0x44000040, 0x44000050)
    assert len(loads) == 1
    assert loads[0].c_string == "Loader ver0.61"
    assert find_pointer_xrefs(raw, 0x44000000, 0x44000010) == [0x440001C0]


def test_pana_frm_tables_and_cp932_mecha_dict() -> None:
    """Verify EXTRHEAD, DRV zlib decompression, CLI table, ATAPI dispatch, Sense LUT, and 0x83A7."""
    # Build compressed DRV payload
    compressed = b"\x00" * 0x40 + zlib.compress(b"DRV_DECOMP_OK")
    assert decompress_drv_payload(compressed) == b"DRV_DECOMP_OK"

    # Build synthetic binary containing EXTRHEAD + tables
    img = bytearray(0x400)
    img[0x10:0x18] = b"EXTRHEAD"
    # CLI table at 0x40 (2 entries, 20 bytes each)
    struct.pack_into("<8sIII", img, 0x40, b"SLOGON\x00\x00", 0x40010000, 0, 1)
    struct.pack_into("<8sIII", img, 0x54, b"CDRW\x00\x00\x00\x00", 0x40020000, 1, 2)
    # ATAPI dispatch at 0x80 (4 entries)
    struct.pack_into("<IIII", img, 0x80, 0x40014B00, 0x40014559, 0x40014559, 0x40014C00)
    # Sense LUT at 0xA0 (2 entries)
    img[0xA0:0xA6] = bytes([0x02, 0x04, 0x01, 0x04, 0x09, 0x04])
    # Error dict at 0x100 (stride 0x34)
    err_txt = "0.5mW発光ができない(HFｵﾝ密調整)".encode("cp932")
    struct.pack_into("<H", img, 0x100, 0x83A7)
    img[0x102 : 0x102 + len(err_txt)] = err_txt

    raw = bytes(img)
    assert [m.offset for m in find_extrhead_containers(raw, 0x44000000)] == [0x10]
    cli = parse_drv_cli_table(raw, 0x40, count=2)
    assert [(c.name, c.handler_va) for c in cli] == [("SLOGON", 0x40010000), ("CDRW", 0x40020000)]

    atapi = parse_atapi_cdb_dispatch(raw, 0x80, count=4)
    assert [e.is_active for e in atapi] == [True, False, False, True]

    senses = parse_sense_lut(raw, 0xA0, count=2)
    assert (senses[1].sense_key, senses[1].asc, senses[1].ascq) == (0x04, 0x09, 0x04)

    mecha = parse_drv_error_table(raw, 0x100, 0x168)
    assert len(mecha) == 1
    assert mecha[0].code == 0x83A7
    assert "0.5mW発光ができない" in mecha[0].message_jp


def _pseudo_random_bytes(size: int, seed: int = 0x12345678) -> bytes:
    """Deterministic incompressible byte sequence to keep zlib payloads larger than crypt spans."""
    state = seed
    out = bytearray(size)
    for idx in range(size):
        state = (state * 1664525 + 1013904223) & 0xFFFFFFFF
        out[idx] = (state >> 24) & 0xFF
    return bytes(out)


def test_feistel_and_extrhead_split_pack_unpack() -> None:
    """Verify key derivation, Feistel ECB blocks, strided Adler-32, and split pack/unpack modes."""
    master = reconstruct_master_key()
    assert master == bytes.fromhex("df6215c7527fd0")
    rk = derive_round_key(master)
    assert rk == bytes.fromhex("a3e79c01b25a5474591db2ef67266dff63038707b656940f026da5612551f315")

    assert feistel_encrypt(b"\x00" * 8) == bytes.fromhex("e5d1f34a518b2690")
    assert feistel_encrypt(b"\xff" * 8) == bytes.fromhex("035065c87bb8e090")
    assert feistel_decrypt(bytes.fromhex("e5d1f34a518b2690")) == b"\x00" * 8
    assert feistel_decrypt(bytes.fromhex("035065c87bb8e090")) == b"\xff" * 8

    vec = b"PANASONIC_EH55_FEISTEL_VEC_12345"
    enc = feistel_encrypt(vec)
    assert enc != vec
    assert feistel_decrypt(enc) == vec

    assert adler32_panasonic(b"", step=16) == 1
    assert adler32_panasonic(b"\x41" * 32, step=16) == (131 | (197 << 16))

    # 1. Plaintext mode (DIGADIAG.FRM / PANA__SD.FRM FVU)
    fvu_payload = b"AM34_DIAG_FVU_IMAGE_" * 64
    packed_plain = pack_extrhead(fvu_payload, tag="FVU ", dst_addr=0x90400000, build_name="DIAG01", mode="plain")
    assert packed_plain.startswith(b"EXTRHEADFVU ")
    unpacked_plain = unpack_extrhead(packed_plain, mode="plain")
    assert unpacked_plain.decompressed == fvu_payload
    assert unpacked_plain.checksum_valid
    assert unpacked_plain.footer_present

    # 2. DRV prefix-only mode (> 0x2800 bytes compressed)
    drv_payload = _pseudo_random_bytes(16 * 1024, seed=0xCAFEBABE)
    packed_drv = pack_extrhead(drv_payload, tag="DRV ", dst_addr=0x40000000, build_name="S211", mode="drv")
    assert packed_drv[:8] != b"EXTRHEAD"
    assert b"EXTRFOOT" in packed_drv[0x2800:]
    unpacked_drv = unpack_extrhead(packed_drv, mode="auto")
    assert unpacked_drv.tag == "DRV "
    assert unpacked_drv.decompressed == drv_payload
    assert unpacked_drv.checksum_valid
    assert unpacked_drv.footer_present

    # 3. MAIN split mode (> 0x32000 bytes compressed so center zlib bytes stay plaintext)
    main_payload = _pseudo_random_bytes(240 * 1024, seed=0xDEAD5558)
    packed_main = pack_extrhead(main_payload, tag="MAIN", dst_addr=0x57000000, build_name="3410PQ", mode="main")
    assert len(packed_main) > 0x32000
    assert packed_main[:8] != b"EXTRHEAD"
    assert not packed_main.endswith(b"EXTRFOOT")
    unpacked_main = unpack_extrhead(packed_main, mode="auto")
    assert unpacked_main.tag == "MAIN"
    assert unpacked_main.build_name == "3410PQ"
    assert unpacked_main.decompressed == main_payload
    assert unpacked_main.checksum_valid
    assert unpacked_main.footer_present

    # 4. Real PANA_DVD.FRM TOC validation when present in workspace
    frm_path = Path("firmware_and_images/PANA_DVD.FRM")
    if frm_path.exists():
        with frm_path.open("rb") as fh:
            head_bytes = fh.read(0x200)
        toc = parse_decrypted_frm_toc(head_bytes, is_encrypted=True)
        assert [e.name for e in toc] == ["DRV S211", "PROG061 ", "MAIN3060NP"]
        assert [(e.module_offset, e.module_size) for e in toc] == [
            (0x00001800, 0x000B0000),
            (0x000B1800, 0x00040000),
            (0x000F1800, 0x0063D4E0),
        ]
        assert all(e.list_sum_valid for e in toc)
