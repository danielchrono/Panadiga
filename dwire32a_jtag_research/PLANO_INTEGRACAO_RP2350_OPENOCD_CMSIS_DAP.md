# Planejamento de Engenharia: Suporte ao RP2350 (Pico 2) e Integração CMSIS-DAP / OpenOCD / GDB

**Documento de Arquitetura e Roadmap Técnico**  
**Subsistema:** Emulação de Hardware, Interface On-Chip Debug (OCD) e Ferramental de Bancada  
**Target:** Panasonic UniPhier `MN2DS0016` (Matsushita `AM34` / `MN103S`)  
**Host Controllers:** Raspberry Pi Pico (RP2040) & Raspberry Pi Pico 2 (RP2350)  

---

## 1. Avaliação Técnica Comparativa: RP2040 vs RP2350

| Parâmetro de Silício | Raspberry Pi Pico (RP2040) | Raspberry Pi Pico 2 (RP2350) | Impacto Prático na Engenharia Reversa do DMR-EH55 |
| :--- | :--- | :--- | :--- |
| **Núcleos de CPU** | Dual-core ARM Cortex-M0+ @ 133 MHz | Dual-core ARM Cortex-M33 (ou Hazard3 RISC-V) @ 150 MHz | Execução mais rápida de parsers e decodificadores de protocolo em C. |
| **Memória SRAM Interna** | 264 KB SRAM dividida em 6 bancos | **520 KB SRAM** dividida em 10 bancos | **Vantagem Crítica:** O RP2350 permite manter em buffer local na RAM do Pico um setor completo de 64 KB da Spansion S29JL064H (`EEMUL`) ou até 8 microsetores de 8 KB simultâneos para verificação de redundância sem round-trips pela USB. |
| **Subsistema PIO** | 2 blocos PIO (8 state machines no total) | **3 blocos PIO (12 state machines no total)** | Permite rodar a máquina de estados DWire32A (`dwire32a.pio`) em um bloco PIO e dedicar outros blocos para analisador lógico de barramento (sniffing de clock/dados a 150 MHz). |
| **Instruções PIO** | Conjunto base PIO | PIO 2.0 (com instruções estendidas e FIFO aprimorada) | O código PIO atual (`dwire32a.pio`) é 100% retrocompatível com RP2350. |
| **Tensão de I/O** | 3.3V Estrito | 3.3V Estrito | **Segurança Elétrica Total:** Ambos operam nativamente a 3.3V, sem necessidade de conversores de nível lógico para a placa `VEP79138`. |
| **Tolerância e Proteção** | Padrão CMOS | Hardware Glitch Protection & Secure Boot | Maior robustez contra ruídos espúrios induzidos pelas fontes chaveadas do gravador de DVD. |

---

## 2. Esclarecimento Técnico Crucial: JTAG Tradicional (IEEE 1149.1) vs Matsushita DWire32A

> [!WARNING]
> **Por que firmwares JTAG genéricos (`pico-dirtyJtag`, `UrJTAG` padrão) NÃO funcionam diretamente no conector CN55001:**
> 
> * **JTAG Clássico (IEEE 1149.1 / 4 Fios):** Utiliza linhas dedicadas unidirecionais em full-duplex: `TDI` (Test Data In), `TDO` (Test Data Out), `TMS` (Test Mode Select) e `TCK` (Test Clock).
> * **Conector CN55001 da Panasonic:** O processador `MN2DS0016` implementa a interface proprietária **Matsushita DWire32A**, que possui apenas **uma linha de dados física (`SDATA`)** operando em **Half-Duplex bidirecional síncrono**, sincronizada pela linha mestre `SCLK`.
> * **Consequência Elétrica:** Se um firmware JTAG padrão tentar dirigir `TDI` ou `TDO` fixos contra `SDATA`, ocorrerá **colisão de barramento (bus contention)** e perda total de sinal. A alternância dinâmica entre TX e RX na mesma via exige o controle estrito de turnaround em hardware fornecido pelo nosso `dwire32a.pio`.

---

## 3. Especificação Mecânica e Comercial do Conector CN55001 / P7501

* **Padrão Oficial do Conector:** **JST ZR de 6 pinos (Pitch de 1.5 mm)**, Part Number oficial Panasonic **`B6B-ZR-3R4`** (*Top Entry*), identificado com a legenda *"TO WRITE JIGU"*.
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
  * Atualização dos scripts de automação `build_pico_firmware.ps1` e `build_pico_firmware.bat` adicionando a flag `-Board pico` / `-Board pico2`.
  * Geração dos dois artefatos UF2:
    * `panasonic_dwire32a_rp2040.uf2`
    * `panasonic_dwire32a_rp2350.uf2`

### Fase 2: Implementação do Servidor GDB Remote Serial Protocol (RSP) no Pico
* **Objetivo:** Conectar ferramentas de depuração de alto nível do PC (como o GNU Debugger `mn10300-elf-gdb`) diretamente ao processador AM34 através da porta USB CDC do Pico.
* **Arquitetura da Ponte:**
  ```text
  [ mn10300-elf-gdb ]  <--- Protocolo GDB RSP (TCP/Serial) --->  [ Pico RP2040/RP2350 ]  <--- DWire32A (PIO) --->  [ MN2DS0016 (AM34) ]
  ```
* **Comandos GDB Suportados na Ponte:**
  * `?` : Query halt status (verifica se a CPU está parada via `EXTRG` ou cold break-in).
  * `g` / `G` : Leitura e escrita do banco de registradores AM34 (D0..D3, A0..A3, SP, PC, MDR, PSW).
  * `m` / `M` : Leitura e escrita de palavras de 32 bits na memória SRAM e espaço de periféricos (`0x80000000`).
  * `c` / `s` : Continue / Single-step (passo único de instrução).

### Fase 3: Bridge CMSIS-DAP v2 & Perfil OpenOCD
* **Objetivo:** Permitir que o **OpenOCD** utilize o Pico como adaptador de bancada padronizado.
* **Estratégia:**
  * Incorporar o protocolo CMSIS-DAP v2 (USB Bulk endpoints de alta velocidade com drivers nativos WinUSB no Windows, sem necessidade de driver serial virtual).
  * Implementar *Vendor Extension Commands* (`DAP_Vendor0` a `DAP_Vendor3`) para encapsular pacotes de transação DWire32A.
  * Fornecer script de configuração `openocd_panasonic_am34.cfg` para automatização de leitura e gravação de memória.
