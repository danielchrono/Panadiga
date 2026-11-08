# Testing Architecture & Compliance Matrix: Panadiga Ecosystem

## Multi-Layer Verification: Pure FP Domain, Strict MVVM, Introspective GDB & SSOT

> **Version:** 1.0.0 | **Approval Date:** November 2026
> **Status:** 100% Quality Gates Passed (41 Rust Tests, 33 Vitest Tests, 8 Playwright E2E, 27 Pytest, 7 C11 Host Tests)
> **Authors:** Panadiga Systems Architecture & Reverse Engineering Team

---

## 1. Overview & Testing Philosophy

The architectural evolution of **Panadiga** required a fundamental restructuring of its testing strategy. The previous approach, based on ad-hoc tests tightly coupled to physical filesystem paths, was replaced by a **Multi-Layer Testing Pyramid**, where each test strictly protects one or more architectural principles:

```
                          PANADIGA TESTING PYRAMID
                                       ▲
                                      / \
                                     /   \
                                    / E2E \   (8 Playwright Chromium tests + Multiplatform CI/CD)
                                   /───────\
                                  /  MVVM   \  (33 Vitest tests: State Machines, Subcomponents, Logger)
                                 /───────────\
                                /  GDB & HAL  \ (7 C11 Host Tests + Telemetry Ring Buffer + Checksums)
                               /───────────────\
                              /  FP Pure Domain \ (41 Rust tests: &[u8] slices, Clap Matrix, SSOT Memory Map)
                             /───────────────────\
```

---

## 2. Layer 1: Pure FP Domain (Immutable Slice Tests on `&[u8]`)

### Architectural Goal

Decouple low-level forensic and cryptanalysis logic from disk I/O. All math, parsing, and decoders must operate as pure functions over immutable byte slices (`&[u8]`).

### Implemented Test Suite

| Test File                             | Tested Function / Module                | Compliance Assurance                                                                                                                             |
| :------------------------------------ | :-------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/nvram/pure_fp_tests.rs`        | `clean_nvram_slice(&[u8])`            | Tests surgical microsector sanitization (SA134..SA141) directly in RAM, verifying bootloader immutability and zeroing of corrupt U99 parameters. |
| `src/hdd/inspect/pure_hdd_tests.rs` | `inspect_hdd_slice_pure(&[u8])`       | Validates proprietary Panasonic MBR, mirror table at LBA 57344, and partition descriptors over memory buffers without filesystem access.         |
| `tests/entropy_math_tests.rs`       | `calculate_entropy(&[u8])`            | Validates Shannon entropy calculation (0.0 for pure patterns, 1.0 for alternating binary, 8.0 for uniform distribution).                         |
| `tests/crypto_codebook_tests.rs`     | `test_real_frm_dump_toc_and_codebook` | Dissects 48-byte periodic TOC entries and known plaintext-ciphertext dictionary from `PANA_DVD.FRM`.                                           |

---

## 3. Layer 2: Introspective GDB & Hardware Protocol (DWire32A / RSP)

### Architectural Goal

Guarantee absolute protocol fidelity in the serial-to-TCP GDB bridge, prevent packet corruption, and protect the host RP2040/RP2350 microcontroller.

### Implemented Test Suite

| Test File                                            | Tested Function / Module                | Compliance Assurance                                                                                                                                    |
| :--------------------------------------------------- | :-------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `tests/debugger_gdb_tests.rs`        | `test_gdb_rsp_checksum_calculation`   | Formal validation of 2-digit hex checksum calculation (`sum % 256`) for RSP commands (`qSupported`, `g`, `?`, `c`, `s`, `OK`).            |
| `src/desktop/pico/gdb_bridge.rs`                   | `test_verify_and_classify_rsp_packet` | Tests formal RSP packet classifier: `$` prefix, `#` delimiter, checksum comparison, and command type tagging.                                       |
| `src/desktop/pico/gdb_bridge.rs`                   | `test_telemetry_ring_capacity`        | Validates introspective circular buffer (`RingBuffer<GdbPacketTelemetryEvent>`): fixed capacity, FIFO eviction without leaks, chronological ordering. |
| `dwire32a_jtag_research/tests/test_gdb_rsp_host.c` | Host C11 Firmware (`-Werror`)         | 7 protocol integration tests compiled under GCC C11: SRAM read/write, rejection of write to IPL flash (`E05`), AM34 registers, halt/continue control. |

---

## 4. Layer 3: Strict MVVM, Subcomponents & Dedicated Logger

### Architectural Goal

Eliminate scattered booleans in favor of closed finite state machines (ADTs / Discriminated Unions) in React, isolate UI subcomponents according to SRP, and centralize structured logging.

### Implemented Test Suite (Vitest)

| Test File                        | Component / ViewModel                                | Compliance Assurance                                                                                                                                                    |
| :------------------------------- | :--------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `useInCircuitErase.test.ts`    | `useInCircuitErase`                                | Verifies discrete states (`idle`, `confirming`, `erasing`, `success`, `error`) and callback triggering during surgical unbrick.                               |
| `useSerialConsole.test.ts`     | `useSerialConsole`                                 | Validates interactive terminal: command history ring buffer, bounded log array, and command dispatch.                                                                   |
| `useGdbBridge.test.ts`         | `useGdbBridge`                                     | Tests bridge state transitions (`stopped`, `starting`, `running`, `stopping`, `error`), async telemetry polling, and log emission.                            |
| `GdbServerComponents.test.tsx` | `GdbServerControls` & `GdbExternalToolLaunchers` | Tests decomposed GDB card: status badge rendering, TCP start/stop invocation, and tool launcher button handlers (GDB Multiarch, OpenOCD AM34, Ghidra, HxD, PowerShell). |
| `logger.test.ts`               | `PanadigaLogger`                                   | Tests frontend logging subsystem: severity filtering (`minLevel`), multi-sink dispatch, subscription disposal, and error resilience.                                  |
| `translations.test.ts`         | `i18n` Engine                                      | Validates complete parity between `pt-BR` and `en-US` dictionaries and canonical key resolution.                                                                    |

---

## 5. Layer 4: CLI Decomposition & SSOT Constants

### Architectural Goal

Guarantee strong CLI argument parsing via Clap, deterministic handler routing, and zero magic numbers or file paths hardcoded outside `constants.rs`.

### Implemented Test Suite

| Test File                   | Tested Module                                     | Compliance Assurance                                                                                                                                                                            |
| :-------------------------- | :------------------------------------------------ | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tests/firmware_scan_tests.rs`       | `test_cli_args_parsing_matrix`                  | Validates parsing and deserialization of CLI subcommands (`memory-map`, `gdb-config`, `pico list`, `nvram clean`, etc.), aliases (`tui`, `verify`), and rejection of unknown flags. |
| `tests/system_constants_tests.rs`    | `test_constants_memory_map_and_candidate_paths` | Validates all 8 official AM34/UniPhier memory regions, ASCII table formatting, canonical JSON schema export, and presence of universal candidate paths.                                         |
| `tests/pico_bench_tests.rs`          | `test_all_host_controller_platforms_matrix`     | Validates USB VID/PID hardware identification matrix for Raspberry Pi Pico, Pico W, Pico 2 (ARM/RISC-V), and clones.                                                                            |

---

## 6. Layer 5: End-to-End (E2E) & Multi-Platform CI/CD

### Playwright Coverage (Chromium)

8 automated browser tests verify end-to-end user workflows:

1. `navigation.spec.ts`: Default dashboard rendering with real-time system metrics.
2. `navigation.spec.ts`: Seamless navigation across all workbench modules without state breakage.
3. `nvram.spec.ts`: Spansion NOR Flash S29JL064H specifications and surgical action card rendering.
4. `nvram.spec.ts`: 8 microsector breakdown (SA134..SA141) and integrity indicators.
5. `pico-bench.spec.ts`: Host controller telemetry and architecture badge display.
6. `pico-bench.spec.ts`: P7501 connector (DWire32A) pinout table with ground loop disclaimers.
7. `pico-bench.spec.ts`: Cold Break-In mode switching (Mode 0, 1, and 2).
8. `pico-bench.spec.ts`: Interactive serial terminal with command echo and transmission.

### Continuous Integration Matrix (GitHub Actions)

All 8 jobs execute under every push to `main`:

- **Android APK Build (Tauri 2.0 / ARM64)**: Native Android compilation with `cargo-ndk`.
- **Rust Workspace (Ubuntu Linux)**: `cargo check` and full test suite.
- **Ghidra Generator & Python Tools**: `py_compile`, `mypy --strict`, and `pytest` (27 tests).
- **Desktop UI**: `tsc --noEmit`, Vitest (33 tests), `uxlint` (100/100), and Playwright E2E.
- **Rust Workspace (macOS latest ARM64)**: Apple Silicon validation.
- **Rust Workspace (Windows latest)**: Native Windows build and test.
- **Code Quality & Duplication**: `cargo fmt`, `clippy`, and `jscpd` clone detector (strict 0% threshold).
- **Pico Firmware Dual Target**: UF2 compilation for RP2040 and RP2350 (Cortex-M33).

---

## 7. Traceability Matrix: Architectural Requirement ➔ Protecting Test

| Architectural Requirement            | Mitigated Risk                                                                    | Protecting Test Suite                                                                            |
| :----------------------------------- | :-------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------- |
| **Pure FP Domain**             | Regressions in forensic decoders dependent on physical file state or disk paths.  | `pure_fp_tests.rs`, `pure_hdd_tests.rs`, `test_entropy_calculation`                        |
| **Strict MVVM**                | UI locking in intermediate states due to scattered boolean flags.                 | `useInCircuitErase.test.ts`, `useGdbBridge.test.ts`                                          |
| **Introspective GDB**          | RSP packet corruption or telemetry buffer overflow under high traffic.            | `test_gdb_rsp_checksum_calculation`, `test_telemetry_ring_capacity`, `test_gdb_rsp_host.c` |
| **Component SRP**              | Regressions in quick tool launchers or TCP debugging socket controls.             | `GdbServerComponents.test.tsx`                                                                 |
| **Zero Raw Prints / Consoles** | Stdout pollution breaking JSON-RPC parsing for MCP servers or external consumers. | `logger.test.ts`, `test_logger_custom_sink`, `test_panadiga_mcp_server.py`                 |
| **SSOT Constants**             | Memory map address conflicts between Ghidra, TUI, CLI, and Desktop.               | `test_constants_memory_map_and_candidate_paths`, `test_system_constants_integrity`           |
| **CLI Decomposition**          | Silent parsing errors or ignored flags in the new slim entrypoint.                | `test_cli_args_parsing_matrix`                                                                 |
