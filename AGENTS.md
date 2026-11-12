# Panadiga — Panasonic DIGA Engineering & Forensics Ecosystem
## Universal AI Agent System Directives & Codebase Conventions

Bem-vindo ao repositório de engenharia reversa, recuperação de falhas críticas (destaque para o erro **`U99`**), restauração de firmware, reconstrução forense de storage (**MEIHFS**) e instrumentação do protocolo **Matsushita DWire32A / OCD** para a linha de gravadores Panasonic DIGA (foco no DMR-EH55).

Qualquer agente de IA que opere nesta base de código **DEVE** seguir rigorosamente as regras, invariantes físicas e convenções documentadas abaixo.

---

## 1. Visão Arquitetural do Ecossistema

O repositório está estruturado em subsistemas integrados e desacoplados:

1. **Rust Core Engine & CLI (`src/`, `Cargo.toml`)**:
   - Núcleo de alta performance para parsing forense de storage (`hdd.rs`), criptoanálise (`crypto.rs`), reparo cirúrgico de NVRAM (`nvram.rs`), ponte serial RP2040/RP2350 (`pico.rs`), servidor GDB RSP introspectivo (`gdb_bridge.rs`), gerador de scripts Ghidra (`ghidra.rs`) e inspeção de cabeçalhos de firmware (`headers.rs`).
   - Binário compilado: `panadiga.exe` (ou `panadiga`).
2. **Desktop & Mobile Workbench (`apps/desktop/`)**:
   - Interface gráfica construída com **Tauri 2**, **React 18**, **TypeScript**, **Tailwind CSS**, **Vitest** e **Playwright**.
   - **Alvo Desktop**: Windows (x64), Linux (`.deb` / `tar.gz`), macOS (ARM64).
   - **Alvo Mobile**: Android APK (`aarch64-linux-android`) com HAL nativa em Rust (BLE Nordic UART Service / USB CDC OTG) sem necessidade de código Kotlin.
   - Fornece módulos visuais para visualização do HDD MEIHFS, cirurgia de NVRAM Spansion, telemetria do Pico e terminal serial desacoplado.
3. **Firmware C11 & Emulador PIO (`dwire32a_jtag_research/`)**:
   - Máquina de estados PIO (`dwire32a.pio`) e firmware C11 para Raspberry Pi Pico (RP2040) e Pico 2 (RP2350) com suporte a rádio sem fio (Pico W / Pico 2W via CYW43439).
   - Implementa o protocolo Matsushita DWire32A síncrono half-duplex (SCLK 1.0 MHz, temporização de nanossegundos local) e servidor GDB Remote Serial Protocol (RSP).
4. **Tools & RAG Engine (`tools/`, `tools/rag_engine/`)**:
   - CLI administrativo em Python com **Typer** e tipagem estrita (`mypy --strict`).
   - Mecanismo RAG (`rag_engine`) com guardrails determinísticos de segurança física (`safety_invariants.py`).
   - Servidor MCP nativo (`panadiga_mcp_server.py`) para orquestração por modelos de IA.
5. **Acervo de Engenharia & Manuais (`manuals_and_schematics/`, `docs/`)**:
   - 15 manuais de serviço oficiais esquematizados e indexados para busca automatizada.
   - 13 dossiês técnicos bilíngues canônicos em `docs/pt-BR/` e `docs/en-US/`.

---

## 2. Invariantes de Segurança Físicas e Forenses (NÃO NEGOCIÁVEIS)

Ao propor scripts, comandos de bancada ou modificações de firmware:

1. **Invariante de Tensão Máxima de Bancada**:
   - NUNCA exceder **3.3V** nos pinos do conector de depuração `P7501` (CN55001) da placa principal `VEP79138`. O SoC Matsushita UniPhier opera em 3.3V LVTTL. Conectar tensões de 5V ou superiores causa destruição permanente do silício.
2. **Invariante de Alimentação da Porta J (P7501)**:
   - **NUNCA ALIMENTE A SONDA PICO PELO CONECTOR P7501**. O pino 6 (VDD 3.3V) é destinado unicamente ao sensoriamento de presença de nível lógico ($< 10\text{ mA}$). A sonda Pico (especialmente Pico W em transmissão BLE/Wi-Fi) consome até $300\text{ mA}$, o que colapsaria o barramento do gravador e desarmaria o supervisor de reset (`XRST`). O Pico deve receber alimentação externa (5V no `VSYS` ou USB).
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
| **`panadiga-hardware-embedded`** | `.agents/agents/panadiga-hardware-embedded.md` | Firmware RP2040/RP2350 PIO C11, protocolo DWire32A OCD, GDB RSP, Bluetooth BLE e pinagem P7501 |
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
1. **Rust Workspace**: `cargo check --workspace --all-targets` & `cargo test --workspace --verbose` (**41 testes unitários e de integração**).
2. **TypeScript Strict**: `npm run typecheck` em `apps/desktop` com zero erros.
3. **Vitest Frontend**: `npm test` em `apps/desktop` (**33 testes unitários e de componentes**).
4. **Detector de Duplicação (jscpd)**: Limiar estrito de **0% de duplicação** (0 clones permitidos).
5. **Linter de UX (uxlint)**: Auditoria completa de estados de interface (100/100).
6. **Playwright E2E**: Testes ponta a ponta com Chromium headless (**8 testes**).
7. **Ghidra Native Generator**: Exportação idempotente de scripts a partir da SSOT em Rust (`panadiga ghidra export-all`).
8. **Pico C11 Host Test**: Compilação e execução dos testes unitários do servidor GDB RSP com GCC `-Werror` (**7 assertions**).
9. **Python Typer CLI & Pytest**: Verificação com `mypy --strict` e execução do pytest (**27 testes**).

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
