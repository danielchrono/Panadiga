# Panadiga — Panasonic DIGA Engineering & Forensics Ecosystem
## Universal AI Agent System Directives & Codebase Conventions

Bem-vindo ao repositório de engenharia reversa, recuperação de falhas críticas (destaque para o erro **`U99`**), restauração de firmware, reconstrução forense de storage (**MEIHFS**) e emulação do protocolo **Matsushita DWire32A / OCD** para a linha de gravadores Panasonic DIGA (foco no DMR-EH55).

Qualquer agente de IA que opere nesta base de código **DEVE** seguir rigorosamente as regras, invariantes físicas e convenções documentadas abaixo.

---

## 1. Visão Arquitetural do Ecossistema

O repositório está estruturado em 5 subsistemas integrados:

1. **Rust Core Engine & CLI (`src/`, `Cargo.toml`)**:
   - Núcleo de alta performance para parsing de storage (`hdd.rs`), criptoanálise (`crypto.rs`), reparo cirúrgico de NVRAM (`nvram.rs`), ponte serial RP2040 (`pico.rs`), gerador de scripts Ghidra (`ghidra.rs`) e inspeção de cabeçalhos de firmware (`headers.rs`).
   - Binário gerado: `panadiga.exe` (ou `panadiga`).
2. **Desktop Laboratory Workbench (`apps/desktop/`)**:
   - Interface gráfica construída com **Tauri 2**, **React 18**, **TypeScript**, **Tailwind CSS**, **Vitest** e **Playwright**.
   - Fornece módulos visuais para visualização do HDD MEIHFS, cirurgia de NVRAM Spansion, telemetria do RP2040 e terminal serial.
3. **Firmware C11 & Emulador PIO RP2040 (`dwire32a_jtag_research/`)**:
   - Máquina de estados PIO (`dwire32a.pio`) e firmware C11 para Raspberry Pi Pico (`dwire32a_pico_main.c`, `gdb_rsp.c`).
   - Implementa o protocolo Matsushita DWire32A síncrono half-duplex e servidor GDB Remote Serial Protocol (RSP).
4. **Tools & RAG Engine (`tools/`, `tools/rag_engine/`)**:
   - CLI administrativo em Python com **Typer** e tipagem estrita (`mypy --strict`).
   - Mecanismo RAG (`rag_engine`) com guardrails determinísticos de segurança física (`safety_invariants.py`).
   - Servidor MCP nativo (`panadiga_mcp_server.py`) para orquestração por modelos de IA.
5. **Acervo de Engenharia & Manuais (`manuals_and_schematics/`, `docs/`)**:
   - 15 manuais de serviço oficiais esquematizados e indexados para busca automatizada e fundamentação técnica.

---

## 2. Invariantes de Segurança Físicas e Forenses (NÃO NEGOCIÁVEIS)

Ao propor scripts, comandos de bancada ou modificações de firmware:

1. **Invariante de Tensão Máxima de Bancada**:
   - NUNCA exceder **3.3V** nos pinos do conector de depuração `CN55001` da placa principal `VEP79138`. O microcontrolador Matsushita UniPhier opera em 3.3V LVTTL. Conectar tensões de 5V ou superiores causa destruição permanente do silício.
2. **Invariante de Escrita em Memória Flash**:
   - A memória Flash Spansion `S29JL064H70TFI00` (8 MB) possui mapeamento de barramento físico de `0x80000000` a `0x807FFFFF`.
   - Região do Bootloader/IPL (`0x80000000` a `0x807EFFFF`): **LEITURA ESTRITA**. Qualquer tentativa de escrita deve ser terminantemente rejeitada com erro `E05`.
   - Única região autorizada para escrita de sanitização de erro `U99`: Os últimos 64 KB de NVRAM/EEMUL (`0x807F0000` a `0x807FFFFF`, offset físico `0x7F0000` a `0x7FFFFF`).
3. **Invariante Forense de Storage (MEIHFS)**:
   - Dumps brutos de disco rígido (como `panasonic DMR EH-55 Firmware hdd_100.bin`) são evidências forenses imutáveis.
   - NUNCA editar ou sobrescrever dumps originais no local (*in-place*). Qualquer extração ou reconstrução deve gerar cópias de trabalho e validar somas de verificação SHA-256 contra os manifestos auditados.
   - O MBR proprietário reside em LBA 0 identificado pela assinatura ASCII `"SH"`. O setor espelho idêntico reside exatamente em 28 MB (LBA 57344). O journal de 10 blocos reside em 8 MB (LBA 16384).

---

## 3. Matriz de Agentes Especialistas (.agents/agents)

Para tarefas complexas, utilize a persona correspondente ou invoque o subagente especializado:

| Subagente | Arquivo de Definição | Especialidade / Escopo |
| :--- | :--- | :--- |
| **`panadiga-architect`** | `.agents/agents/panadiga-architect.md` | Orquestração cross-stack, decisões arquiteturais, integridade do sistema |
| **`panadiga-storage-forensics`** | `.agents/agents/panadiga-storage-forensics.md` | Engenharia reversa MEIHFS, parsing de MBR "SH", extração de partições de 32MB/64MB |
| **`panadiga-firmware-re`** | `.agents/agents/panadiga-firmware-re.md` | Engenharia reversa Matsushita AM34, scripts Ghidra, criptoanálise CPRM e entropia |
| **`panadiga-hardware-embedded`** | `.agents/agents/panadiga-hardware-embedded.md` | Firmware RP2040 PIO C11, protocolo DWire32A OCD, GDB RSP, pinagem TSOP-48 e CN55001 |
| **`panadiga-desktop-ui`** | `.agents/agents/panadiga-desktop-ui.md` | Interface Tauri 2, React, TypeScript, componentes de telemetria, testes Vitest e Playwright |
| **`panadiga-qa-auditor`** | `.agents/agents/panadiga-qa-auditor.md` | Guardião dos 9 Quality Gates, conformidade de tipos (`mypy --strict`), tolerância zero a duplicação (`jscpd`) |

---

## 4. O Portão de Qualidade Obrigatório (9 Quality Gates)

Antes de considerar qualquer alteração concluída ou pronta para commit, execute o script de validação universal:

```powershell
powershell -ExecutionPolicy Bypass -File validate_all.ps1
```

O portão valida de forma automatizada:
1. **Rust Workspace**: `cargo check --workspace --all-targets` & `cargo test --workspace --verbose` (16 testes unitários).
2. **TypeScript Strict**: `npm run typecheck` em `apps/desktop` com zero erros.
3. **Vitest Frontend**: `npm test` em `apps/desktop` (26 testes unitários).
4. **Detector de Duplicação (jscpd)**: Limiar estrito de **0% de duplicação** (0 clones permitidos).
5. **Linter de UX (uxlint)**: Auditoria completa de estados de interface (100/100).
6. **Playwright E2E**: Testes ponta a ponta com Chromium headless (8 testes).
7. **Ghidra Native Generator**: Exportação idempotente de scripts a partir da SSOT em Rust (`panadiga ghidra export-all`).
8. **Pico C11 Host Test**: Compilação e execução dos testes unitários do servidor GDB RSP com GCC `-Werror` (7 assertions).
9. **Python Typer CLI & Pytest**: Verificação com `mypy --strict` e execução do pytest (21 testes).

---

## 5. Model Context Protocol (MCP) & Ferramentas Integradas

O projeto inclui um servidor MCP nativo e configurado em `.mcp.json` e `.agents/mcp_config.json`:
- **Comando do Servidor**: `python tools/panadiga_mcp_server.py`
- **Ferramentas MCP Expostas**:
  - `panadiga_memory_map`: Retorna o mapa de memória oficial do processador Matsushita AM34.
  - `panadiga_inspect_hdd`: Inspeciona arquivos de imagem de disco rígido MEIHFS.
  - `panadiga_clean_nvram`: Executa a limpeza cirúrgica dos 64 KB de NVRAM do dump Spansion.
  - `panadiga_safety_check`: Valida limites elétricos e endereços de barramento contra acidentes.
  - `panadiga_run_quality_gate`: Dispara os testes do portão de qualidade.
  - `panadiga_search_manuals`: Realiza consultas semânticas e literais nos 15 manuais de serviço.
