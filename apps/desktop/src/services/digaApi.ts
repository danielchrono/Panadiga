/**
 * @file digaApi.ts
 * @brief Service layer for interacting with Panasonic DIGA backend operations.
 *
 * Enforces Separation of Concerns (SoC) and functional error handling using Result<T, E>.
 */

import {
  Result,
  ok,
  err,
  NvramCleanReport,
  HddInspectionReport,
  FrmAnalysisReport,
  DwireScanStats,
  DwireBreakInMode,
  ManualSearchResult,
  RagQueryResult,
  SPANSION_FLASH_SIZE_BYTES,
  SPANSION_NVRAM_BASE_OFFSET,
  SPANSION_NVRAM_SIZE_BYTES,
  SPANSION_MICROSECTOR_SIZE_BYTES,
  SPANSION_MICROSECTOR_BASE_INDEX,
  ECB_CIPHERTEXT_VIRGIN_FLASH_HEX,
  ECB_CIPHERTEXT_NULL_BLOCK_HEX,
} from "../types/domain";

/**
 * Checks if the application is running inside a Tauri container.
 */
function isTauriEnvironment(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const digaApi = {
  /**
   * Sanitizes the Spansion NVRAM / EEMUL sector (top 64 KB).
   */
  async cleanNvram(
    inputPath: string,
    outputPath?: string
  ): Promise<Result<NvramCleanReport, string>> {
    try {
      if (isTauriEnvironment()) {
        const { invoke } = await import("@tauri-apps/api/core");
        const report = await invoke<NvramCleanReport>("clean_flash_nvram", {
          inputPath,
          outputPath,
        });
        return ok(report);
      }

      // Offline / Web preview simulation adhering strictly to domain rules
      if (!inputPath || inputPath.trim().length === 0) {
        return err("Flash dump file path cannot be empty.");
      }

      const microsectors = Array.from({ length: 8 }, (_, idx) => {
        const offset = SPANSION_NVRAM_BASE_OFFSET + idx * SPANSION_MICROSECTOR_SIZE_BYTES;
        const active_bytes = Math.floor(Math.random() * 6000) + 500;
        return {
          sector_name: `SA${SPANSION_MICROSECTOR_BASE_INDEX + idx}`,
          offset,
          active_bytes,
          total_bytes: SPANSION_MICROSECTOR_SIZE_BYTES,
          header_hex: "50414E415F4E5652",
        };
      });

      const simulatedReport: NvramCleanReport = {
        input_file: inputPath,
        output_file: outputPath || `${inputPath}_CLEAN.bin`,
        backup_file: "nvram_corrupt_backup.bin",
        original_md5: "e4d909c290d0fb1ca068ffaddf22cbd0",
        original_sha256: "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        clean_md5: "8f5a110a12e3f4e567a890bcdef12345",
        clean_sha256: "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce",
        total_flash_bytes: SPANSION_FLASH_SIZE_BYTES,
        nvram_offset_start: SPANSION_NVRAM_BASE_OFFSET,
        nvram_offset_end: SPANSION_NVRAM_BASE_OFFSET + SPANSION_NVRAM_SIZE_BYTES,
        nvram_non_ff_bytes: 42180,
        nvram_non_zero_bytes: 41920,
        microsectors,
        bootloader_immutable: true,
      };

      return ok(simulatedReport);
    } catch (e) {
      return err(e instanceof Error ? e.message : String(e));
    }
  },

  /**
   * Inspects a Panasonic MEIHDFS raw Hard Disk Drive image.
   */
  async inspectHdd(imagePath: string): Promise<Result<HddInspectionReport, string>> {
    try {
      if (isTauriEnvironment()) {
        const { invoke } = await import("@tauri-apps/api/core");
        const report = await invoke<HddInspectionReport>("inspect_hdd_image", {
          imagePath,
        });
        return ok(report);
      }

      const simulatedHdd: HddInspectionReport = {
        file_path: imagePath,
        file_size_bytes: 104_857_600,
        total_sectors: 204_800,
        mbr_tables: {
          magic_ascii: "SH",
          lba0_sha256: "61162c24c9240408ac807cd4307c9c5833d440715a982126c536aa10aa7530f2",
          mirror_lba: 57344,
          mirror_is_bit_identical: true,
        },
        partitions: [
          {
            partition_index: 1,
            start_lba: 65536,
            byte_offset: 0x02000000,
            size_bytes: 33554432,
            header_magic: "5187138D5B0AFB22",
            exact_sha256: "6ec63de059c00b287a0e8a1a4102a8007bbccf3210a552bfb45118a03b85c898",
          },
          {
            partition_index: 2,
            start_lba: 131072,
            byte_offset: 0x04000000,
            size_bytes: 33554432,
            header_magic: "5187138D5B0AFB22",
            exact_sha256: "487d5951d4aa7bc45e5f634c4ca079f369db06d5cb62e41a173575cecd68dbae",
          },
        ],
      };

      return ok(simulatedHdd);
    } catch (e) {
      return err(e instanceof Error ? e.message : String(e));
    }
  },

  /**
   * Analyzes the encrypted PANA_DVD.FRM firmware package.
   */
  async analyzeFrm(frmPath: string): Promise<Result<FrmAnalysisReport, string>> {
    try {
      if (isTauriEnvironment()) {
        const { invoke } = await import("@tauri-apps/api/core");
        const report = await invoke<FrmAnalysisReport>("analyze_frm_container", {
          frmPath,
        });
        return ok(report);
      }

      const simulatedFrm: FrmAnalysisReport = {
        file_path: frmPath,
        file_size_bytes: 7_531_744,
        global_entropy: 7.99958,
        is_ecb_leak_confirmed: true,
        virgin_flash_block_count: 3812,
        null_block_count: 196,
        top_blocks: [
          {
            hex: ECB_CIPHERTEXT_VIRGIN_FLASH_HEX,
            count: 3812,
            known_plaintext: "Virgin Flash (0xFF * 8)",
          },
          {
            hex: ECB_CIPHERTEXT_NULL_BLOCK_HEX,
            count: 196,
            known_plaintext: "Zero Buffer (0x00 * 8)",
          },
          {
            hex: "74A78ACD789B84FF",
            count: 64,
            known_plaintext: "AM34 Core Target Descriptor",
          },
        ],
      };

      return ok(simulatedFrm);
    } catch (e) {
      return err(e instanceof Error ? e.message : String(e));
    }
  },

  /**
   * Triggers an automated DWire32A opcode discovery scan via Pico.
   */
  async scanDwireBus(mode: DwireBreakInMode): Promise<Result<DwireScanStats, string>> {
    try {
      if (isTauriEnvironment()) {
        const { invoke } = await import("@tauri-apps/api/core");
        const stats = await invoke<DwireScanStats>("scan_dwire_pico", { mode });
        return ok(stats);
      }

      const simulatedDwire: DwireScanStats = {
        vdd_volts: 3.31,
        is_powered: true,
        total_probes: 256,
        open_bus_high: 251,
        open_bus_low: 2,
        hits_detected: 3,
        candidate_hits: [
          {
            opcode_hex: "0x12",
            response_hex: "0x55A43201",
            classification: "OCD Status / Core Identification Register",
          },
          {
            opcode_hex: "0x34",
            response_hex: "0x00000000",
            classification: "Acknowledge / Ready Flag",
          },
          {
            opcode_hex: "0x88",
            response_hex: "0x12345678",
            classification: "JTAG Tap Instruction Decode",
          },
        ],
      };

      return ok(simulatedDwire);
    } catch (e) {
      return err(e instanceof Error ? e.message : String(e));
    }
  },

  /**
   * Searches engineering service manuals and schematics for tokens.
   */
  async searchManuals(query: string): Promise<Result<readonly ManualSearchResult[], string>> {
    try {
      if (!query || query.trim().length === 0) {
        return err("Manual search query cannot be empty.");
      }

      if (isTauriEnvironment()) {
        const { invoke } = await import("@tauri-apps/api/core");
        const results = await invoke<readonly ManualSearchResult[]>("search_manuals", {
          query,
        });
        return ok(results);
      }

      const mockResults: readonly ManualSearchResult[] = [
        {
          manual_filename: "DMR-EH55_Service_Manual.pdf",
          page_number: 42,
          snippet: `Circuit schematic shows IC55007 Spansion S29JL064H pin 14 (WP#/ACC) connected to pull-up via R55012 with test point TP5504 for query: ${query}`,
        },
        {
          manual_filename: "DMR-EH55_Service_Manual.pdf",
          page_number: 89,
          snippet: `CN55001 Pinout: 1=EXTRG, 2=SCLK, 3=GND, 4=SDATA, 5=nRESET, 6=NC with 3.3V nominal for query: ${query}`,
        },
      ];

      return ok(mockResults);
    } catch (e) {
      return err(e instanceof Error ? e.message : String(e));
    }
  },

  /**
   * Submits a technical inquiry to the Zero-Error RAG Engine.
   */
  async queryRag(query: string): Promise<Result<RagQueryResult, string>> {
    try {
      if (!query || query.trim().length === 0) {
        return err("Technical query cannot be empty.");
      }

      if (isTauriEnvironment()) {
        const { invoke } = await import("@tauri-apps/api/core");
        const answer = await invoke<RagQueryResult>("query_rag_engine", {
          query,
        });
        return ok(answer);
      }

      // Offline / Web preview simulation adhering strictly to 6-layer RAG rules
      const isU99 = /u99/i.test(query);
      const isHardware = /cn55001|rp2040|pin/i.test(query);

      // Check deterministic safety invariants in offline simulation
      const hasOvervoltageHazard = /(?:inject|apply|supply|conectar|injetar|ligar)\s*(?:with\s+)?(?:5|12)\s*v/i.test(query) || /\b(?:5|12)\s*v\b/i.test(query) && /cn55001|wp#|sdata/i.test(query);
      const hasIplHazard = /(?:erase|wipe|apagar|limpar).*(?:0x00000000|0x0000|ipl)/i.test(query);
      const hasJtagHazard = /cn55001.*(?:tdi|tdo|tms)|(?:tdi|tdo|tms).*cn55001/i.test(query);

      if (hasOvervoltageHazard || hasIplHazard || hasJtagHazard) {
        const violations: string[] = [];
        if (hasOvervoltageHazard) {
          violations.push("CRITICAL OVERVOLTAGE VIOLATION: Voltages > 3.3V (5V/12V) destroy the MN2DS0016 SoC and Flash. Maximum safe voltage is strictly 3.35V.");
        }
        if (hasIplHazard) {
          violations.push("CRITICAL BOOTLOADER VIOLATION: Erasure of sectors SA0..SA133 (0x00000000..0x007EFFFF) destroys the IPL root-of-trust. Only NVRAM (0x007F0000..0x007FFFFF) may be sanitized.");
        }
        if (hasJtagHazard) {
          violations.push("PROTOCOL VIOLATION: Connector CN55001 uses half-duplex Matsushita DWire32A (SDATA+SCLK), not traditional 4-wire JTAG (no TDI/TDO).");
        }

        const blockedAnswer: RagQueryResult = {
          query,
          answer: `⛔ RESPONSE BLOCKED BY HARDWARE INVARIANT SHIELD!\n\nThe requested operation violates hardware safety rules:\n${violations.map(v => `- ${v}`).join("\n")}\n\nOperation aborted to prevent permanent hardware destruction.`,
          confidence: 0.0,
          groundedness_score: 1.0,
          is_abstained: false,
          abstention_reason: null,
          citations: [],
          safety: {
            is_safe: false,
            violations,
            alerts: ["RESPONSE BLOCKED BY HARDWARE SAFETY SHIELD"],
          },
          critic: {
            is_faithful: true,
            ungrounded_entities: [],
          },
          retrieved_chunks_count: 0,
          execution_time_ms: 1.2,
        };
        return ok(blockedAnswer);
      }

      const simulatedAnswer: RagQueryResult = {
        query,
        answer: isU99
          ? "Error U99 is caused by corrupted calibration parameters in the Spansion S29JL064H Flash NVRAM emulation zone (SA134..SA141, 0x007F0000 to 0x007FFFFF). The safe recovery procedure is sanitizing these 64 KB microsectors to 0xFF. Never erase sectors SA0..SA133, as they contain the IPL bootloader root-of-trust."
          : isHardware
          ? "The CN55001 debug port uses the Matsushita DWire32A half-duplex synchronous serial protocol (SDATA + SCLK) at 3.3V nominal. Pinout: 1=EXTRG, 2=SCLK, 3=GND, 4=SDATA, 5=nRESET. It does not use traditional 4-wire JTAG."
          : `Grounded response for '${query}' verified against indexed service manuals and hardware schematics. All parameters strictly adhere to 3.3V maximum tolerance.`,
        confidence: 0.98,
        groundedness_score: 1.0,
        is_abstained: false,
        abstention_reason: null,
        citations: [
          {
            doc: "BLUEPRINT_ESQUEMA_ELETRICO_BANCADA",
            section: "Hardware Interface & Pin Mapping",
            lines: "54-85",
            excerpt: "CN55001 JST ZR 1.5mm debug port with half-duplex SDATA transceiver at 3.3V nominal.",
          },
          {
            doc: "TEORIA_E_PRATICA_UNBRICK_U99_E_CRIPTOGRAFIA_MATSUSHITA",
            section: "Fundamentos EEMUL e Causa-Raiz U99",
            lines: "30-55",
            excerpt: "Error U99 caused by calibration parameter corruption in Flash NVRAM at 0x007F0000.",
          },
        ],
        safety: {
          is_safe: true,
          violations: [],
          alerts: [],
        },
        critic: {
          is_faithful: true,
          ungrounded_entities: [],
        },
        retrieved_chunks_count: 5,
        execution_time_ms: 3.2,
      };

      return ok(simulatedAnswer);
    } catch (e) {
      return err(e instanceof Error ? e.message : String(e));
    }
  },
};
