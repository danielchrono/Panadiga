# MEIHDFS STORAGE ARCHITECTURE & PARTITIONING SPECIFICATION
## Reverse Engineering the Hard Disk & Firmware Structure of Panasonic DMR-EH55
### Matsushita UniPhier / AM34 Platform — DIGA Chassis (PCB: VEP79138)

---

## 1. System Overview & Storage Engineering Scope

The **Panasonic DMR-EH55** digital DVD/HDD recorder (part of Matsushita Electric Industrial Co., Ltd.'s DIGA product line) utilizes a non-conventional hybrid storage architecture. Unlike generic PC platforms or commodity Linux-based DVRs, the DMR-EH55 is powered by the proprietary **Matsushita UniPhier MN2DS0016** system-on-chip (32-bit **AM34** RISC core) and delegates persistent storage across two complementary media:

1. **On-Board NOR Flash Memory (Spansion S29JL064H — 8 MBytes TSOP-48):**
   Contains the primary initial program loader (*IPL*), minimal hardware initialization routines, exception vectors, and non-volatile emulated calibration parameter blocks (`EEMUL` / NVRAM).
2. **Internal PATA/IDE Hard Disk Drive (80 GB to 500 GB):**
   Does not merely store user video recordings. The hard disk drive is an active, mandatory component of the boot subsystem: **it stores the operational executive firmware partitions (AM34 executive runtime), audio/video CODEC microcode, on-screen display (OSD) assets, peripheral microcodes, and the MEIHDFS filesystem metadata tables**.

If a blank or uninitialized hard disk drive is installed in a unit with a malfunctioning optical drive, the unit hangs during POST with front-panel error codes **`HDDERR`** or **`DRV NG`**, refusing to load the executive operating system or format the drive through user menus.

```text
                  PANASONIC DMR-EH55 HYBRID BOOT ARCHITECTURE
 ┌────────────────────────────────────────────────────────────────────────────────────────┐
 │                               MAIN BOARD (VEP79138)                                    │
 │                                                                                        │
 │   ┌──────────────────────────┐                  ┌──────────────────────────────────┐   │
 │   │  Matsushita MN2DS0016    │   Initial Boot   │      Spansion S29JL064H          │   │
 │   │  (UniPhier / AM34 Core)  │◄─────────────────│  (8 MB Flash NOR TSOP-48)        │   │
 │   │   108 MHz / 162 MHz      │   (Stage 1 IPL)  │  - Initial Program Loader (IPL)  │   │
 │   └────────────┬─────────────┘                  │  - Minimal Bootloader            │   │
 │                │                                │  - NVRAM / EEMUL (Calibrations)  │   │
 │                │ IDE / ATA Bus                  └──────────────────────────────────┘   │
 │                ▼                                                                       │
 │   ┌────────────────────────────────────────────────────────────────────────────────┐   │
 │   │                       HARD DISK DRIVE (PATA / IDE / SSD)                       │   │
 │   │                                                                                │   │
 │   │   [ LBA 0 ]                Panasonic Proprietary MBR ("SH", Disk GUID)         │   │
 │   │   [ LBA 24..31 ]           Low-Level Metadata & Allocation Table               │   │
 │   │   [ LBA 16,384..19,199 ]   Transactional Journal (10 x 128 KB Blocks)          │   │
 │   │   [ LBA 57,344 (28 MB) ]   Mirror MBR Table / Identical Backup to LBA 0        │   │
 │   │   [ LBA 65,536 (32 MB) ]   Firmware Partition 1 (0x5E3250 = ~5.89 MB, Stage 2) │   │
 │   │   [ LBA 131,072 (64 MB) ]  Firmware Partition 2 (0xB0000 = ~704 KB, OSD/Micro) │   │
 │   │   [ LBA 204,800+ ]         MEIHDFS Volume (VR_MOVIE.VRO / VR_MANGR.IFO)        │   │
 │   └────────────────────────────────────────────────────────────────────────────────┘   │
 └────────────────────────────────────────────────────────────────────────────────────────┘
```

### Forensic Reference Image File
* **Filename:** `firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin`
* **Exact Size:** `104,857,600` bytes (exactly 100.00 MB)
* **Total LBA Sectors:** `204,800` sectors of 512 bytes
* **Global MD5:** `ef4453fc8651903b0a4b0c1873d950d4`
* **Global SHA-256:** `9d292d5b46f190775f359e30af11d074021fb321f040bf701d0ecf0f2056db00`

---

## 2. Global Memory Map & Sector Layout (LBAs & Offsets)

Exhaustive sector-by-sector analysis of the 100 MB forensic image reveals the exact storage topography engineered by Matsushita. The disk layout partitions the storage into guard bands, transactional metadata zones, mirror redundancy, and executable code partitions:

| LBA Sector Range | Hexadecimal Offset Range | Size (Bytes) | Size (MB) | Primary Role / Technical Description |
| :--- | :--- | :--- | :--- | :--- |
| **LBA 0** | `0x00000000` – `0x000001FF` | 512 B | 0.0005 MB | **Panasonic Proprietary MBR:** Signature `"SH"`, version 1.0, and Disk GUID. |
| **LBA 1 .. 23** | `0x00000200` – `0x00002FFF` | 11,776 B | 0.0112 MB | **Guard Band 1:** Zero-filled (`0x00`) pre-metadata isolation sectors. |
| **LBA 24 .. 31** | `0x00003000` – `0x00003FFF` | 4,096 B | 0.0039 MB | **Descriptor Table:** 8 sectors containing 16-byte physical mapping records. |
| **LBA 32 .. 16,383** | `0x00004000` – `0x007FFFFF` | 8,372,224 B | 7.9844 MB | **Intermediate Guard Band:** Zero-padded alignment space up to 8 MB boundary. |
| **LBA 16,384** | `0x00800000` – `0x008001FF` | 512 B | 0.0005 MB | **Journal Catalog Header:** Block count index (`0x0A` = 10 blocks). |
| **LBA 16,385 .. 16,639** | `0x00800200` – `0x0081FFFF` | 130,560 B | 0.1245 MB | **Pre-Allocation Guard:** Boundary alignment buffer. |
| **LBA 16,640 .. 19,199** | `0x00820000` – `0x0095FFFF` | 1,310,720 B | 1.2500 MB | **Transactional Journal Area:** 10 discrete 128 KB blocks (256 sectors each). |
| **LBA 19,200 .. 49,151** | `0x00960000` – `0x017FFFFF` | 15,335,424 B | 14.625 MB | **Post-Journal Guard Band:** Zero-padded space up to 24 MB boundary. |
| **LBA 49,152** | `0x01800000` – `0x018001FF` | 512 B | 0.0005 MB | **24 MB Transition Marker:** Format status flag and volume geometry boundary. |
| **LBA 49,153 .. 57,343** | `0x01800200` – `0x01BFFFFF` | 4,193,792 B | 3.9995 MB | **Pre-Mirror Guard Band:** Zero-padded isolation space up to 28 MB boundary. |
| **LBA 57,344** | `0x01C00000` – `0x01C001FF` | 512 B | 0.0005 MB | **Mirror MBR Table:** Exact bit-for-bit duplicate of LBA 0 at 28.00 MB. |
| **LBA 65,536** | `0x02000000` – `0x025E324F` | 6,173,264 B | ~5.89 MB | **Firmware Partition 1 (OS/Core):** AM34 Executive, CODECs, and drivers. |
| **LBA 131,072** | `0x04000000` – `0x040AFFFF` | 720,896 B | ~0.70 MB | **Firmware Partition 2 (OSD/Micro):** Graphics assets, fonts, sub-CPU microcodes. |
| **LBA 204,800+** | `0x06400000` onwards | Variable | Variable | **MEIHDFS Audio/Video Volume:** User media recordings and DVD-VR data structures. |

---

## 3. Forensic Analysis of LBA 0 (Panasonic MBR)

The primary Master Boot Record at LBA 0 does not follow IBM-PC partition table standards (`0x55AA` signature at offset `0x1FE` is intentionally absent or repurposed):

* **Magic Bytes:** Offset `0x0000` contains ASCII `"SH"` (`0x53 0x48`), identifying the volume as a Panasonic Matsushita Storage Device.
* **Format Revision:** Offset `0x0004` specifies version `0x00010000` (Version 1.0).
* **Volume GUID:** Offset `0x0010` contains a 16-byte unique identifier paired with the optical DVD subsystem.
* **Allocation Bitmap Bit-Inversion (XOR 0xFF):** Panasonic applies bit-inversion (`0xFF` XOR masking) across allocation bitmaps to prevent standard desktop operating systems (Windows, macOS) from attempting destructive automatic file-system repairs or writing volume headers over firmware partitions.

---

## 4. Panadiga CLI Storage Commands

```bash
# Decode partition descriptors and LBA layout
panadiga hdd decode-descriptors "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"

# Extract binary firmware partitions to working directory
panadiga hdd extract --outdir firmware_and_images/extracted_partitions "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"

# Verify forensic integrity against SHA-256 audited manifest
panadiga hdd verify-hashes "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"
```
