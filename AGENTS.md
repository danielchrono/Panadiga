# Panadiga — Panasonic DIGA Engineering & Forensics Ecosystem
## Universal AI Agent System Directives & Codebase Conventions

Bem-vindo ao repositório de engenharia reversa, recuperação de falhas críticas (destaque para o erro **`U99`**), restauração de firmware, reconstrução forense de storage (**MEIHFS**) e telemetria **UART assíncrona `P7501` (`115200 8N1`)** para a linha de gravadores Panasonic DIGA (foco no DMR-EH55).

Qualquer agente de IA que opere nesta base de código **DEVE** seguir rigorosamente as regras, invariantes físicas e convenções documentadas abaixo.

---

## 1. Visão Arquitetural do Ecossistema

O repositório está estruturado em subsistemas integrados e desacoplados:

1. **Rust Core Workspace & CLI (`crates/`, `Cargo.toml`)**:
   - **`panadiga-core`** (`crates/panadiga-core`): Núcleo de alta performance para parsing forense de storage (`hdd`), criptoanálise (`crypto`), reparo cirúrgico de NVRAM (`nvram`), ponte serial/CDC RP2040/RP2350 (`pico`), servidor GDB RSP introspectivo (`debugger`), gerador de scripts Ghidra (`ghidra`), catálogo visual e SSOT de hardware (`ssot`).
   - **`panadiga-rag`** (`crates/panadiga-rag`): Motor RAG Zero-Error nativo em Rust com 6 camadas de defesa, guardrails determinísticos de segurança física (`safety.rs`) e indexação híbrida (Markdown docs, 30 manuais PDF via `lopdf`, fontes Rust/C11/PIO/TS/Python e scripts Ghidra).
   - **`panadiga-mcp`** (`crates/panadiga-mcp`): Servidor MCP nativo em Rust (`panadiga mcp` via JSON-RPC 2.0 `stdio`) com **13 tools**, **resources** (`panadiga://invariants`, `panadiga://memory-map`, pinouts, catálogo visual e docs/) e **prompts** guiados.
   - **`panadiga-cli`** (`crates/panadiga-cli`): Binário unificado multiplataforma (`panadiga` / `panadiga.exe`) com CLI Clap, modo TUI interativo e lançador do workbench.
2. **Desktop & Mobile Workbench (`apps/desktop/`, `apps/desktop/src-tauri/` / `panadiga-desktop`)**:
   - Interface gráfica **100% livre de mocks em produção** construída com **Tauri 2**, **React 18**, **TypeScript**, **Tailwind CSS**, **Vitest** e **Playwright** (mocks isolados estritamente em `apps/desktop/src/test/mocks/` para suítes de teste).
   - **Alvo Desktop**: Windows (x64), Linux (`.deb` / `tar.gz`), macOS (ARM64) via binário `panadiga-desktop`.
   - **Alvo Mobile**: Android APK (`aarch64-linux-android`, crate `[lib] name = "panadiga"` permanente) com HAL nativa em Rust + bridge USB-OTG / BLE Nordic UART Service sem necessidade de mutação de `Cargo.toml`.
   - Fornece módulos visuais para visualização do HDD MEIHFS, cirurgia de NVRAM Spansion com captura ao vivo via Pico (`CDDIAG`/`DUMP`/`pico://nvram-live-capture`), telemetria do Pico e terminal serial desacoplado.
3. **Firmware C11 & Sonda Pico (`firmware/pico-p7501/`)**:
   - Firmware C11 modular para Raspberry Pi Pico (RP2040) e Pico 2 (RP2350) com suporte a rádio sem fio (Pico W / Pico 2W via CYW43439).
   - Implementa a interface verificada **UART assíncrona `P7501` (`115200 8N1` + controle discreto `SW_UART` / `XRESET` / proteção `NSW5V` + PIO `p7501_uart.pio`)** e servidor GDB Remote Serial Protocol (RSP).
4. **Automação, Ghidra & Ferramentas de Bancada (`tools/`, `scripts/`)**:
   - Orquestração cross-platform de build (`scripts/build.py`), validação (`scripts/validate.py`), tipagem estrita (`mypy --strict`) e testes em `scripts/tests/`.
   - Scripts Ghidra Jython (`tools/ghidra/`) gerados a partir da SSOT Rust e lançadores em `tools/`.
5. **Acervo de Engenharia & Manuais (`manuals_and_schematics/`, `docs/`)**:
   - 15 manuais de serviço oficiais esquematizados e indexados para busca automatizada.
   - 13 dossiês técnicos bilíngues canônicos em `docs/pt-BR/` e `docs/en-US/`.

---

## 2. Invariantes de Segurança Físicas e Forenses (NÃO NEGOCIÁVEIS)

Ao propor scripts, comandos de bancada ou modificações de firmware:

1. **Invariante de Tensão Máxima & Proteção `NSW5V` (`P7501`)**:
   - No conector **`P7501`** do DMR-EH55 (UART `115200 8N1` na placa `VEP79138`), os sinais lógicos (`SW_UART` Pin 1, `XRESET` Pin 2 Open-Drain, `UARTM2P_T` Pin 3, `GND` Pin 4, `UARTP2M_T` Pin 6) e a Flash `IC5001` (`TSOP-48`) operam em **3.3V LVTTL**; já o Pino 5 (`NSW5V` / `GP3`) provém de `IC1506` (`~5.15V` upstream via `D7510`, medido ~`3.29V` em repouso) e **DEVE permanecer SEMPRE como `Hi-Z Input` sem pulls (NUNCA dirigir push-pull!)**. Aplicar 5V ou sinais push-pull indevidos destrói o SoC Matsushita (`AM34`) permanentemente.
2. **Invariante de Alimentação da Porta P7501 (`TO JIGU`)**:
   - **NUNCA ALIMENTE A SONDA PICO PELO CONECTOR P7501**. O pino 1 (`SW_UART` / Sense no P7501) destina-se apenas ao controle de boot/detecção ($< 10\text{ mA}$). O Pico deve receber alimentação externa via USB ou `VSYS`.
3. **Invariante de Escrita em Memória Flash**:
   - A memória Flash Spansion `S29JL064H70TFI00` (8 MB) possui mapeamento de barramento físico de `0x80000000` a `0x807FFFFF`.
   - Região do Bootloader/IPL (`0x80000000` a `0x807EFFFF`): **LEITURA ESTRITA**. Qualquer tentativa de escrita deve ser terminantemente rejeitada com erro `E05`.
   - Única região autorizada para escrita de sanitização de erro `U99`: Os últimos 64 KB de NVRAM/EEMUL (`0x807F0000` a `0x807FFFFF`, offset físico `0x7F0000` a `0x7FFFFF`).
4. **Invariante Forense de Storage (MEIHFS)**:
   - Dumps brutos de disco rígido (como `panasonic DMR EH-55 Firmware hdd_100.bin`) são evidências forenses imutáveis.
   - NUNCA editar ou sobrescrever dumps originais no local (*in-place*). Qualquer extração ou reconstrução deve gerar cópias de trabalho e validar somas de verificação SHA-256 contra os manifestos auditados.
   - O MBR proprietário reside em LBA 0 identificado pela assinatura ASCII `"SH"`. O setor espelho idêntico reside exatamente em 28 MB (LBA 57344). O journal de 10 blocos reside em 8 MB (LBA 16384).

---

## 3. Matriz de Agentes Especialistas (.agents/agents)

| Subagente | Arquivo de Definição | Especialidade / Escopo |
| :--- | :--- | :--- |
| **`panadiga-architect`** | `.agents/agents/panadiga-architect.md` | Orquestração cross-stack, decisões arquiteturais, integridade do sistema |
| **`panadiga-storage-forensics`** | `.agents/agents/panadiga-storage-forensics.md` | Engenharia reversa MEIHFS, parsing de MBR "SH", extração de partições de 32MB/64MB |
| **`panadiga-firmware-re`** | `.agents/agents/panadiga-firmware-re.md` | Engenharia reversa Matsushita AM34, scripts Ghidra, criptoanálise CPRM e entropia |
| **`panadiga-hardware-embedded`** | `.agents/agents/panadiga-hardware-embedded.md` | Firmware RP2040/RP2350 C11, UART Assíncrona P7501 (115200 8N1), GDB RSP e BLE |
| **`panadiga-desktop-ui`** | `.agents/agents/panadiga-desktop-ui.md` | Interface Tauri 2 (Desktop e Android APK), React, TypeScript, testes Vitest e Playwright |
| **`panadiga-qa-auditor`** | `.agents/agents/panadiga-qa-auditor.md` | Guardião dos 9 Quality Gates, conformidade de tipos (`mypy --strict`), tolerância zero a duplicação (`jscpd`) |

---

## 4. O Portão de Qualidade Obrigatório (9 Quality Gates)

Antes de considerar qualquer alteração concluída ou pronta para commit, execute o script de validação universal:

```bash
npm run validate
# ou diretamente:
python scripts/validate.py
```

O portão valida de forma automatizada:
1. **Rust Workspace (5 crates)**: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets` & `cargo test --workspace --verbose`.
2. **TypeScript Strict**: `npm run typecheck` em `apps/desktop` com zero erros.
3. **Vitest Frontend**: `npm test` em `apps/desktop` (**47 testes unitários e de componentes**).
4. **Detector de Duplicação (jscpd)**: `.jscpd.json` com limiar estrito de **0% de duplicação** (0 clones permitidos entre `apps/desktop/src`, `apps/desktop/src-tauri/src`, `crates`, `firmware/pico-p7501`, `tools`, `scripts`).
5. **Linter de UX (uxlint)**: Auditoria completa de estados de interface.
6. **Playwright E2E**: Testes ponta a ponta com Chromium headless (**10 testes**).
7. **Ghidra Native Generator**: Exportação idempotente de scripts a partir da SSOT em Rust (`cargo run -p panadiga-cli -- ghidra export-all`).
8. **Pico C11 Host Test**: Compilação e execução dos testes unitários do servidor GDB RSP com C11 strict warnings.
9. **Python Typer/CLI & Pytest**: Verificação `py_compile`, `mypy --strict scripts` e `pytest scripts/tests`.

---

## 5. Acervo de Documentação Técnica

| Localizador | Documento Canônico (Português) | Canonical Document (English) |
| :--- | :--- | :--- |
| **Portal Geral** | [`docs/INDEX.md`](docs/INDEX.md) | [`docs/INDEX.md`](docs/INDEX.md) |
| **Arquitetura & Blueprint** | [`docs/pt-BR/ARQUITETURA_E_BLUEPRINT_DE_ENGENHARIA.md`](docs/pt-BR/ARQUITETURA_E_BLUEPRINT_DE_ENGENHARIA.md) | [`docs/en-US/SYSTEM_ARCHITECTURE_AND_ENGINEERING_BLUEPRINT.md`](docs/en-US/SYSTEM_ARCHITECTURE_AND_ENGINEERING_BLUEPRINT.md) |
| **Android APK & Bluetooth** | [`docs/pt-BR/ARQUITETURA_MOBILE_ANDROID_E_BLUETOOTH.md`](docs/pt-BR/ARQUITETURA_MOBILE_ANDROID_E_BLUETOOTH.md) | [`docs/en-US/ANDROID_APK_AND_BLUETOOTH_ARCHITECTURE.md`](docs/en-US/ANDROID_APK_AND_BLUETOOTH_ARCHITECTURE.md) |
| **Testes & Conformidade** | [`docs/pt-BR/ARQUITETURA_DE_TESTES_E_CONFORMIDADE.md`](docs/pt-BR/ARQUITETURA_DE_TESTES_E_CONFORMIDADE.md) | [`docs/en-US/TESTING_ARCHITECTURE_AND_COMPLIANCE.md`](docs/en-US/TESTING_ARCHITECTURE_AND_COMPLIANCE.md) |
| **Bancada & Esquemático** | [`docs/pt-BR/BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md`](docs/pt-BR/BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md) | [`docs/en-US/BLUEPRINT_BENCH_WIRING_AND_SCHEMATIC.md`](docs/en-US/BLUEPRINT_BENCH_WIRING_AND_SCHEMATIC.md) |
| **MEIHDFS & Storage** | [`docs/pt-BR/ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md`](docs/pt-BR/ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md) | [`docs/en-US/STORAGE_ARCHITECTURE_MEIHDFS_PARTITIONS.md`](docs/en-US/STORAGE_ARCHITECTURE_MEIHDFS_PARTITIONS.md) |
| **Criptografia PANA_DVD** | [`docs/pt-BR/DOSSIE_CRIPTOGRAFIA_E_FIRMWARE_PANA_DVD.md`](docs/pt-BR/DOSSIE_CRIPTOGRAFIA_E_FIRMWARE_PANA_DVD.md) | [`docs/en-US/DOSSIER_CRYPTOGRAPHY_AND_FIRMWARE_PANA_DVD.md`](docs/en-US/DOSSIER_CRYPTOGRAPHY_AND_FIRMWARE_PANA_DVD.md) |
| **Ghidra & Servidor GDB** | [`docs/pt-BR/ESTRATEGIA_INTEGRACAO_GHIDRA_GDB.md`](docs/pt-BR/ESTRATEGIA_INTEGRACAO_GHIDRA_GDB.md) | [`docs/en-US/GHIDRA_GDB_INTEGRATION_STRATEGY.md`](docs/en-US/GHIDRA_GDB_INTEGRATION_STRATEGY.md) |
| **Montagem & Sonda JST** | [`docs/pt-BR/GUIA_MONTAGEM_E_FOTOS_PROBE_BANCADA.md`](docs/pt-BR/GUIA_MONTAGEM_E_FOTOS_PROBE_BANCADA.md) | [`docs/en-US/BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md`](docs/en-US/BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md) |
| **Inventário & Hashes** | [`docs/pt-BR/INVENTARIO_TECNICO_E_GUIA_DE_OPERACAO.md`](docs/pt-BR/INVENTARIO_TECNICO_E_GUIA_DE_OPERACAO.md) | [`docs/en-US/TECHNICAL_INVENTORY_AND_OPERATIONS_MANUAL.md`](docs/en-US/TECHNICAL_INVENTORY_AND_OPERATIONS_MANUAL.md) |
| **Auditoria & Modularização**| [`docs/pt-BR/RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md`](docs/pt-BR/RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md) | [`docs/en-US/FINAL_AUDIT_AND_MODULARIZATION_REPORT.md`](docs/en-US/FINAL_AUDIT_AND_MODULARIZATION_REPORT.md) |
| **Logging & Telemetria** | [`docs/pt-BR/SUBSISTEMA_LOGGING_E_TELEMETRIA.md`](docs/pt-BR/SUBSISTEMA_LOGGING_E_TELEMETRIA.md) | [`docs/en-US/LOGGING_AND_TELEMETRY_SUBSYSTEM.md`](docs/en-US/LOGGING_AND_TELEMETRY_SUBSYSTEM.md) |
| **Teoria U99 & Semicondutores**| [`docs/pt-BR/TEORIA_E_PRATICA_UNBRICK_U99_E_CRIPTOGRAFIA_MATSUSHITA.md`](docs/pt-BR/TEORIA_E_PRATICA_UNBRICK_U99_E_CRIPTOGRAFIA_MATSUSHITA.md) | [`docs/en-US/THEORY_AND_PRACTICE_U99_UNBRICK_MATSUSHITA_CRYPTO.md`](docs/en-US/THEORY_AND_PRACTICE_U99_UNBRICK_MATSUSHITA_CRYPTO.md) |
| **RAG Engine & Defesa** | [`docs/pt-BR/ZERO_ERROR_RAG_ENGINE_E_SEGURANCA_FISICA.md`](docs/pt-BR/ZERO_ERROR_RAG_ENGINE_E_SEGURANCA_FISICA.md) | [`docs/en-US/ZERO_ERROR_RAG_ENGINE_AND_PHYSICAL_SAFETY.md`](docs/en-US/ZERO_ERROR_RAG_ENGINE_AND_PHYSICAL_SAFETY.md) |
