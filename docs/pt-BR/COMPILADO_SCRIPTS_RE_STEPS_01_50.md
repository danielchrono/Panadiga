# Compilado de Consulta — Scripts Python RE (`Steps 1–50` e Utilitários CAD/Bancada)

> Referência permanente de consulta rápida consolidando todos os **119 scripts Python** gerados durante a investigação (`54` scripts `step1..step50` + `65` utilitários CAD/PDF/bancada), evitando re-executar ou recriar análises.
> As rotinas reutilizáveis e testadas em CI foram promovidas para os pacotes [`scripts/re/`](../../scripts/re/) e [`scripts/bench/`](../../scripts/bench/).

## 1. Módulos de Produção Promovidos (CI + `mypy --strict` + `pytest`)

| Módulo Permanente | Responsabilidade | Substitui / Consolida |
| :--- | :--- | :--- |
| [`scripts/re/mn10300.py`](../../scripts/re/mn10300.py) | Decodificador `PC=cur_insn+disp` (`0xCA/CC/CD/DC/DD`), scanner `0xFC <imm32>`, ponteiros `LE32` | Lógica repetida em `step10..step50` (`step15b`, `step41`, `step47`) |
| [`scripts/re/pana_frm.py`](../../scripts/re/pana_frm.py) | Descompactação `DRV.BIN`, parser `EXTRHEAD`, CLI 62 cmds (`<8sIII`), 256 CDB ATAPI, Sense LUT `0x400de35c`, dicionário CP932 `0x34`-stride (`0x83A7`) | `step5`, `step8`, `step30..34`, `step44..46`, `step49` |
| [`scripts/bench/protocol.py`](../../scripts/bench/protocol.py) | Transporte seguro `SerialTransport` (`COM10`) / `MockBenchTransport`, allowlist não-destrutiva, `BenchSession` com `ISOLATE` garantido no `__exit__` | Scripts ad-hoc em `COM10` / `test_jigboot_cd.py` |
| [`scripts/bench/decoders.py`](../../scripts/bench/decoders.py) | Parsers de `STATUS` (detecção bug `GP26 0.02V`), `PINCHECK`, `BUSDIAG` e logs UART AM34 (UTF-16LE/UTF-8, `[SDCD]`, `SENSE[.. 83 A7 ..]`) | Parsers avulsos e inspeção manual de `.log` |
| [`scripts/bench/runner.py`](../../scripts/bench/runner.py) | CLI `python -m scripts.bench.runner` (`health`, `capture`, `decode-log`, `smoke-suite`) | Execução manual de sessões Pico |

## 2. Catálogo Completo dos Scripts Progressivos (`step1`–`step50`: 54 scripts)

| Script Original (`scratch/`) | Linhas | Alvos Inspecionados | Resumo / Achado Principal |
| :--- | :---: | :--- | :--- |
| `step1b_targeted.py` | 163 | Service Manual PDF, Live Logs | 1. Inspect XFWE, I2C5, P59001, P50001, P58001 on vol2 p.113, p.115, p.117, p.122, p.125, p.126 |
| `step1_deep_verify.py` | 184 | Service Manual PDF, Live Logs | 1. Search vol2 for relevant keywords across all pages |
| `step2_forensics.py` | 96 | Service Manual PDF | 1. All USER_EXPLICIT inputs from step 12500 onward in transcript_full.jsonl |
| `step3_clean_and_bin.py` | 271 | PANA_DVD.FRM, Live Logs | PART A: Manuals |
| `step3_next1.py` | 185 | Live Logs | Find PDFs |
| `step4_boot_drv_inspect.py` | 105 | BOOT.BIN, DRV.BIN, COM10/Pico, Service Manual PDF | 1. Inspect EH58 BOOT.BIN all interesting strings |
| `step5_decomp_and_crypto.py` | 86 | BOOT.BIN, DRV_decomp, DRV.BIN | 1. Decompress EH58 DRV.BIN |
| `step6_drv_boot_deep.py` | 86 | BOOT.BIN, DRV_decomp | Keep strings that look like English words/identifiers/messages (not random opcodes) |
| `step7_drv_and_boot_deep.py` | 186 | BOOT.BIN, DRV_decomp, Service Manual PDF | 1. DRV_decomp.bin base address & Sense Table <-> Strings |
| `step8_mecha_83a7_and_boot_re.py` | 151 | BOOT.BIN, DRV_decomp | 1. Decode Shift-JIS & error tables in DRV_decomp.bin (0x0e3400..0x0ea200 & 0x0de000..0x0de600) |
| `step9_decode_83a7_and_boot_disasm.py` | 139 | BOOT.BIN, DRV_decomp | 1. Inspect DRV_decomp.bin sections & headers (0x00000..0x10000) & find base address! |
| `step10_table_0x83a7_and_boot_disasm.py` | 92 | BOOT.BIN, DRV_decomp | 1. DUMP THE ENTIRE uint16 ERROR CODE + SHIFT-JIS STRING TABLE AROUND 0x0e7800..0x0e8ec0! |
| `step11_trace_83a7_and_boot_disasm.py` | 107 | BOOT.BIN, DRV_decomp, Service Manual PDF | 1. MN10300 / AM33 Quick Structural Disassembler & String Annotator |
| `step12_toc_mn10300_disasm.py` | 253 | BOOT.BIN, DRV_decomp, Service Manual PDF | 1. Instant TOC of Schematic PDFs |
| `step13_real_disasm_and_pages.py` | 135 | BOOT.BIN, DRV_decomp, Service Manual PDF | 1. Exact opcode lengths in MN10300/AM33 so we NEVER misalign 32-bit immediates! |
| `step14_trace_sense56_and_boot_i2c.py` | 96 | BOOT.BIN, DRV_decomp | 1. Inspect MN10300 opcode table pages 246..265 of cross_assembler PDF (silently) |
| `step15a_disasm_probe.py` | 140 | BOOT.BIN, DRV_decomp | Scan for 'fc <subop> <u32>' candidates (only exact fc xx imm32; validate known op ranges) |
| `step15b_exact_trace.py` | 177 | BOOT.BIN, DRV_decomp | Exact MN10300 / AM33 table-driven length+annotation helper for empirical checks |
| `step15c_deep_decode.py` | 167 | BOOT.BIN, DRV_decomp | PART 1: Find ALL callers of 0x40028ea7 (or function starting near it) & inspect 0x40028e50..0x40028f40 |
| `step16_finalize_trace.py` | 158 | BOOT.BIN, DRV_decomp | 1. DRV: Sense table 0x400de35c & MechaErr table around 0x400e8c1e |
| `step17_finalize_links.py` | 134 | BOOT.BIN, DRV_decomp | strip any weird surrogate/private chars |
| `step18_complete_proof.py` | 111 | BOOT.BIN, DRV_decomp | 1. BOOT strings in 0x44034100..0x44034f50 |
| `step19_prove_both.py` | 109 | BOOT.BIN, DRV_decomp, Live Logs | 1. Check all .log / .txt captures in scratch for BGO / X(8002 / normal mode |
| `step20_exact_proof.py` | 118 | BOOT.BIN, DRV_decomp | 1. DRV: Sense 0x38 usage at 0x40012826 & callers of 0x40014434 / 0x400154df |
| `step21_final_proof.py` | 167 | BOOT.BIN, DRV_decomp, Live Logs | 1. Inspect normal_boot_90s.log first 35 lines |
| `step22_verify_hypotheses.py` | 104 | BOOT.BIN, DRV_decomp | 1. BOOT: BGO inner init 0x44001c40..0x44001d4d & close tray / startup drive 0x44009ba0..0x44009dd0 |
| `step23_trace_proof.py` | 95 | BOOT.BIN, DRV_decomp | BOOT: pre-normal mode, ATAPI packet helper 0x44009b11, tray/start retries 0x44009dd0..9ee0 |
| `step24_eeprom_and_laser.py` | 87 | BOOT.BIN, DRV_decomp | 1. DRV: EEPROM byte helpers, 0x400d71e3/0x400d7378, 0x40085171, 0x400c6fe0..0x400c7040 |
| `step25_audit_recovery_and_diag.py` | 100 | BOOT.BIN, DRV_decomp | 1. BOOT: mode pin/switch checks, FEonly callee 0x44004b06, pre-Loader F/E uncompress, [nfrom] recovery 0x44008bc0.. |
| `step26_modes_and_drive_diag.py` | 125 | BOOT.BIN, DRV_decomp, PANA_DVD.FRM | 1. BOOT: 0x44006440..0x44006580 (pre-0x440065bc), 0x4400bb00..0x4400bc80 (0x4400bb14), 0x4400f5b0..0x4400f700 (GPIO |
| `step27_audit_diag_and_modes.py` | 115 | BOOT.BIN, DRV_decomp, PANA_DVD.FRM | 0. Workspace / eh58 metadata |
| `step28_deep_diag_map.py` | 129 | BOOT.BIN, DRV_decomp, PANA_DVD.FRM, Live Logs | 1. Summarize capturas/captura_live_20261112_144101.log key events (unique non-repeating lines) |
| `step29_uart_and_bootmodes.py` | 108 | BOOT.BIN, DRV_decomp | 1. BOOT: 0x44009340..0x44009480 (serial boot / conveyer boot check!), 0x44007f80..0x440080d0 (0x44007fa1 / 0x440080 |
| `step30_complete_diag_map.py` | 167 | BOOT.BIN, DRV_decomp | 1. BOOT.BIN checks |
| `step31_decode_all_gaps.py` | 147 | BOOT.BIN, DRV_decomp | 1. BOOT: find exact start of "[SDCD]This SD Card" and "*** CD diag boot mode ***" and "SD init failed" |
| `step32_atapi_and_cli_entry.py` | 127 | DRV_decomp | 1. Inspect 0x400df500..0x400df5e4 & full 20-byte CLI table 0x400df5d4..0x400dfa80 |
| `step33_uart_hw_and_cdb_lut.py` | 95 | DRV_decomp | 1. Dump 0x400de960..0x400deb90 |
| `step34_final_facts.py` | 111 | BOOT.BIN, DRV_decomp | 1. CDB[0] -> LUT[cdb] -> 3B / 2B / tbl[idx] |
| `step35_boot_fmu_sdcd.py` | 118 | BOOT.BIN, DRV_decomp | 1. BOOT wait 1 second / timer correspond initialize (0x4400b340..0x4400b420) |
| `step36_rx_and_sdcd.py` | 77 | BOOT.BIN | Check UART RX & TX in BOOT.BIN (0x44004760..0x44004860) and callers of any RX func! |
| `step37_verify_all_for_docs.py` | 107 | BOOT.BIN, DRV_decomp, PANA_DVD.FRM | 1. BOOT RX wrapper & callers + fvu.exe + fio_recv |
| `step38_doc_facts_bundle.py` | 238 | BOOT.BIN, DRV_decomp, DRV.BIN, COM10/Pico, MAIN.BIN, Live Logs | SECTION 1: File hashes & headers |
| `step39_exact_callers.py` | 96 | BOOT.BIN, DRV_decomp, DRV.BIN, MAIN.BIN | BOOT.BIN exact rel32 callers |
| `step40_check_rx_and_fmu.py` | 62 | BOOT.BIN, DRV_decomp | Callers in BOOT 0x44004790..0x44004930 |
| `step41_verify_mn103_rel.py` | 98 | BOOT.BIN, DRV_decomp | Check BOOT key targets with exact MN10300 PC=cur_insn (va + disp) |
| `step42_drv_exact_entries.py` | 96 | BOOT.BIN, DRV_decomp | walk backward to previous fc df / cf 10 or 0xf2 fc df |
| `step43_final_micro_checks.py` | 82 | BOOT.BIN, DRV_decomp | Scan for prologues: cf <mask> 3f or cf <mask> f8 fe or df/fc df boundaries |
| `step44_sense_japanese_and_bootflags.py` | 74 | BOOT.BIN, DRV_decomp | 1. DRV Error Code -> Shift-JIS Table around 0x400e8900..0x400e8ed0 |
| `step45_error_dict_and_sdcd_start.py` | 77 | BOOT.BIN, DRV_decomp | 1. Scan full DRV Error Dictionary around 0x400e7e00..0x400e8ed0 |
| `step46_decode_remaining.py` | 62 | BOOT.BIN, DRV_decomp, DRV.BIN, MAIN.BIN | 1. DEV.BIN & MAIN.BIN structure |
| `step47_final_decode.py` | 157 | BOOT.BIN, DRV_decomp | 1. BOOT.BIN: CD diag -> Drive start failed -> reboot & FC CC 0x44035602 |
| `step48_verify_callers.py` | 100 | BOOT.BIN, DRV_decomp | 1. BOOT UART RX callers (0x4400479f, 0x4400488a) & 0x44007e40 exit block & 0x4400bcc2/0x4400b52e |
| `step49_final_check.py` | 82 | BOOT.BIN, DRV_decomp | 1. DRV console I/O helpers |
| `step50_check.py` | 48 | BOOT.BIN, DRV_decomp | Check exact callers of 0x40028c87 / 0x40028d01 (the Sense LUT formatter) and 0x40028ea5 (MechaErr formatter) |

## 3. Scripts Utilitários de CAD Vetorial, Renderização PDF, Bancada e Auditoria (65 scripts)

> Todos os PNGs válidos produzidos por estes scripts de recorte e inspeção já foram curados em [`docs/assets/images/re_dmr_eh55/`](../assets/images/re_dmr_eh55/) e indexados no manifesto MCP [`visual_manifest.json`](../assets/visual_manifest.json).

| Script Utilitário (`scratch/`) | Linhas | Alvos Inspecionados | Finalidade |
| :--- | :---: | :--- | :--- |
| `annotate_pcb.py` | 25 | Service Manual PDF | Grid coordinates: |
| `check_CF_exact.py` | 35 | Service Manual PDF | p63 bottom half |
| `check_bins_and_parts.py` | 44 | PANA_DVD.FRM, Service Manual PDF | Check where non-FF or non-00 transitions happen, and check last 64KB |
| `check_p63_p55_rails.py` | 49 | Service Manual PDF | NA p.{pno} rect={pg.rect} rot={pg.rotation} |
| `check_parts_list.py` | 9 | Service Manual PDF | Vol 1, Page 91 |
| `collect_metrics.py` | 54 | BOOT.BIN, DRV_decomp, DRV.BIN, PANA_DVD.FRM, MAIN.BIN | 1. Exact binary sizes and SHA-256 hashes (non-destructive) |
| `crop_ic7404_pins.py` | 7 | Workspace/Scratch | IC7404 is around x: 500..750, y: 150..400 |
| `debug_pdf_vectors.py` | 54 | Service Manual PDF | Note: in unrotated PDF coords vs visual, drawings & text words share the same unrotated coordinate space except tex |
| `deep_manual_and_bin_check.py` | 49 | PANA_DVD.FRM, Service Manual PDF | 1. IC74xx / IC75xx / EEPROM / IC55xxx / TEST MODE IN MANUALS |
| `dump_ic7401_info.py` | 13 | Service Manual PDF | Vol 1, Page 68 |
| `dump_ide_drive_pages.py` | 11 | Service Manual PDF | VOL1 PAGE {p} |
| `dump_manual_diag_pages.py` | 11 | Service Manual PDF | VOL1 PAGE {p} |
| `dump_p117.py` | 9 | Service Manual PDF | Vol 2 Page 117 text |
| `extract_key_manual_pages.py` | 39 | Service Manual PDF | Render schematic/block diagram pages at high DPI for visual inspection |
| `find_digital_ics.py` | 16 | Workspace/Scratch | {os.path.basename(f)} ({len(doc)} pp |
| `find_grid.py` | 10 | Service Manual PDF | Let's inspect the text layout on page 74 to find the grid lines A, B, C and 1, 2, 3 |
| `find_hw_facts.py` | 14 | Workspace/Scratch | STEP 13362 |
| `find_pcb_layouts_and_zoom.py` | 26 | Service Manual PDF | 1. Print TOC/section headers of vol1 and vol2 around pages 50..97 and 110..177 |
| `generate_re_dossiers.py` | 920 | BOOT.BIN, DRV_decomp, DRV.BIN, PANA_DVD.FRM, COM10/Pico, MAIN.BIN, Live Logs | 1. Generate Inventory Rows cleanly from real files |
| `guided_recovery_monitor.py` | 46 | COM10/Pico, Live Logs | Guided recovery monitor for DMR-EH55 via Pico on COM10. |
| `inspect_eh80v_and_wires.py` | 43 | Service Manual PDF | 1. Inspect eh80v p.14, 15, 28, 67, 75, 111, 120, 133 |
| `inspect_ez485v.py` | 15 | Workspace/Scratch | Script auxiliar de análise |
| `inspect_pcbs_and_logs.py` | 75 | Service Manual PDF | 1. Check V2 p.96 Address Information for P7501 / connectors |
| `inspect_prev_and_rails.py` | 36 | Service Manual PDF | PLANNER RESPONSES 14500..15005 |
| `inspect_tail.py` | 26 | Workspace/Scratch | Script auxiliar de análise |
| `investigate_state.py` | 39 | Live Logs | WORKSPACE & PANADIGA FILES |
| `list_pngs.py` | 19 | Workspace/Scratch | Script auxiliar de análise |
| `make_overview.py` | 29 | Service Manual PDF | Let's render page 74 (Section 1/4) at 150 DPI |
| `prepare_err_translations.py` | 459 | DRV_decomp | Verify exact 431 contiguous 0x34 records from 0x400e3736 to 0x400e8e8e |
| `read_mid_hist.py` | 12 | Workspace/Scratch | \n=== USER [{idx}] ===\n{txt[:1000]} |
| `read_pcb_and_history.py` | 59 | Service Manual PDF | 1. Read exact text around P7501 on NA p.76 and V2 p.96, and NA p.62 & p.64 & V2 p.151/157/173 |
| `read_user_hist.py` | 10 | Workspace/Scratch | Script auxiliar de análise |
| `render_b3.py` | 16 | Service Manual PDF | Grid B-3: x around 200..400, y around 320..480 |
| `render_disasm.py` | 6 | Service Manual PDF | Script auxiliar de análise |
| `render_exploded.py` | 7 | Service Manual PDF | Script auxiliar de análise |
| `render_ic7404.py` | 16 | Service Manual PDF | Find rect of IC7404 |
| `render_ic7404_hires.py` | 12 | Service Manual PDF | Search for IC7404 |
| `render_map.py` | 7 | Service Manual PDF | Script auxiliar de análise |
| `render_more_manuals.py` | 27 | Service Manual PDF | Script auxiliar de análise |
| `render_p32.py` | 7 | Service Manual PDF | Script auxiliar de análise |
| `render_p74.py` | 7 | Service Manual PDF | Script auxiliar de análise |
| `render_strip.py` | 9 | Service Manual PDF | Script auxiliar de análise |
| `render_timer_all_rails.py` | 33 | Service Manual PDF | Render NA p62 right side (IC7501 pins 1..25 & top rail feeding R7506/R7569/Q7513/Q7511) |
| `render_zoom_ic7404.py` | 26 | Service Manual PDF | Let's render page 74 at 400 DPI for ultra clear details |
| `scan_all_bins.py` | 20 | Workspace/Scratch | Script auxiliar de análise |
| `scan_all_service_manuals.py` | 14 | Workspace/Scratch | Script auxiliar de análise |
| `scan_ez485v_strings.py` | 44 | PANA_DVD.FRM | Inspect panasonic_dmr_ez485v_flash_dump_4mb.bin and Panadiga PANA_DVD/hdd for any plaintext or compressed strings |
| `scan_manuals_and_user.py` | 46 | Service Manual PDF | 1. Print USER_INPUT steps between 12000 and 12700 verbatim from transcript_full.jsonl |
| `search_digital_pcb.py` | 19 | Service Manual PDF | \n=== {os.path.basename(p)} |
| `search_ic7404.py` | 19 | Service Manual PDF | Script auxiliar de análise |
| `search_manuals_deep.py` | 22 | Service Manual PDF | \n=== {os.path.basename(p)} ({len(doc)} pages |
| `search_service_manuals.py` | 26 | Service Manual PDF | Scanning {fname} |
| `slice_b3.py` | 15 | Workspace/Scratch | Slices in 2x2 grid |
| `summarize_inventory.py` | 54 | Workspace/Scratch | Script auxiliar de análise |
| `summarize_steps_out.py` | 47 | Workspace/Scratch | Extract section headings & key numbers from scratch text outputs and script groups. |
| `test_jigboot_cd.py` | 21 | COM10/Pico | LOG OUTPUT |
| `trace_fp50004.py` | 10 | Service Manual PDF | Rect around FP50004 |
| `trace_ic7404_and_manuals.py` | 45 | Service Manual PDF | 1. Extract all IC lines from vol1 parts list (pages 89..97) |
| `trace_net_coords.py` | 15 | Service Manual PDF | Trace vector lines & net labels on vol2 Main Net 1/4..4/4 and Timer 1/4..4/4 |
| `trace_nets_vector.py` | 90 | Service Manual PDF | Notice pg.get_text('words') ALREADY matches pg.get_drawings() coords! (Look: R7570 at 572.6, 685.7 and segs max x=7 |
| `trace_sw_uart_xreset_5r2v.py` | 50 | Service Manual PDF | 1. Search ALL pages of NA manual for X_SW_5R2V, SW_5R2V, 5R2V, JC_REG5V, NSW5V, SW_UART |
| `trace_tl7401.py` | 12 | Service Manual PDF | Search for TL7401 |
| `vector_net_p117.py` | 67 | Service Manual PDF | Print segments originating near P59001 pins (x=88..105, y=260..315) |
| `verify_all_facts_for_dossier.py` | 290 | BOOT.BIN, DRV_decomp, DRV.BIN, PANA_DVD.FRM, MAIN.BIN, Live Logs | 1. File Hashes & Cross-Generation Correlation |
| `verify_part2.py` | 168 | BOOT.BIN, DRV_decomp | 1. Exact 62 DRV CLI commands using <8sIII |
