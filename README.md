<p align="center">
  <strong>English</strong> | <a href="README.pt-BR.md">Português (Brasil)</a>
</p>

# Panadiga — Hardware Diagnostic, Firmware & Reverse Engineering Suite for Panasonic DIGA Recorders

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![SoC](<https://img.shields.io/badge/SoC-Matsushita%20UniPhier%20(AM34%2FMN103S)-blue.svg>)]()
[![Flash](<https://img.shields.io/badge/NOR%20Flash-Spansion%20S29JL%20%2F%20Am29LV%20(2MB--16MB)-green.svg>)]()
[![Storage](<https://img.shields.io/badge/Storage-MEIHDFS%20(IDE%2FPATA%2FSATA)-yellow.svg>)]()
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> **Master Technical Systems & Hardware Engineering Reference**
> **Classification:** Forensic Engineering / Unified Recovery & Tooling Platform
> **Primary Target Device:** Digital DVD/HDD Video Recorder **Panasonic DMR-EH55** (DIGA Chassis / Main Board `VEP79138`)
> **Ecosystem Compatibility:** Panasonic DIGA Lineup (**DMR-E, DMR-EH, DMR-ES, DMR-EX, DMR-BW Series**)
> **Core Architecture:** Matsushita UniPhier `MN2DS0016` / `MN2DS0018` / `MN103S` (32-bit RISC `AM34` / `MN10300`)
> **Flash Memory Range:** 2 MB (16 Mbit) to 16 MB (128 Mbit) TSOP-48 / TSOP-56 Parallel NOR Flash
> **Storage Subsystem:** PATA/IDE and SATA Hard Disk Drives with proprietary **MEIHDFS** partition tables
> **Hardware Interface:** Proprietary synchronous serial bus **Matsushita DWire32A / On-Chip Debug (OCD)** on connector `P7501` (`CN55001`)

---

## 1. Executive Project Overview

The **Panasonic DIGA** series represents the pinnacle of Japanese consumer electronics engineering from the 2001–2008 era. Designed by **Matsushita Electric Industrial Co., Ltd.** (now Panasonic Corporation), these devices integrate high-performance audio/video processing built upon the proprietary **UniPhier** System-on-Chip (SoC) family, executing the **Matsushita AM34** (MN10300 architecture) instruction set.

While the primary laboratory focus of this project is the widely deployed **DMR-EH55**, the engineering discoveries, algorithms, and tools within Panadiga span the **entire Panasonic DIGA recorder ecosystem**:

1. **Two-Stage Hybrid Bootloader Architecture:** The onboard Spansion NOR Flash (`IC55007`, 2 MB to 16 MB depending on generation) stores only the Initial Program Loader (*IPL*). The primary operating system, OSD graphics acceleration drivers, optical drive servo microcodes, and codecs reside entirely in reserved sectors within the first 100 MB of the internal hard drive.
2. **Proprietary MEIHDFS Storage System:** The internal hard drive does not use standard PC MBR partition tables or FAT/ext filesystems. The storage layout is structured around **MEIHDFS** (*Matsushita Electric Industrial HDD File System*), featuring an ASCII `"SH"` magic superblock at LBA 0, a redundant security mirror at LBA 57,344 (28 MB), a 10-block transactional journal, and encrypted firmware partitions.
3. **Cryptography & Intellectual Property Protection:** The optical distribution container (`PANA_DVD.FRM`) and the executable HDD partitions are protected with proprietary 64-bit symmetric block ciphers operating in ECB mode, tied to CPRM (*Content Protection for Recordable Media*) and DivX DRM engines.
4. **The Critical `U99` Error Vulnerability:** Channel presets, HDD volume mount states, and operating parameters are saved into the final 64 KB sector of the Flash memory, which emulates EEPROM wear-leveling (`EEMUL`). Cell degradation, unexpected power loss during disk write operations, or corrupted status words trigger startup validation panics. Consequently, the Matsushita kernel halts permanently with the fatal code **`U99`** on the front VFD vacuum fluorescent display, ignoring remote commands and deadlocking the optical tray mechanism.

This repository establishes the **definitive forensic reverse engineering, physical unbricking, and laboratory diagnosis ecosystem** for the Panasonic DIGA lineup. It provides the high-performance Rust toolchain **`panadiga`**, a zero-latency desktop and mobile Android (`.apk`) bench GUI, C11/PIO firmware for **Raspberry Pi Pico (RP2040) and Pico 2 (RP2350)** with **Bluetooth Low Energy (BLE)** and USB-C OTG support, 30 cataloged factory service manuals, schematics and silicon datasheets, and 13 comprehensive bilingual engineering reference dossiers.

---

## 2. End-to-End System Architecture

```mermaid
%%{init: {
  'theme': 'base',
  'themeVariables': {
    'background': '#161b22',
    'primaryColor': '#21262d',
    'primaryTextColor': '#f0f6fc',
    'primaryBorderColor': '#30363d',
    'lineColor': '#8b949e',
    'secondaryColor': '#21262d',
    'tertiaryColor': '#21262d',
    'clusterBkg': '#161b22',
    'clusterBorder': '#30363d',
    'defaultLinkColor': '#8b949e',
    'titleColor': '#f0f6fc',
    'edgeLabelBackground': '#161b22'
  }
}}%%
flowchart TD
    subgraph LAB_BENCH["1. LABORATORY BENCH AND COMMUNICATIONS"]
        direction LR
        USER["Forensic Engineer / Operator"]
        BLE["Bluetooth Low Energy (NUS) / USB-C OTG"]
        PICO["Raspberry Pi Pico (RP2040 / RP2350)<br/>C11 Firmware + DWire32A PIO"]
        USER <--> BLE <--> PICO
    end

    subgraph SOFTWARE_SUITE["2. PANADIGA SOFTWARE ECOSYSTEM"]
        direction TB
        GUI["Panadiga Desktop / Android GUI<br/>(Tauri 2.0 + React 19 + TypeScript)"]
        CLI["Unified Core Engine<br/>(Native Rust 1.85+ Binary)"]
        GUI <--> CLI

        RAG["Zero-Error RAG Engine<br/>(6 Invariant Shields)"]
        DW_HAL["DWire32A HAL<br/>(115200 / PIO Sync)"]
        GDB_SRV["GDB RSP Server (:3333)<br/>(Register Cache)"]
        MEIHDFS["MEIHDFS Parser<br/>(Storage Forensics)"]
        CRYPTO["Cryptanalysis Engine<br/>(ECB 64-bit & KPA)"]
        NVRAM["NVRAM Sanitizer<br/>(Safe 0xFF Clean)"]

        CLI <--> RAG
        CLI <--> DW_HAL
        CLI <--> GDB_SRV
        CLI <--> MEIHDFS
        CLI <--> CRYPTO
        CLI <--> NVRAM
    end

    subgraph TARGET_HARDWARE["3. PANASONIC DIGA TARGET HARDWARE"]
        direction LR
        P7501["Port P7501 (JST ZR-6)<br/>Pins: VDD, nRESET, SDATA, GND, SCLK, EXTRG"]
        AM34["Matsushita UniPhier MN2DS0016<br/>32-bit AM34 RISC CPU"]
        FLASH["Parallel NOR Flash (2MB -- 16MB)<br/>IPL + 64 KB NVRAM (len - 64KB)"]
        HDD["PATA/IDE or SATA Hard Disk<br/>LBA 0 Superblock (SH) + Partitions"]
        P7501 <--> AM34
        AM34 <--> FLASH
        AM34 <--> HDD
    end

    PICO <-->|"DWire32A Bus 1.0 MHz"| P7501
    BLE <--> GUI

    style LAB_BENCH fill:#161b22,stroke:#30363d,stroke-width:1.5px,color:#f0f6fc
    style SOFTWARE_SUITE fill:#161b22,stroke:#30363d,stroke-width:1.5px,color:#f0f6fc
    style TARGET_HARDWARE fill:#161b22,stroke:#30363d,stroke-width:1.5px,color:#f0f6fc

    classDef greyBox fill:#21262d,stroke:#30363d,color:#f0f6fc,stroke-width:1.5px;
    class USER,BLE,PICO,GUI,CLI,RAG,DW_HAL,GDB_SRV,MEIHDFS,CRYPTO,NVRAM,P7501,AM34,FLASH,HDD greyBox;
```

---

## 3. Repository File Structure & Semantic Mapping

```text
Panadiga/
├── run.cmd                                           <-- Fast Desktop GUI & CLI Launcher (Windows)
├── run.sh                                            <-- Fast Desktop GUI & CLI Launcher (Linux/macOS)
├── build.cmd                                         <-- Master Parallel Build Pipeline (Windows)
├── build.sh                                          <-- Master Parallel Build Pipeline (Linux/macOS)
├── validate_all.cmd                                  <-- 9 Quality Gates Execution Script (CMD)
├── validate_all.ps1                                  <-- 9 Quality Gates Execution Script (PowerShell)
├── build-android.cmd                                 <-- Android APK Packaging Pipeline (Windows)
├── build-android.sh                                  <-- Android APK Packaging Pipeline (Unix)
├── Cargo.toml                                        <-- Master Rust Workspace Specification (2024 Edition)
├── tauri.conf.json                                   <-- Tauri 2.0 Desktop & Mobile Configuration
├── src/                                              <-- High-Performance Native Rust Core Engine
│   ├── main.rs                                       <-- Unified Entrypoint (CLI Dispatcher & Tauri GUI)
│   ├── lib.rs                                        <-- Core Library Crate Exporting All Engines
│   ├── logger.rs                                     <-- Unified Multi-Sink Structured Logger
│   ├── manuals.rs                                    <-- Service Manuals Indexer & Fast Regex Scanner
│   ├── nvram/                                        <-- Multi-Geometry NVRAM & EEPROM Emulation Parser
│   ├── hdd/                                          <-- MEIHDFS Storage & Partition Table Engine
│   ├── frm/                                          <-- PANA_DVD.FRM Container Cryptanalysis Engine
│   ├── rag/                                          <-- Zero-Error RAG Engine & 6 Invariant Shields
│   └── desktop/                                      <-- Tauri 2.0 Native Rust Commands & GDB Bridge
├── apps/desktop/                                     <-- Zero-Latency Modern Desktop UI (React 19 + Vite)
│   ├── src/                                          <-- Strict MVVM Components & ViewModels
│   │   ├── components/                               <-- Pure UI Panels (GDB, DWire32A, HexViewer)
│   │   ├── services/                                 <-- Typed Tauri IPC Transport & Native Bridges
│   │   └── viewmodels/                               <-- Algebraic State Machines (Pure FP Logic)
│   ├── package.json                                  <-- Node.js Dependencies & Build Scripts
│   └── playwright.config.ts                          <-- End-to-End Browser Testing Configuration
├── docs/                                             <-- Official Technical Documentation Repository
│   ├── INDEX.md                                      <-- Master Documentation Index & Gateway Portal
│   ├── en-US/                                        <-- Canonical English Technical Dossiers (13 Docs)
│   ├── pt-BR/                                        <-- Native Portuguese Technical Dossiers (13 Docs)
│   └── images/                                       <-- High-Resolution Schematics, Photos & Pinouts
├── manuals_and_schematics/                           <-- Technical Library (30 Manuals, Schematics & Datasheets)
│   ├── README.md                                     <-- Master Catalog Indexed by Model & Subsystem
│   ├── datasheets/                                   <-- Silicon Datasheets (MN103, RP2040, RP2350, Spansion)
│   ├── dmr-eh55/                                     <-- DMR-EH55 Manuals (Service, Schematics, User, Firmware)
│   ├── dmr-eh56/                                     <-- DMR-EH56 European Operating Instructions
│   ├── dmr-eh50/                                     <-- DMR-EH50 User & Operations Manual
│   ├── dmr-eh65/                                     <-- DMR-EH65 High-Capacity HDD User Manual
│   ├── dmr-eh80v/                                    <-- DMR-EH80V VHS+DVD+HDD Service Manual & Schematics
│   ├── dmr-es10/                                     <-- DMR-ES10 Service Manual (U99 Code & Schematics)
│   ├── dmr-es30v/                                    <-- DMR-ES30V Combo VHS+DVD User Manual
│   ├── dmr-e55/                                      <-- DMR-E55 2MB Flash Optical User Manual
│   ├── dmr-e85h/                                     <-- DMR-E85H 4MB Flash Pioneer HDD/DVD Manual
│   ├── dmr-e500h/                                    <-- DMR-E500H Massive Storage HDD User Manual
│   └── dmr-ex98v/                                    <-- DMR-EX98V DVB-T/HDMI Service Manual Vol 1
├── tools/                                            <-- Lab Automation, Ghidra Scripts & Utilities
│   ├── Abrir_HxD.cmd                                 <-- Fast HxD Hex Editor Benchmark Launcher
│   ├── conectar_gdb_multiarch.cmd                    <-- GDB Multiarch Target Connection Script
│   ├── executar_openocd.cmd                          <-- OpenOCD Daemon Launcher for AM34
│   ├── iniciar_ponte_gdb.cmd                         <-- GDB RSP Server TCP Forwarder (:3333)
│   ├── panadiga_cli.py                               <-- Python Typer Forensic Companion CLI
│   ├── panadiga_mcp_server.py                        <-- Model Context Protocol (MCP) Lab Server
│   └── ghidra_am34_memory_map.py                     <-- Ghidra AM34 Memory Map & Symbol Loader
└── dwire32a_jtag_research/                           <-- RP2040/RP2350 C11 Firmware & PIO State Machines
    ├── dwire32a.pio                                  <-- 1.0 MHz Half-Duplex PIO Synchronous Engine
    ├── dwire32a_hw.c                                 <-- Hardware Driver & Timing Control (100% C11)
    ├── gdb_rsp.c                                     <-- Deterministic GDB RSP Protocol Parser
    └── build_pico_firmware.cmd                       <-- RP2040/RP2350 Bare-Metal Compilation Pipeline
```

---

### 3.1. Unified Cross-Platform Build & Launcher Scripts

| Script                          | Platform      | Mode                     | Description                                                                                                                                                                                                   |
| :------------------------------ | :------------ | :----------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **`run.cmd`**           | Windows       | Fast Launch / Auto-Build | Launches the Desktop GUI directly from `dist/` or `target/release/`. If uncompiled, invokes `build.cmd` first. Supports `--dev`, `--cli`, `--validate`.                                           |
| **`run.sh`**            | Linux / macOS | Fast Launch / Auto-Build | Unix equivalent of `run.cmd`. Automatically verifies binaries, builds missing artifacts, and executes GUI or CLI.                                                                                           |
| **`build.cmd`**         | Windows       | Full Parallel Release    | Builds React bundle, exports Ghidra SSOT scripts, compiles native Rust release binary (`%NUMBER_OF_PROCESSORS%` threads), builds Pico firmware, and packages `dist/releases/panadiga-windows-x86_64.zip`. |
| **`build.sh`**          | Linux / macOS | Full Parallel Release    | Complete parallel Unix release pipeline with packaging and checksum generation.                                                                                                                               |
| **`validate_all.cmd`**  | Windows       | 9 Quality Gates          | Executes the 9 quality gates through native CMD dispatching.                                                                                                                                                  |
| **`validate_all.ps1`**  | Windows       | 9 Quality Gates          | Comprehensive PowerShell validation runner with colorized step-by-step reporting and zero regression tolerance.                                                                                               |
| **`build-android.cmd`** | Windows       | Mobile Android APK       | Compiles the Android APK target (`aarch64-linux-android`) via Tauri 2 Mobile with BLE and USB-C OTG support.                                                                                                |
| **`build-android.sh`**  | Linux / macOS | Mobile Android APK       | Unix version of the Android compilation pipeline.                                                                                                                                                             |

---

## 4. Complete Native Rust CLI Guide (`panadiga`)

The native binary `panadiga` functions as both the Tauri graphical desktop client and a high-performance command-line utility.

### 4.1. Release Compilation

```bash
# Build the native binary with maximum optimizations
cargo build --release -p panadiga

# Directly run via unified wrapper:
./run.cmd --help       # Windows
./run.sh --help        # Linux / macOS
```

### 4.2. Command 1: Multi-Geometry NVRAM Sanitation (`nvram clean`)

Wipes the corrupted wear-leveling state (`EEMUL`) within the NOR Flash without modifying the bootloader (*IPL*). Automatically detects dump size (2 MB, 4 MB, 8 MB, 16 MB) and calculates the exact 64 KB sector at `len - 64 KB`.

```bash
# 8 MB Dump (DMR-EH50 / EH55 / EH56 / EH60 / EH65 / EH80V):
panadiga nvram clean --input flash_dump_8mb.bin --output flash_recovered.bin

# 4 MB Dump (DMR-E85H / E95H / ES20):
panadiga nvram clean --input flash_dump_4mb.bin --output flash_recovered.bin

# 2 MB Dump (DMR-E50 / E55 / ES10):
panadiga nvram clean --input flash_dump_2mb.bin --output flash_recovered.bin

# 16 MB Dump (DMR-EX75 / EX85 / EX98V):
panadiga nvram clean --input flash_dump_16mb.bin --output flash_recovered.bin
```

* **Sanitation Byte:** Erased state `0xFF` (canonical virgin state of NOR flash sectors).
* **Silicon Invariant:** Never touches offsets `0x00000000` to `len - 65537`, preserving the factory IPL.

### 4.3. Command 2: Structural Hard Drive Inspection (`hdd inspect`)

Parses the MEIHDFS partition layout, locates the ASCII `"SH"` superblock, verifies the 28 MB backup mirror, and outputs partition boundaries.

```bash
panadiga hdd inspect --input hdd_image.bin
```

### 4.4. Command 3: Surgical Partition Extraction (`hdd extract`)

Extracts raw firmware partitions from the MEIHDFS drive image for reverse engineering in Ghidra or IDA Pro.

```bash
panadiga hdd extract --input hdd_image.bin --output-dir partitions/
```

### 4.5. Command 4: Optical Container Cryptanalysis (`frm analyze`)

Validates the optical update container `PANA_DVD.FRM`, checking block entropy, TOC records, and electronic codebook (ECB) leakage patterns.

```bash
panadiga frm analyze --input firmware_and_images/PANA_DVD.FRM
```

### 4.6. Command 5: MEIHDFS Bit-Inverted Descriptor Decoding (`hdd decode-descriptors`)

Decodes the bit-inverted (One's Complement / XOR 0xFF) partition descriptors found in the MEIHDFS superblock.

```bash
panadiga hdd decode-descriptors --input hdd_100.bin
```

### 4.7. Command 6: Container Deconstruction & TOC Extraction (`frm dump-toc`)

Splits `PANA_DVD.FRM` into its three functional payloads: Main CPU OS (AM34), Optical Drive Servo DSP (MN102), and OSD Graphics Engine.

```bash
panadiga frm dump-toc --input firmware_and_images/PANA_DVD.FRM
```

### 4.8. Command 7: 64-Bit ECB Codebook Audit (`frm codebook`)

Extracts high-frequency 64-bit blocks across the optical container and HDD firmware partitions to establish cryptographic equivalence.

```bash
panadiga frm codebook --frm firmware_and_images/PANA_DVD.FRM --hdd firmware_and_images/hdd_100.bin
```

### 4.9. Command 8: Critical Firmware Headers Inspection (`headers`)

```bash
panadiga headers --input firmware_and_images/hdd_100.bin
```

### 4.10. Command 9: Forensic String & Keyword Scanner (`scan`)

```bash
panadiga scan --input firmware_and_images/PANA_DVD.FRM --terms U99,FIRMWARE,PANASONIC
```

### 4.11. Command 10: Service Manuals Indexer & Fast Search (`manuals`)

Searches the 30 official Panasonic service manuals, schematics, and silicon datasheets using fast regex matching across subdirectories.

```bash
panadiga manuals --query "P7501"
panadiga manuals --query "U99" --filter "dmr-es10"
panadiga manuals --query "MN103" --filter "datasheets"
```

### 4.12. Command 11: Zero-Error RAG Engine Query (`rag query`)

Queries the local retrieval-augmented generation engine, enforcing the 6 physical invariant shields.

```bash
panadiga rag query --question "Qual o pino de terra canônico do conector P7501?"
panadiga rag query --question "What is the canonical ground pin of P7501?" --json
```

---

## 5. Electrical Specification & DWire32A Interface (`P7501`)

### 5.1. Canonical Pinout, Bench Wiring & Universal RP2040 Connection

On the digital board `VEP79138` (and related `VEP79xxx` family boards across the DMR-EH series), the debugging header (labeled on the official Panasonic schematic as **`P7501 [B6B-ZR-3R4]`**, designated *"TO WRITE JIGU"*) is physically located along the top edge of the board, positioned between the 10-conductor flat flex cable (`FP50003`) on the left and the 8-conductor flat flex cable (`FP50004`) on the right.

> [!CAUTION]
> **ELECTRICAL SAFETY INVARIANT — MANDATORY CHASSIS GROUND (`GND`) VALIDATION:**
>
> * **According to the official Panasonic Service Manual (Connector P7501 Schematic):** **`GND` IS STRICTLY PIN 4**.
> * **Pin 1 (VDD Sense) is positioned adjacent to the 8-conductor flat cable (`FP50004`).**
> * **Pin 6 (EXTRG) is positioned adjacent to the 10-conductor flat cable (`FP50003`), marked by a square pad on the PCB silkscreen.**
> * **NEVER connect ground to Pin 1 or Pin 6** (which would short the 3.3V power plane or clamp the CPU interrupt trigger gate). Before powering the Raspberry Pi Pico or any bench instrument, **always verify with a digital multimeter a continuous $0.0\,\Omega$ resistance beep between Pin 4 and the bare metal chassis!**

<p align="center">
  <img src="docs/images/panasonic_dmr_eh55_pcb_p7501_orientation.png" alt="Physical Topography and P7501 Orientation on PCB VEP79138" width="720" />
  <br/>
  <em>Figure 5.1: Physical topography of the digital PCB VEP79138 showing connector P7501 between flat cables FP50003 (10-pin) and FP50004 (8-pin).</em>
</p>

```text
       CANONICAL MECHANICAL ORIENTATION OF CONNECTOR P7501 [B6B-ZR-3R4] (PANASONIC SCHEMATIC)
       Rear wall facing board interior (top) | Locking key window facing front edge (bottom)
   
       10-Pin Flat Cable (FP50003)                                         8-Pin Flat Cable (FP50004)
       [On the Left]                                                                     [On the Right]
            ┌────────────────────────────────────────────────────────────────────────┐
      Pin 6 │ ■ EXTRG   [Square Pad]   External OCD Break / Halt Trigger             │
      Pin 5 │ ● SCLK    [Round Pad]    Synchronous Clock DWire32A (Master 1.0 MHz)   │
      Pin 4 │ ● GND     [Round Pad]    Canonical Chassis Ground (0.0Ω Continuous!)   │
      Pin 3 │ ● SDATA   [Round Pad]    Half-Duplex Data (Pull-up 4.7kΩ to 3.3V)      │
      Pin 2 │ ● nRESET  [Round Pad]    Master Reset Control (Strict Open-Drain)      │
      Pin 1 │ ● VDD     [Round Pad]    Target Power Sense (+3.3V Reference)          │
            └────────────────────────────────────────────────────────────────────────┘
```

<p align="center">
  <img src="docs/images/schematic_p7501_panasonic_official.png" alt="Official Panasonic DMR-EH55 P7501 Schematic" width="460" />
  <br/>
  <em>Figure 5.2: Excerpt from official Panasonic DMR-EH55 schematic (VEP79138) detailing header P7501 [B6B-ZR-3R4] "TO WRITE JIGU".</em>
</p>

> [!NOTE]
> **RP2040 / RP2350 Hardware Compatibility:** Any board powered by an **RP2040** (Pico, Pico W) or **RP2350** (Pico 2) is 100% compatible. Nanosecond bus timing for DWire32A is handled directly in silicon via **PIO0** state machines.

| P7501 Pin (Canonical) | Matsushita Signal |     RP2040 / Pico Pin     |                 Conditioning Component                 |         Signal Type         | Bench Function & Safety Directive                                                                                                        |
| :-------------------: | :---------------: | :------------------------: | :-----------------------------------------------------: | :-------------------------: | :--------------------------------------------------------------------------------------------------------------------------------------- |
|    **Pin 1**    |   `VDD Sense`   |  `GP26 / ADC0` (Pin 31)  |    $1.0	ext{ k}\Omega$ Series Resistor (Optional)    |        Analog Input        | Senses the presence of $+3.3	ext{V}$ from the target board. **Do NOT power the Pico from this pin ($I < 10	ext{ mA}$).**       |
|    **Pin 2**    |    `nRESET`    |      `GP5` (Pin 7)      |             $100\,\Omega$ Series Resistor             | **Strict Open-Drain** | Master CPU reset line (`XRST` / `D_RESET`). **Drain to 0V to assert reset; float to Hi-Z to run. Never drive 3.3V Push-Pull!** |
|    **Pin 3**    |     `SDATA`     |      `GP4` (Pin 6)      | $100\,\Omega$ + $4.7	ext{ k}\Omega$ Pull-up to 3.3V |     Bidirectional Sync     | Half-duplex DWire32A data line. Pull-up ensures rapid rise time on extended test leads.                                                  |
|    **Pin 4**    | **`GND`** | **Pin 38 (`GND`)** |           **Direct low-impedance bond**           |   **0V Reference**   | **Mandatory equipotential bond: verify $0.0\,\Omega$ continuity to bare metal recorder chassis!**                                |
|    **Pin 5**    |     `SCLK`     |      `GP3` (Pin 5)      |             $100\,\Omega$ Series Resistor             |     Master Clock Output     | Synchronous serial master clock generated by Pico PIO (Operating frequency: $1.0	ext{ MHz}$).                                          |
|    **Pin 6**    |     `EXTRG`     |      `GP2` (Pin 4)      |             $100\,\Omega$ Series Resistor             |   Digital Trigger Output   | External trigger forcing CPU execution halt (*Halt Request*) on the AM34 core during Cold Break-In.                                    |

<p align="center">
  <img src="docs/images/dwire32a_pico_probe_assembly_schematic.png" alt="Standardized Electrical Schematic of DWire32A Pico Probe" width="760" />
  <br/>
  <em>Figure 5.3: Standardized electrical schematic (IEEE Std 315 / IEC 60617) for building the laboratory bench probe linking Raspberry Pi Pico to header P7501.</em>
</p>

---

### 5.2. Critical Engineering Rules & Damage Prevention

1. **Rule 1 (Strict 3.3V LVCMOS Logic):** The MN2DS0016 SoC pins tolerate a maximum absolute rating of $+3.6	ext{V}$. Connecting 5.0V TTL probes (such as classic CH341A or standard Arduino AVR) directly destroys the I/O pads.
2. **Rule 2 (Open-Drain Emulation on `nRESET`):** Driving `nRESET` with active 3.3V push-pull logic burns out the onboard reset supervisor circuit (`IC55005`). Driving to ground resets the system; floating to Hi-Z allows operation.
3. **Rule 3 (Mandatory Ground Verification):** Ensure $0.0\,\Omega$ continuity between Pin 4 and chassis ground before mating any other signals.
4. **Rule 4 (No Desoldering Required):** Connect directly to header P7501 using a 1.5mm pitch JST ZR female connector. Removing the TSOP-48 flash chip from the multilayer PCB carries high delamination risks.
5. **Rule 5 (Cold Break-In Sequence):** To gain CPU control before the IPL watchdog expires, assert `EXTRG` high and pulse `nRESET` low for $10	ext{ ms}$, entering OCD debug mode at clock cycle zero.

---

### 5.3. DWire32A Timing & Signal Diagram

```text
Clock Cycle:       1     2     3     4     5     6     7     8
                 _____       _____       _____       _____
SCLK   _________/     \_____/     \_____/     \_____/     \_____  (1.0 MHz Master)
   
SDATA  ===< CMD[7] >< CMD[6] >< CMD[5] >...========[ Hi-Z Turnaround ]===> DATA[7] >
                 ^           ^
                 |           |-- Data sampled on SCLK rising edge
                 |-------------- Data driven on SCLK falling edge

Timing Constraints:
  * t_CLK_PERIOD = 1000 ns (1.0 MHz)
  * t_SETUP      = 250 ns minimum before SCLK rising edge
  * t_HOLD       = 250 ns minimum after SCLK rising edge
  * t_TURNAROUND = 2 full SCLK cycles (2000 ns) for bus direction handoff
```

---

## 6. Physical Memory Map of Matsushita AM34 (`MN2DS0016`)

```text
+-------------------+-------------------+----------------------+--------------------------------------------+
| Physical Range    | Mirrored Range    | Bus Type / Target    | Functional Description                     |
+-------------------+-------------------+----------------------+--------------------------------------------+
| 0x00000000..0FFF  | 0x40000000..0FFF  | On-Chip I-RAM/SRAM   | 4 KB Core Scratchpad & Exception Vectors   |
| 0x00400000..07FF  | 0x40400000..07FF  | On-Chip D-RAM        | 64 KB High-Speed System Data Memory        |
| 0x04000000..07FF  | 0x44000000..07FF  | DDR-SDRAM Controller | 64 MB Main System Memory (DDR 333/400)     |
| 0x08000000..0FFF  | 0x48000000..0FFF  | MPEG / OSD Frame     | 32 MB Dedicated Framebuffer / Video Plane  |
| 0x10000000..10FF  | 0x50000000..10FF  | IDE / PATA Master    | PATA ATA/ATAPI-5 Host Controller (HDD/DVD) |
| 0x18000000..187F  | 0x58000000..187F  | External CS0 (NOR)   | Spansion S29JL064H Flash (8 MB - IPL)      |
| 0x187F0000..187F  | 0x587F0000..187F  | External CS0 (NV)    | 64 KB NVRAM / EEMUL Sector (U99 Target)    |
| 0x1F000000..1FFF  | 0x5F000000..1FFF  | Internal MMIO/SoC    | Peripheral Registers, Timers & OCD Module  |
+-------------------+-------------------+----------------------+--------------------------------------------+
```

### 6.1. JEDEC Golden Rule for Injected RAM Loader

When loading executable payloads directly into DDR-SDRAM via DWire32A OCD commands:

* **Target Load Address:** `0x04010000` (safe DDR buffer offset avoiding zero-page exception vectors).
* **Stack Pointer Initializer:** `SP = 0x0040FFF0` (within high-speed on-chip SRAM).
* **Watchdog Disable:** Write `0x00000000` to MMIO register `0x1F000104` within $100	ext{ ms}$ of cold boot.

---

## 7. Forensic Cryptanalysis & Storage Architecture Deep-Dive

### 7.1. Electronic Codebook (ECB) Leakage

Statistical analysis of both `PANA_DVD.FRM` and raw HDD dumps reveals that Matsushita implemented 64-bit symmetric block encryption without Cipher Block Chaining (CBC). Consecutive identical 8-byte blocks in plaintext yield identical 8-byte ciphertext blocks, exposing memory tables and structures.

### 7.2. 64-Bit ECB Codebook & Known-Plaintext Attack (KPA)

| Ciphertext Block (Hex) |      Plaintext Meaning      | FRM Occurrences | HDD Occurrences | Forensic Significance                        |
| :--------------------: | :-------------------------: | :-------------: | :-------------: | :------------------------------------------- |
|  `035065C87BB8E090`  | `FF FF FF FF FF FF FF FF` |      3,812      |      1,280      | Virgin NOR Flash erased cells / zero-padding |
|  `E5D1F34A518B2690`  | `00 00 00 00 00 00 00 00` |       196       |        3        | Null padding / Zero blocks                   |
|  `4D3C5DBB0C357777`  |   Partition Table Header   |      1,280      |        0        | Internal Module Descriptor Frame             |
|  `EFDC1CDA13881BAB`  |   Processor Family Header   |        3        |        0        | TOC: Matsushita UniPhier SoC Family Marker   |
|  `74A78ACD789B84FF`  |      CPU Core Selector      |        2        |        0        | TOC: AM34 / MN103S Core Identifier           |
|  `3B7E7EDBB6C57A53`  |      DSP Core Selector      |        1        |        0        | TOC: MN102 Optical Drive Servo DSP           |

### 7.3. Optical Container Structure (`PANA_DVD.FRM`)

The update disc packaging decomposes cleanly into three discrete modules preceded by a 16-byte container header (`492F0560CEDE508CE3C08AFD49B64A9A`) and a 144-byte TOC (three 48-byte records):

1. **Module 0 (`MOD_0`):** Main OS Kernel for the AM34 processor (operating system, GUI, filesystem drivers).
2. **Module 1 (`MOD_1`):** Optical drive servo microcode for the MN102 DSP.
3. **Module 2 (`MOD_2`):** OSD assets, font tables, graphics primitives, and multi-language strings.

### 7.4. Irrefutable Cryptographic Link to the Internal HDD

The presence of identical cipher blocks `E5D1F34A518B2690` (zeros) and `035065C87BB8E090` (erased flash) in both the optical `.FRM` container and the hard drive partitions proves conclusively that:

1. The firmware stored on the hard drive is encrypted using the **same algorithm and key** as the optical update media.
2. The internal HDD cannot be cloned or repaired with arbitrary unencrypted data; it requires the correct Panasonic partition structure and decryption keys.

### 7.5. Bit-Inverted (One's Complement / XOR 0xFF) MEIHDFS Architecture

The LBA 0 superblock stores partition geometry inverted with bitwise NOT (`XOR 0xFF`):

* ASCII magic `"SH"` appears in raw disk hex as `0xAE 0xB7` (`~0x51 0x48`).
* LBA offset counters and cluster sizes are similarly inverted, preventing standard partition managers from identifying or corrupting the volume.

---

## 8. Practical U99 Resolution Guide (3 Engineering Routes)

```text
                   +---------------------------------------------+
                   |       PANASONIC DMR-EH55 U99 ERROR         |
                   |       (Permanent VFD Display Panic)         |
                   +----------------------+----------------------+
                                          |
                +-------------------------+-------------------------+
                |                         |                         |
                v                         v                         v
     [ROUTE A: HARDWARE LAB]   [ROUTE B: IN-CIRCUIT OCD]   [ROUTE C: FORENSIC CLONE]
     Desolder TSOP-48 Flash    Connect RP2040 to P7501     Direct raw clone of HDD
     Program via T48 / SVOD4   Run DWire32A RAM Loader     Zero sector 0x7F0000
     Direct Clean 0x7F0000     Surgical In-Memory Wipe     Preserve 100 MB System Area
```

### Route A: External Flash Programmer (TSOP-48 via T48 / SVOD4)

* Desolder `IC55007` (Spansion S29JL064H70TFI00).
* Read full 8 MB dump and save backup.
* Use `panadiga nvram clean --input dump.bin --output fixed.bin`.
* Flash `fixed.bin` back to chip and resolder.

### Route B: In-Circuit via DWire32A OCD (Raspberry Pi Pico + RP2040)

* Connect 6-wire probe from Pico to header `P7501` (confirm Pin 4 GND).
* Run `panadiga pico flash --unbrick`.
* Pico halts AM34 core, injects RAM loader, and resets NVRAM sector to `0xFF`.

### Route C: Hard Drive Cloning & Sector Sanitization (Preserving MEIHDFS)

* When U99 is caused by corrupted HDD system state, connect drive to bench PC.
* Clone raw 100 MB system area: `panadiga hdd extract --input /dev/sdb --output-dir backup/`.
* Write clean partition descriptors and remount in recorder.

---

## 9. Comprehensive Panasonic DIGA Lineup Compatibility & Architectural Comparison Matrix

The following matrix formally specifies the architecture, silicon variants, Flash geometries, NVRAM locations, and unbricking compatibility across the 5 generations of the Panasonic DIGA lineup:

| Device Lineup & Models                                                                                          | Generation / Era                             | SoC Processor & Architecture                                              | NOR Flash Size & Chip                                   | NVRAM / EEMUL Offset Range              | Storage System & Interface                     | DWire32A / OCD Debug Header                                                              | U99 Root Cause & Panadiga Unbrick Method                                                                                |
| :-------------------------------------------------------------------------------------------------------------- | :------------------------------------------- | :------------------------------------------------------------------------ | :------------------------------------------------------ | :-------------------------------------- | :--------------------------------------------- | :--------------------------------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------- |
| **DMR-E20, DMR-E30, DMR-E50, DMR-E55, DMR-E60**                                                           | **Gen 1** (2001–2003)                 | Matsushita MN10300 (16/32-bit AM32 / AM33)                                | **2 MB** (16 Mbit) TSOP-48 (Am29LV160)            | `0x001F0000..0x001FFFFF` (Last 64 KB) | Optical DVD-RAM / DVD-R (Direct UDF 1.5)       | 5-pin / 6-pin OCD connector (3.3V)                                                       | Flash wear-leveling corruption. Fixed via `panadiga nvram clean` (auto-detected 2 MB).                                |
| **DMR-E80H, DMR-E85H, DMR-E95H, DMR-E100H, DMR-E500H, DMR-ES20**                                          | **Gen 2** (2003–2004)                 | Matsushita MN103S / Early UniPhier                                        | **4 MB** (32 Mbit) TSOP-48 (Am29LV320)            | `0x003F0000..0x003FFFFF` (Last 64 KB) | PATA/IDE HDD (40–160 GB) + MEIHDFS-V1         | 6-pin OCD header (3.3V LVCMOS)                                                           | Pioneer generation of the `U99` panic. Fixed via `panadiga nvram clean` (auto-detected 4 MB).                       |
| **DMR-EH50, DMR-EH52, DMR-EH55, DMR-EH56, DMR-EH60, DMR-EH65, DMR-EH75V, DMR-EH80V, DMR-ES10, DMR-ES30V** | **Gen 3** (2005–2007) [Core Platform] | Matsushita UniPhier `MN2DS0016` (32-bit RISC AM34)                      | **8 MB** (64 Mbit) TSOP-48 (Spansion S29JL064H)   | `0x007F0000..0x007FFFFF` (Last 64 KB) | PATA/IDE HDD (80–250 GB) + MEIHDFS-V2         | Canonical **`P7501`** (JST ZR 1.5mm 6-pin: VDD, nRESET, SDATA, GND, SCLK, EXTRG) | Classical `U99` panic due to EEMUL ring desync. Fixed via `panadiga nvram clean` or in-circuit via RP2040 DWire32A. |
| **DMR-EX75, DMR-EX85, DMR-EX95V, DMR-EX98V, DMR-EH67, DMR-EH68**                                          | **Gen 4** (2006–2008)                 | Matsushita UniPhier Gen 2 (`MN2DS0018` / `MN2DS0022` / `MN2DS0024`) | **16 MB** (128 Mbit) TSOP-56 (Spansion S29GL128N) | `0x00FF0000..0x00FFFFFF` (Last 64 KB) | SATA / PATA HDD (160–320 GB) + MEIHDFS-V2+    | 6-pin DWire32A header (JST ZR 1.5mm)                                                     | Digital DVB-T channel table desync triggering U99. Fixed via `panadiga nvram clean` (auto-detected 16 MB).            |
| **DMR-BW500, DMR-BR500, DMR-BW700, DMR-BW800, DMR-BWT700**                                                | **Gen 5** (2008–2011)                 | Matsushita UniPhier Dual-Core (`MN2WS0040` / `MN2WS0050`)             | **32 MB -- 64 MB** NOR/NAND Flash                 | Dynamic allocation per volume           | SATA 3.5" HDD (250 GB -- 1 TB) + BD-RE Optical | JTAG / DWire hybrid debug header                                                         | Blu-ray generation. MEIHDFS-V3 with AES encryption. Partition extraction via `panadiga hdd extract`.                  |

### 9.1. Why Panadiga Operates Universally Across Generations ("Why It Works")

1. **Mathematical Invariant of Flash Sizing (`len - 64 KB`):**Matsushita firmware engineers strictly allocated the circular EEPROM emulation ring buffer (`EEMUL`) in the highest 64 KB physical sector of the primary NOR Flash across all generations. Instead of hardcoding static offsets, `panadiga nvram clean` dynamically evaluates `offset = len - 65536`. Thus, whether an engineer feeds a 2 MB dump (`0x1F0000`), 4 MB dump (`0x3F0000`), 8 MB dump (`0x7F0000`), or 16 MB dump (`0xFF0000`), the tool safely targets the exact wear-leveling sector without touching IPL code.
2. **DWire32A Bus Protocol Invariance:**The On-Chip Debug (OCD) interface built into the Matsushita AM3x/MN103 core family adheres to a uniform synchronous serial state machine: $1.0	ext{ MHz}$ master clock (`SCLK`), 250 ns setup time, 500 ns hold time, half-duplex open-drain `SDATA` with $4.7	ext{ k}\Omega$ pull-up, and active-LOW open-drain `nRESET`. The RP2040 PIO state machine (`dwire32a.pio`) interacts seamlessly with all models featuring this interface.
3. **Storage Superblock Invariance (MEIHDFS-V1 through V2+):**
   The ASCII `"SH"` magic byte sequence and 28 MB backup mirror geometry were preserved from early HDD models (DMR-E85H) through mature digital units (DMR-EH55, DMR-EX98V).

---

## 10. Technical Documentation Library (30 Manuals, Schematics & Datasheets)

All official documentation is cataloged and hierarchically structured in `manuals_and_schematics/`:

### A. Panasonic DIGA Service Manuals & Technical Schematics

* `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_vol2_schematics.pdf` (177 pages, VEP79138 schematics, P7501 pinout)
* `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_vol1_diagnostics.pdf` (97 pages, diagnostics, U99 error flow)
* `dmr-eh55/service/panasonic_dmr_eh55p9_eh55pc9_service_manual_north_america.pdf` (97 pages, North American NTSC chassis)
* `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_europe.pdf` (92 pages, European PAL chassis)
* `dmr-eh55/service/panasonic_dmr_eh55ee_service_manual_east_europe.pdf` (91 pages, Eastern European SECAM chassis)
* `dmr-eh80v/service/panasonic_dmr_eh80v_service_manual_europe.pdf` (12.2 MB, Triple combo VHS+DVD+HDD service schematics)
* `dmr-es10/service/panasonic_dmr_es10_service_manual_north_america.pdf` (5.3 MB, DVD-RAM service manual, U99 error code analysis)
* `dmr-ex98v/service/panasonic_dmr_ex98v_service_manual_vol1.pdf` (10.4 MB, DVB-T/HDMI combo service manual Vol 1)
* `dmr-eh55/firmware_update/panasonic_dmr_eh55_firmware_installation_guide.pdf` (8 pages, factory firmware CD-R update guide)

### B. Panasonic DIGA User & Operational Instructions

* `dmr-eh55/user/panasonic_dmr_eh55_operating_instructions_global.pdf` (89 pages)
* `dmr-eh55/user/panasonic_dmr_eh55s_operating_instructions_silver.pdf` (89 pages)
* `dmr-eh55/user/panasonic_dmr_eh55_eh56_operating_instructions_europe.pdf` (84 pages)
* `dmr-eh55/user/panasonic_diga_dmr_eh55_user_manual.pdf` (84 pages)
* `dmr-eh55/user/panasonic_dmr_eh55_user_manual_oceania.pdf` (76 pages)
* `dmr-eh55/user/panasonic_dmr_eh55_australia_new_zealand_manual.pdf` (76 pages)
* `dmr-eh55/user/panasonic_dmr_eh55_hdmi_hdavi_control_guide.pdf` (6 pages)
* `dmr-eh55/user/panasonic_dmr_eh55_quick_setup_guide.pdf` (2 pages)
* `dmr-eh56/user/panasonic_dmr_eh56_operating_instructions.pdf` (89 pages)
* `dmr-eh50/user/panasonic_dmr_eh50_user_manual.pdf` (4.1 MB)
* `dmr-eh65/user/panasonic_dmr_eh65_user_manual.pdf` (4.8 MB)
* `dmr-e85h/user/panasonic_dmr_e85h_user_manual.pdf` (2.7 MB)
* `dmr-e55/user/panasonic_dmr_e55_operating_instructions.pdf` (3.2 MB)
* `dmr-e500h/user/panasonic_dmr_e500h_user_manual.pdf` (4.9 MB)
* `dmr-es30v/user/panasonic_dmr_es30v_user_manual.pdf` (7.9 MB)

### C. Silicon Datasheets & Architecture Reference Manuals (`datasheets/`)

* `datasheets/matsushita_mn10300_series_cross_assembler_users_manual.pdf` (Instruction set, addressing modes, register conventions)
* `datasheets/matsushita_mn10300_series_c_debugger_users_manual.pdf` (C-level debugging and On-Chip Debug OCD protocol)
* `datasheets/rp2040_datasheet_raspberrypi.pdf` (5.3 MB, RP2040 microcontroller datasheet, PIO state machines)
* `datasheets/rp2350_datasheet_raspberrypi.pdf` (11.1 MB, RP2350 microcontroller datasheet, Pico 2 architecture)
* `datasheets/spansion_am29lv640d_nor_flash_datasheet.pdf` (1.0 MB, Parallel NOR Flash 64 Mbit TSOP-48 specifications)
* `datasheets/spansion_s29gl128n_nor_flash_datasheet.pdf` (4.2 MB, MirrorBit NOR Flash 128 Mbit TSOP-56 specifications)

---

## 11. Direct Links to Specialized Technical Dossiers (`docs/`)

The full repository of 13 bilingual engineering dossiers is maintained in `docs/en-US/` (with Portuguese mirrors in `docs/pt-BR/` and portal index at `docs/INDEX.md`):

* 🏛️ **[Systems Architecture &amp; Engineering Blueprint](docs/en-US/SYSTEM_ARCHITECTURE_AND_ENGINEERING_BLUEPRINT.md)** ([Portuguese Mirror](docs/pt-BR/ARQUITETURA_E_BLUEPRINT_DE_ENGENHARIA.md))*Master specification: modular decomposition, Pure FP domain, strict MVVM, GDB ring buffer, and silicon invariants.*
* 📱 **[Android APK &amp; Bluetooth Architecture](docs/en-US/ANDROID_APK_AND_BLUETOOTH_ARCHITECTURE.md)** ([Portuguese Mirror](docs/pt-BR/ARQUITETURA_MOBILE_ANDROID_E_BLUETOOTH.md))*Android APK build, Native Rust HAL (BLE NUS / USB-C OTG), and P7501 power safety rules.*
* 🧪 **[Testing Architecture &amp; Compliance Verification](docs/en-US/TESTING_ARCHITECTURE_AND_COMPLIANCE.md)** ([Portuguese Mirror](docs/pt-BR/ARQUITETURA_DE_TESTES_E_CONFORMIDADE.md))*5-layer test pyramid (Pure FP, GDB RSP, MVVM ADTs, CLI SSOT, E2E), traceability matrix, and CI/CD status.*
* ⚡ **[Bench Wiring Blueprint &amp; Electrical Schematic](docs/en-US/BLUEPRINT_BENCH_WIRING_AND_SCHEMATIC.md)** ([Portuguese Mirror](docs/pt-BR/BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md))*Complete wiring harness, P7501 connector pinout, RP2040 hookup, prototype vs production resistors.*
* 💾 **[MEIHDFS Storage &amp; Partition Forensics](docs/en-US/STORAGE_ARCHITECTURE_MEIHDFS_PARTITIONS.md)** ([Portuguese Mirror](docs/pt-BR/ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md))*Panasonic MEIHFS, LBA 0 "SH" MBR, 28 MB mirror, 8 MB journal, and firmware partitions.*
* 🔐 **[Dossier on Cryptography &amp; PANA_DVD.FRM Container](docs/en-US/DOSSIER_CRYPTOGRAPHY_AND_FIRMWARE_PANA_DVD.md)** ([Portuguese Mirror](docs/pt-BR/DOSSIE_CRIPTOGRAFIA_E_FIRMWARE_PANA_DVD.md))*Optical container PANA_DVD.FRM, CPRM/C2 ciphers, 64-bit ECB codebooks, and entropy analysis.*
* 🔬 **[Ghidra &amp; GDB Integration Strategy](docs/en-US/GHIDRA_GDB_INTEGRATION_STRATEGY.md)** ([Portuguese Mirror](docs/pt-BR/ESTRATEGIA_INTEGRACAO_GHIDRA_GDB.md))*AM34/MN103S processor architecture, Ghidra SLED module, and GDB Remote Serial Protocol (:3333).*
* 🛠️ **[Bench Probe Assembly &amp; Photographic Guide](docs/en-US/BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md)** ([Portuguese Mirror](docs/pt-BR/GUIA_MONTAGEM_E_FOTOS_PROBE_BANCADA.md))*Step-by-step soldering, JST ZR 1.5mm female socket mating, Pin 4 GND continuity verification.*
* 📋 **[Technical Inventory &amp; Operations Manual](docs/en-US/TECHNICAL_INVENTORY_AND_OPERATIONS_MANUAL.md)** ([Portuguese Mirror](docs/pt-BR/INVENTARIO_TECNICO_E_GUIA_DE_OPERACAO.md))*File inventory, cryptographic SHA-256 manifests, and step-by-step U99 unbrick runbook.*
* 📊 **[Final Audit &amp; Modularization Report](docs/en-US/FINAL_AUDIT_AND_MODULARIZATION_REPORT.md)** ([Portuguese Mirror](docs/pt-BR/RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md))*Modularization audit, cross-stack integration (Rust/C/Python/TypeScript), 9 quality gates status.*
* 📡 **[Logging &amp; Telemetry Subsystem Specification](docs/en-US/LOGGING_AND_TELEMETRY_SUBSYSTEM.md)** ([Portuguese Mirror](docs/pt-BR/SUBSISTEMA_LOGGING_E_TELEMETRIA.md))*Structured JSON logging, thread-safe sinks, and real-time serial terminal integration.*
* 💡 **[Semiconductor Theory &amp; Practical U99 Unbricking](docs/en-US/THEORY_AND_PRACTICE_U99_UNBRICK_MATSUSHITA_CRYPTO.md)** ([Portuguese Mirror](docs/pt-BR/TEORIA_E_PRATICA_UNBRICK_U99_E_CRIPTOGRAFIA_MATSUSHITA.md))*Semiconductor physics of NOR Flash, EEMUL circular wear-leveling, and 64 KB NVRAM sanitization.*
* 🛡️ **[Zero-Error RAG Engine &amp; Physical Safety Invariants](docs/en-US/ZERO_ERROR_RAG_ENGINE_AND_PHYSICAL_SAFETY.md)** ([Portuguese Mirror](docs/pt-BR/ZERO_ERROR_RAG_ENGINE_E_SEGURANCA_FISICA.md))
  *6-layer defense system (QueryGuard, CRAG, GroundedSynthesis, FactCritic, InvariantShield).*

---

## 12. Recommended Laboratory Equipment & Consumables

* **Microcontroller Probe:** Raspberry Pi Pico (RP2040) or Raspberry Pi Pico 2 (RP2350).
* **Mating Connector:** JST ZR female 6-pin socket, 1.5mm pitch (`B6B-ZR-3R4` compatible).
* **Passive Protection:** Four $100\,\Omega$ $0.25	ext{W}$ resistors, one $4.7	ext{ k}\Omega$ pull-up resistor, one $1.0	ext{ k}\Omega$ ADC isolation resistor.
* **Instrumentation:** Digital Multimeter (continuity testing), 2-channel 100 MHz Oscilloscope or 24 MHz USB Logic Analyzer.
* **Storage Forensics:** PATA/IDE to USB 3.0 bridge adapter with independent 12V/5V Molex power supply.

---

## 13. Total Quality, Test Pyramid & 9 Quality Gates

The Panadiga engineering ecosystem enforces a strict **9 Quality Gates** anti-regression pipeline:

```text
       Layer 5: End-to-End E2E Tests (Playwright Chromium 8/8)
       Layer 4: Desktop UI Unit Tests (Vitest + React Testing Library 33/33)
       Layer 3: C11 Pico Firmware Host Tests (GCC Host Test Harness 7/7)
       Layer 2: Python Typer CLI & RAG Pytest Verification (Pytest 21/21, Mypy Strict)
       Layer 1: Native Rust Core & RAG Integration Tests (Cargo Test 41/41)
```

### 13.1. The 9 Active Quality Gates

1. **Rust Cargo Check & Test Gate:** Verifies formatting (`cargo fmt`), type safety, and all 41 unit/integration tests.
2. **TypeScript Strict Type-Check Gate:** `tsc --noEmit` verifies strict frontend types with zero `any` leaks.
3. **Frontend Vitest Unit Suite:** 33 ViewModel and algebraic state transition tests.
4. **Copy/Paste Detector (`jscpd`):** Enforces 0.00% code duplication threshold.
5. **UI/UX Completeness Linter (`uxlint`):** Verifies loading, empty, and error state coverage across all views (100/100).
6. **Playwright E2E Suite:** 8 browser-level functional tests running against headless Chromium.
7. **Ghidra Script Generator Gate:** Validates automatic export of memory maps and symbols from Rust SSOT.
8. **C11 Firmware Host Test Gate:** Compiles and tests bare-metal Pico firmware with `-Wall -Wextra -Werror`.
9. **Python Typer & Pytest Gate:** Verifies strict typing (`mypy --strict`) and 21 automated tests.

### 13.2. Executing the Full Validation Suite

```bash
# On Windows PowerShell:
.validate_all.ps1

# On Windows CMD:
.validate_all.cmd

# On Linux / macOS / WSL:
./run.sh --validate
```

---

## 14. Artificial Intelligence Governance, MCP Server & Structured Logging

### 14.1. Dedicated Specialist Subagents (`.agents/agents/`)

The project configures 5 specialized subagents for autonomous engineering assistance:

* **`panasonic-hardware-engineer`:** Schematics, PCB layout, voltages, and DWire32A signal integrity.
* **`panasonic-crypto-analyst`:** Block cipher analysis, ECB codebooks, and CPRM/DivX forensics.
* **`panasonic-storage-architect`:** MEIHDFS partition geometry, MBR repair, and filesystem forensics.
* **`panasonic-desktop-architect`:** Tauri 2.0 desktop frontend, React 19, and typed IPC bindings.
* **`panasonic-rag-evaluator`:** Zero-Error RAG engine, factual grounding, and physical safety guards.

### 14.2. Model Context Protocol (MCP) Server (`tools/panadiga_mcp_server.py`)

A standardized JSON-RPC 2.0 MCP server exposes native tools for AI agents:

* `panadiga_rag_query`: Queries local RAG knowledge base with factual grounding.
* `panadiga_manuals_search`: Searches the 30 PDF technical manuals and datasheets with page citations.
* `panadiga_inspect_hdd`: Inspects partition boundaries and validates the MEIHDFS superblock.
* `panadiga_analyze_frm`: Analyzes optical update containers and extracts module TOCs.

### 14.3. Unified Logging Subsystem (`src/logger.rs`)

All components log exclusively through structured sinks:

* **Zero console.log/println:** No raw or unstructured logging is permitted in production code.
* **Log Levels:** `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` with microsecond timestamps and subsystem tags.
* **Desktop GUI Streaming:** Real-time log events stream directly into the UI terminal panel.

---

## 15. Engineering Compliance & License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for complete terms.

All reverse engineering, cryptanalysis, and hardware documentation in this project were conducted strictly for interoperability, historical preservation, and repair of obsolete electronics in accordance with international reverse engineering exceptions.
