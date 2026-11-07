# Utilitários de Baixo Nível e Automação de Bancada (`tools/`)

Este diretório contém utilitários de suporte, depuradores interativos e scripts de automação para trabalho de engenharia reversa e bancada no Panasonic DMR-EH55.

---

## 1. Scripts de Conexão GDB & Depuração Interativa

* 🔌 **[`iniciar_ponte_gdb.bat`](iniciar_ponte_gdb.bat):**  
  Varre automaticamente as portas seriais conectadas buscando o Raspberry Pi Pico (RP2040 ou RP2350) e inicia o servidor ponte TCP-para-Serial GDB RSP na porta `3333`.

* 🐛 **[`conectar_gdb_multiarch.bat`](conectar_gdb_multiarch.bat):**  
  Dispara o GNU Debugger (`gdb-multiarch` ou `mn10300-elf-gdb`) pré-configurado com a arquitetura `mn10300` e conecta automaticamente à ponte local `target remote 127.0.0.1:3333`.

* ⚡ **[`executar_openocd.bat`](executar_openocd.bat):**  
  Inicializa o OpenOCD carregando a configuração oficial [`openocd_panasonic_am34.cfg`](../dwire32a_jtag_research/openocd_panasonic_am34.cfg) para automações de leitura e regravação de memória.

---

## 2. Scripts de Engenharia Reversa para Ghidra

* 🗺️ **[`ghidra_am34_memory_map.py`](ghidra_am34_memory_map.py):**  
  Script Python para o Ghidra que cria automaticamente todos os blocos de memória da placa do gravador (`INTERNAL_SRAM`, `SPANSION_FLASH`, `NVRAM_EEMUL`, `ASIC_MMIO`, `ATA_IDE_MMIO`), permitindo ao descompilador C resolver ponteiros físicos de hardware.

* 🏷️ **[`ghidra_importar_simbolos_panadiga.py`](ghidra_importar_simbolos_panadiga.py):**  
  Importa automaticamente os vetores de reset do processador AM34, comandos da Flash Spansion, micro-setores da NVRAM (`SA134` a `SA141` onde ocorre a corrupção do erro U99), registradores UniPhier MMIO e barramento PATA/IDE do disco rígido MEIHDFS.

* 🔗 **[`ghidra_conectar_debugger.py`](ghidra_conectar_debugger.py):**  
  Instrui e auxilia a conexão do Ghidra Debugger (perspectiva interativa) diretamente ao servidor GDB RSP na porta `3333`, permitindo depurar o processador em tempo real enquanto navega no código descompilado.

---

## 3. Utilitários Hexadecimais & Análise Binária

* 🔍 **[`Abrir_HxD.bat`](Abrir_HxD.bat):**  
  Abre o editor hexadecimal HxD 64-bit/32-bit portátil integrado no repositório para inspeção visual imediata de dumps e partições.
