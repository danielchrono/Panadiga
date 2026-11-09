# INDEPENDENT FORENSIC AUDIT OPINION & CRYPTANALYSIS DELIBERATION
**Target:** Optical Ecosystem (`PANA_DVD.FRM`) and MEIHDFS Storage (`hdd_100.bin`) — Panasonic DIGA DMR-EH55  
**Audited Single Source of Truth (SSOT):** `src/constants.rs`, `src/crypto/toc.rs`, `src/crypto/codebook.rs`, and `src/hdd/meihdfs.rs`.

---

## 0. Forensic Fact-Check & Evidence Rectification

| Audited Property | Physical Binary Reality (Audited SSOT) | Code Reference |
| :--- | :--- | :--- |
| **1. File Size of `PANA_DVD.FRM`** | **`7,531,744 bytes`** ($7.18\text{ MB}$ / $941,468$ blocks of 8 bytes). | `dump_toc` |
| **2. Global Header at `0x00000000`** | **100% encrypted from byte `0x00`:** `49 2F 05 60 CE DE 50 8C E3 C0 8A FD 49 B6 4A 9A`. | `PANA_FRM_GLOBAL_HEADER` |
| **3. TOC Geometry (`.FRM`)** | Offset **`0x0010..0x009F`** ($144\text{ bytes}$) with **3 periodic 48-byte entries** (`0x30`): AM34 OS, Optical Servo MN102, Front Panel MN103S. Active payload starts at `0x00A0`. | `PANA_FRM_TOC_ENTRY_SIZE` |
| **4. Header Entropy (`.FRM`)** | Local drops ($H \approx 2.8\text{--}3.0\text{ bits/byte}$) reflect repetitive **64-bit ECB padding** patterns. | `ECB_VIRGIN_FLASH_8B` |
| **5. Signature Offset (LBA 0)** | `0x00..0x03` = `00 00 00 00`, `0x04..0x05` = `01 00` (Rev 1.0), **`0x06..0x07` = `53 48` (`"SH"`)**, **`0x08..0x0F` = Volume GUID**. | `PANASONIC_HDD_MBR_FULL_SIGNATURE` |
| **6. State of Partitions 1 & 2** | Both containers begin with magic `51 87 13 8D 5B 0A FB 22` and **are encrypted in 64-bit ECB** using the identical key as `.FRM`. Descriptors at `0x00003000` are inverted (`XOR 0xFF`). | `PANASONIC_HDD_PARTITION_MAGIC` |

---

## 1. Cryptanalysis Validation & Padding Safeguards

1. **Structural Breakthrough vs. Full Key Recovery:** The ECB leak combined with KPA breaks optical container obfuscation but does not recover the PRP key across $2^{64}$ states. Complete plaintext is obtained by extracting the NOR Flash IPL routines (`0x80000000..0x807EFFFF`) or capturing decrypted payload from SDRAM (`0x40000000`) via DWire32A probe (`P7501`).
2. **Strict Prohibition of PKCS#7 Unpadding:** Matsushita firmware utilizes constant-block padding without PKCS#7. Slicing must strictly respect payload descriptors.

---

## 2. Two-Stage Boot Chain Invariant (`mkfs.meihdfs` and `U99` Prevention)

- Procedural `mkfs.meihdfs` accurately synthesizes LBA 0 (`"SH"`), bit-inverted descriptors LBA 24..31 (`XOR 0xFF`), journal, and mirror tables.
- **Stage 2 Invariant:** The NOR Flash IPL requires Partition 1 (`0x02000000`) to contain the bootable AM34 kernel. If unprovisioned, initial boot requires inserting the official service disc (`PANA_DVD.FRM`) into the optical drive to avoid the fatal `U99` error.

---

## 3. Atomic Dual-Mirror Disaster Recovery

- Accidental PC initialization (MBR/GPT overwriting LBAs 0..33) destroys both LBA 0 and LBA 24..31 descriptors.
- Full recovery requires atomic dual-mirror restoration:
  - Superblock MBR: `0x01C00000` (LBA 57344) $\longrightarrow$ `0x00000000` (LBA 0)
  - Descriptors Table: `0x01C03000` (LBA 57368) $\longrightarrow$ `0x00003000` (LBA 24..31)
- Preserves 100% of recorded video titles on Partition 3 with zero data loss.
