---
name: panadiga-pico-flashing
description: >-
  Compilação e gravação do firmware C11 P7501 Async UART (115200 8N1) no Raspberry Pi Pico (RP2040/RP2350) para bancada do Panasonic DMR-EH55.
  Use esta skill para construir o firmware do Pico, gravar arquivo UF2 ou diagnosticar conexão serial de depuração.
---

# Construção e Gravação de Firmware do Raspberry Pi Pico (P7501 Async UART)

Esta skill guia a compilação do firmware do Raspberry Pi Pico (`P7501>`) que implementa telemetria elétrica Hi-Z e interface UART assíncrona `P7501` (`115200 8N1`) para o Panasonic DMR-EH55.

## Pré-requisitos
- Raspberry Pi Pico conectado via cabo micro-USB (ou RP2350 Pico 2).
- Pico SDK configurado ou binário `.uf2` unificado em `crates/panadiga-core/src/pico/firmware_bins/panasonic_p7501_uart_unified.uf2` (ou compilação via `python scripts/build.py --pico`).

## Procedimento de Gravação (CLI Nativa ou Drag-and-Drop)
1. **Automático via Panadiga CLI**:
```bash
cargo run --quiet -p panadiga-cli --bin panadiga -- pico flash
```
2. **Manual (Drag-and-Drop em modo BOOTSEL)**:
   - Pressione e mantenha pressionado o botão **BOOTSEL** no Raspberry Pi Pico ao conectá-lo à porta USB do computador (`RPI-RP2` ou `RP2350`).
   - Copie o binário compilado `.uf2` para a unidade:
```powershell
Copy-Item "crates/panadiga-core/src/pico/firmware_bins/panasonic_p7501_uart_unified.uf2" -Destination "E:\" # (substitua E:\ pela letra da unidade montada)
```
- A unidade reinicializará automaticamente e iniciará a comunicação serial COM no computador (`P7501>`).

## Verificação e Telemetria
No Panadiga Desktop Workbench, acesse a aba **Pico Bench** para:
- Detectar a porta serial COM.
- Executar matriz elétrica segura `PINCHECK`, `BUSDIAG`, `SNIFF` e `ISOLATE` no conector `P7501`.
- Capturar telemetria de boot UART (`BOOTCAP`, `JIGBOOT`, `UARTMON`, `UARTSEND`, `UARTPROBE`) com o canal P7501 115200 8N1.
