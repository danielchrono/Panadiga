# Utilitários de Baixo Nível e Automação de Bancada (`tools/`)

Este diretório contém utilitários Windows de bancada (`.cmd`), editor hexadecimal e scripts de automação **Ghidra** em [`ghidra/`](ghidra/) para engenharia reversa no **Panasonic DMR-EH55**.

> **Nota Arquitetural:** Os comandos CLI forenses, servidor RAG Zero-Error e servidor nativo **Model Context Protocol (MCP)** agora executam diretamente no binário único em Rust (`panadiga`). Consulte [`cargo run -- --help`](../README.md).

---

## 1. Scripts de Conexão GDB & Depuração Interativa (Windows)

* 🔌 **[`iniciar_ponte_gdb.cmd`](iniciar_ponte_gdb.cmd):**  
  Varre automaticamente as portas seriais conectadas buscando o Raspberry Pi Pico (RP2040 ou RP2350) e inicia a ponte TCP-para-Serial GDB RSP na porta `127.0.0.1:3333`.

* 🐞 **[`conectar_gdb_multiarch.cmd`](conectar_gdb_multiarch.cmd):**  
  Dispara o GNU Debugger (`gdb-multiarch` ou `mn10300-elf-gdb`) pré-configurado com a arquitetura `mn10300` e conecta automaticamente à ponte local `target remote 127.0.0.1:3333`.

---

## 2. Scripts de Engenharia Reversa para NSA Ghidra ([`tools/ghidra/`](ghidra/))

Os mapeamentos de memória e símbolos são gerados diretamente pela **Single Source of Truth (SSOT)** em Rust via `panadiga ghidra export-all --dir tools/ghidra`:

* 🚀 **[`iniciar_ghidra.cmd`](iniciar_ghidra.cmd):**  
  Localiza a instalação do NSA Ghidra (`GHIDRA_HOME`, `PATH` ou diretórios padrão) ou abre `tools\ghidra\` no Windows Explorer.

* 🗺️ **[`ghidra/ghidra_am34_memory_map.py`](ghidra/ghidra_am34_memory_map.py):**  
  Cria os blocos físicos de memória do processador Matsushita AM34 / MN103S (`INTERNAL_SRAM`, `SPANSION_FLASH`, `NVRAM_EEMUL`, `ASIC_MMIO`, `ATA_IDE_MMIO`) no projeto Ghidra.

* 🏷️ **[`ghidra/ghidra_importar_simbolos_panadiga.py`](ghidra/ghidra_importar_simbolos_panadiga.py)** / **[`ghidra_import_symbols_panadiga.py`](ghidra/ghidra_import_symbols_panadiga.py):**  
  Importa vetores de exceção AM34, micro-setores Spansion (`SA134..SA141`, região U99), periféricos MMIO e registradores PATA/IDE MEIHDFS.

* 🔗 **[`ghidra/ghidra_connect_debugger.py`](ghidra/ghidra_connect_debugger.py):**  
  Exibe instruções interativas para acoplar a perspectiva *Ghidra Debugger* à ponte GDB RSP (`127.0.0.1:3333`) com `set architecture mn10300`.

---

## 3. Utilitários Hexadecimais & Análise Binária (Windows)

* 🔍 **[`Abrir_HxD.cmd`](Abrir_HxD.cmd):**  
  Abre o editor hexadecimal HxD 64-bit portátil priorizando o dump real da Spansion em `firmware_and_images\bkp\panasonic_dmr-eh55_flash_dump.bin`.
