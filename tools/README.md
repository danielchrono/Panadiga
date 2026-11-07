# Utilitários de Baixo Nível e Automação de Bancada (`tools/`)

Este diretório contém utilitários de suporte, depuradores interativos e scripts de automação para trabalho de bancada no Panasonic DMR-EH55.

---

## 1. Scripts de Conexão GDB & Depuração Interativa

* 🔌 **[`iniciar_ponte_gdb.bat`](iniciar_ponte_gdb.bat):**  
  Varre automaticamente as portas seriais conectadas buscando o Raspberry Pi Pico (RP2040 ou RP2350) e inicia o servidor ponte TCP-para-Serial GDB RSP na porta `3333`.

* 🐛 **[`conectar_gdb_multiarch.bat`](conectar_gdb_multiarch.bat):**  
  Dispara o GNU Debugger (`gdb-multiarch` ou `mn10300-elf-gdb`) pré-configurado com a arquitetura `mn10300` e conecta automaticamente à ponte local `target remote 127.0.0.1:3333`.

* ⚡ **[`executar_openocd.bat`](executar_openocd.bat):**  
  Inicializa o OpenOCD carregando a configuração oficial [`openocd_panasonic_am34.cfg`](../dwire32a_jtag_research/openocd_panasonic_am34.cfg) para automações de leitura e regravação de memória.

* 🐉 **[`ghidra_am34_memory_map.py`](ghidra_am34_memory_map.py):**  
  Script Python para o Ghidra que cria automaticamente os blocos de memória da placa do gravador (`SRAM`, `FLASH`, `NVRAM`, `ASIC_MMIO`, `ATA_IDE_MMIO`), alinhando o descompilador de código C.

---

## 2. Utilitários Hexadecimais

* 🔍 **[`Abrir_HxD.bat`](Abrir_HxD.bat):**  
  Abre o editor hexadecimal HxD configurado para a arquitetura do sistema operacional (32-bit ou 64-bit) para inspeção visual de dumps e partições.
