# TECHNICAL INVENTORY, QUALITY AUDIT & BENCH OPERATIONS MANUAL
## Reverse Engineering & Firmware Recovery Ecosystem — Panasonic DMR-EH55

**Classification:** Systems Engineering Technical Document / Quality Assurance Audit  
**Target Hardware:** Panasonic DMR-EH55 (Matsushita `AM34` / `MN103S` on `VEP79138` Main PCB)  
**Audit Status:** Fully Certified & Approved  
**Version:** 1.0.0  

---

## 1. Executive Summary & Quality Assurance (QA) Certification

The engineering recovery and reverse engineering ecosystem for the **Panasonic DMR-EH55** digital DVD/HDD recorder has undergone rigorous quality assurance auditing, cryptographic artifact inventorying, and physical bench procedure certification. The primary objective is to provide electronics engineers and embedded firmware specialists with an exhaustive operational reference to permanently resolve critical startup failures—specifically the fatal **`U99`** parameter corruption error in Spansion NOR Flash, hard drive partition faults, and microcode corruption.

### QA Audit Deliverables & Results:
1. **Low-Level Firmware & Disk Image Repository:** All low-level binary artifacts (`panasonic DMR EH-55 Firmware hdd_100.bin`, `PANA_DVD.FRM`, and `UPDATE_EH55.zip`) have been forensically cataloged, entropy-analyzed, and fixed with cryptographic checksums (SHA-256 and MD5).
2. **Tooling & Build Automation:** The build/validation orchestrators (`scripts/build.py`, `scripts/validate.py`, typed with `mypy --strict`) and native Rust MCP server (`panadiga mcp`) adhere to strict typing and relative path resolution, eliminating machine-specific absolute path dependencies.
3. **Hexadecimal Diagnostic Tools:** Hex editor launcher scripts have been hardened for cross-platform and portable Windows execution.
4. **Service Manuals & Technical Schematics Archive:** 15 official PDF manuals have been categorized into Schematic Service Manuals, Regional Operating Guides, and Firmware Update Instructions.
5. **`P7501` Async UART Subsystem:** The `115200 8N1` asynchronous UART + discrete `SW_UART` / `XRESET` / `NSW5V` Hi-Z implementation (`firmware/pico-p7501/`) has been verified against board `VEP79138` schematics, enforcing `3.3V` LVTTL limits, Open-Drain `XRESET` control, and strict `Hi-Z Input` protection on `NSW5V` (`GP3`).

---

## 2. Consolidated Cryptographic Artifact Inventory

| Directory | Filename | Size (Bytes) | SHA-256 Hash | Technical Role / Description | QA Status |
| :--- | :--- | :---: | :--- | :--- | :---: |
| `firmware_and_images/` | `panasonic DMR EH-55 Firmware hdd_100.bin` | 104,857,600 | `9d292d5b46f190775f359e30af11d074021fb321f040bf701d0ecf0f2056db00` | Forensic dump of first 100 MB of factory HDD (MEIHFS, LBA 0, and firmware partitions). | Certified |
| `firmware_and_images/` | `PANA_DVD.FRM` | 7,531,744 | `a596d52a8220a9113777c9caf271ac4d518202d0639d6118a4d0bc03b9a6367f` | Official optical firmware update container (Multi-stage encrypted payload). | Certified |
| `firmware_and_images/` | `UPDATE_EH55.zip` | 7,474,766 | `07589e79722409413c5da25c700227a9a04cef39f0178ffa08df3d8d3f7debd4` | Official Matsushita distribution archive containing `PANA_DVD.FRM`. | Certified |
| `firmware/pico-p7501/` | `p7501_uart.pio` | 1,861 | `2d08a2621bb116bde5b23c06ee9879deddb85229d74ce5a0ffa82bc056412528` | RP2040/RP2350 PIO state machine for `115200 8N1` P7501 UART framing. | Certified |
| `firmware/pico-p7501/` | `p7501_pico_main.c` | 2,149 | `5c5970bfc940b3ef01e7d8f49d50ecf5b6ebcfbb0f499548569750f97f89b6fc` | C11 RP2040/RP2350 entrypoint orchestrating HAL, `P7501` UART capture, FMU/CDDIAG, and GDB RSP. | Certified |
| `firmware/pico-p7501/flash/` | `ram_loader_am34_target.c` | 6,715 | `b1238c0bafac7a28f16c54d41e3bd09bbf84a5141fe3f164091f43d7e3102940` | Position-independent AM34 machine code payload for JEDEC Flash verification/sanitization. | Certified |

---

## 3. Operational Bench Runbook: Curing the U99 Error

### Step 1: Pre-Flight Safety Verification
1. Ensure the Panasonic recorder is unplugged from AC mains.
2. Connect digital multimeter in continuity mode between Pin 4 of connector `P7501` and chassis metal. Verify $0.0\,\Omega$.
3. Ensure no voltage higher than $3.3\text{V}$ is applied to UART/control pins and `GP3` (`NSW5V` Pin 5) stays configured as `Hi-Z Input` without pulls.

### Step 2: In-Circuit `P7501` Boot Capture & Serial Diagnostics
1. Connect the Raspberry Pi Pico flashed with `panasonic_p7501_uart_unified.uf2` to connector `P7501`.
2. Plug the Pico USB cable into the bench workstation.
3. Run `PINCHECK` / `UARTPROBE` (`115200 8N1`) or `JIGBOOT` to release `XRESET` (`GP5` Open-Drain) and observe boot telemetry (`<00>DDR CHECK: OK`, `CD diag boot mode`).

### Step 3: NVRAM Sanitization (`IC5001` Flash Image / In-Circuit Workflow)
1. For offline Flash dumps (`S29JL064H` 8 MB `IC5001`), launch the Panadiga Desktop UI or run:
   ```bash
   panadiga nvram clean --input dump_ic5001.bin --output fixed.bin
   ```
2. Verify the top 64 KB NVRAM/EEMUL region (`0x807F0000..0x807FFFFF`, sectors `SA134..SA141`) is sanitized to virgin `0xFF` while `0x80000000..0x807EFFFF` (IPL) remains untouched.
3. Keep `XRESET` released (`Hi-Z` / `ISOLATE`) and verify clean system startup with verified SHA-256 manifests.
