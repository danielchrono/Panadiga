# Planejamento de Engenharia: Suporte ao RP2350 (Pico 2) e Integração GDB / Ghidra via `P7501`

**Documento de Arquitetura e Roadmap Técnico**  
**Subsistema:** Interface UART Assíncrona `P7501` (`115200 8N1`), Ponte GDB RSP e Ferramental de Bancada  
**Target:** Panasonic DMR-EH55 (`AM34` / `MN103S`, Syscon `IC7501`, Flash `IC5001`)  
**Host Controllers:** Raspberry Pi Pico (RP2040) & Raspberry Pi Pico 2 (RP2350)  

---

## 1. Avaliação Técnica Comparativa: RP2040 vs RP2350

| Parâmetro de Silício | Raspberry Pi Pico (RP2040) | Raspberry Pi Pico 2 (RP2350) | Impacto Prático na Engenharia Reversa do DMR-EH55 |
| :--- | :--- | :--- | :--- |
| **Núcleos de CPU** | Dual-core ARM Cortex-M0+ @ 133 MHz | Dual-core ARM Cortex-M33 (ou Hazard3 RISC-V) @ 150 MHz | Execução mais rápida de parsers, captura serial em `115200 8N1` e decodificadores em C. |
| **Memória SRAM Interna** | 264 KB SRAM dividida em 6 bancos | **520 KB SRAM** dividida em 10 bancos | **Vantagem Crítica:** O RP2350 permite manter em buffer local na RAM do Pico um setor completo de 64 KB da Spansion S29JL064H (`EEMUL`) ou buffers estendidos de captura/telemetria sem perda de quadros na USB. |
| **Subsistema PIO** | 2 blocos PIO (8 state machines no total) | **3 blocos PIO (12 state machines no total)** | Permite rodar o motor PIO `P7501` (`p7501_uart.pio`) em um bloco PIO e dedicar outros blocos para monitoramento/sniffing de linhas de boot (`SW_UART`, `XRESET`). |
| **Instruções PIO** | Conjunto base PIO | PIO 2.0 (com instruções estendidas e FIFO aprimorada) | O código PIO atual (`p7501_uart.pio`) é 100% compatível com RP2040 e RP2350. |
| **Tensão de I/O** | 3.3V Estrito (Pino 5 `NSW5V` sempre `Hi-Z`) | 3.3V Estrito (Pino 5 `NSW5V` sempre `Hi-Z`) | **Segurança Elétrica Total:** Os sinais lógicos de `P7501` operam em 3.3V LVTTL, enquanto `GP3` (`NSW5V`) permanece sempre como entrada `Hi-Z` sem pulls. |
| **Tolerância e Proteção** | Padrão CMOS | Hardware Glitch Protection & Secure Boot | Maior robustez contra ruídos espúrios induzidos pelas fontes chaveadas do gravador de DVD. |

---

## 2. Esclarecimento Técnico Crucial: Adaptadores JTAG / FTDI Passivos vs. Sonda Pico `P7501`

> [!WARNING]
> **Por que firmwares JTAG genéricos (`pico-dirtyJtag`, `UrJTAG`) ou adaptadores USB-Serial comuns de 3 fios NÃO bastam no conector `P7501`:**
> 
> * **Não há JTAG IEEE 1149.1 nem interface síncrona em `P7501`:** Os esquemáticos oficiais da placa `VEP79138` confirmam que o conector **`P7501` (`TO JIGU`)** não expõe `TDI`/`TDO`/`TMS`/`TCK`. Tratar suas linhas como JTAG causa contenção elétrica ou falha de comunicação.
> * **Interface Verificada (`P7501`):** Trata-se de uma porta **UART Assíncrona (`115200 8N1`)** multiplexada por hardware discreto (`Q7510`, `Q7514`, `QR7503`): Pino 1 (`SW_UART`), Pino 2 (`XRESET` Open-Drain), Pino 3 (`UARTM2P_T`), Pino 4 (`GND`), Pino 5 (`NSW5V` — manter `Hi-Z`) e Pino 6 (`UARTP2M_T`).
> * **Vantagem da Sonda Pico (`p7501_uart.pio`):** Adaptadores seriais passivos sem controle sincronizado de `SW_UART` (`GP26`) e `XRESET` (`GP5`) não entram em `serial boot mode` nem evitam o corte de `Q7514` durante TX; além disso, a proteção em software mantém `GP3` (`NSW5V`) estritamente em `Hi-Z Input`.

---

## 3. Especificação Mecânica e Comercial do Conector `P7501`

* **Padrão Oficial do Conector:** **JST ZR de 6 pinos (Pitch de 1.5 mm)**, Part Number oficial Panasonic **`B6B-ZR-3R4`** (*Top Entry*), identificado com a legenda *"TO JIGU"*.
* **Espaçamento entre Pinos:** `1.5 mm` (centro a centro). Não confundir com conectores JST SH (1.0 mm), JST PH (2.0 mm) ou JST XH (2.54 mm).
* **Guia de Busca Comercial:**  
  * Buscar por: `Cabo JST ZR / ZH 1.5mm 6 pinos Fêmea para Dupont 2.54mm Macho` (alojamento fêmea `ZHR-6` com terminais `SZH-002T-P0.5`).
* **Vantagem de Bancada:** O uso desse chicote pronto com conector de 1.5mm elimina qualquer necessidade de microsolda na placa digital do aparelho, conectando o Pico de forma plug-and-play e segura.

---

## 4. Roadmap de Integração em 3 Fases

### Fase 1: Suporte Dual-Target RP2040 / RP2350 no Firmware C (Imediata)
* **Objetivo:** Permitir compilar o firmware nativo para ambas as arquiteturas sem alterações no código fonte C.
* **Entregáveis:**
  * Atualização do `CMakeLists.txt` com target configurável (`-DPICO_BOARD=pico` ou `-DPICO_BOARD=pico2`).
  * Orquestração out-of-tree via `scripts/build.py --pico` (`npm run build:pico`) e fusão binária nativa via `panadiga pico merge-uf2`.
  * Geração dos artefatos UF2:
    * `panasonic_p7501_uart_rp2040.uf2`
    * `panasonic_p7501_uart_rp2350.uf2`
    * `panasonic_p7501_uart_unified.uf2`

### Fase 2: Implementação do Servidor GDB Remote Serial Protocol (RSP) no Pico
* **Objetivo:** Conectar ferramentas de depuração de alto nível do PC (como o GNU Debugger `mn10300-elf-gdb`) diretamente à ponte serial do Pico através da porta USB CDC.
* **Arquitetura da Ponte:**
  ```text
  [ mn10300-elf-gdb ]  <--- Protocolo GDB RSP (TCP/Serial) --->  [ Pico RP2040/RP2350 ]  <--- UART 115200 8N1 (P7501) --->  [ AM34 / MN103S ]
  ```
* **Comandos GDB Suportados na Ponte:**
  * `?` : Query halt status (consulta estado de parada/sessão ativa na sonda).
  * `g` / `G` : Leitura e escrita do banco de registradores AM34 (D0..D3, A0..A3, SP, PC, MDR, PSW).
  * `m` / `M` : Leitura e escrita de palavras de 32 bits na memória SRAM e espaço de periféricos (`0x80000000`).
  * `c` / `s` : Continue / Single-step (passo único de instrução).

### Fase 3: Ponte TCP <-> Serial Nativa em Rust & Telemetria Contínua
* **Objetivo:** Eliminar intermediários JTAG genéricos incompatíveis (como OpenOCD ARM) e prover uma ponte determinística de baixa latência integrada diretamente à UI Tauri e ao GDB Multiarch (`set architecture mn10300`).
* **Estratégia:**
  * Encaminhamento bidirecional direto USB CDC / BLE NUS via `panadiga pico gdb-bridge` na porta TCP `127.0.0.1:3333`.
  * Ring buffer introspectivo de pacotes RSP (`crates/panadiga-core/src/pico/gdb_bridge.rs`) para inspeção em tempo real na interface desktop.
  * Gerador nativo de scripts Ghidra Jython (`tools/ghidra/`) derivado da SSOT em Rust.
