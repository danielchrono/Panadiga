---
name: panadiga-pico-flashing
description: >-
  Compilação e gravação do firmware C11 e máquina PIO DWire32A no Raspberry Pi Pico para depuração de bancada do Panasonic DMR-EH55.
  Use esta skill para construir o firmware do Pico, gravar arquivo UF2 ou diagnosticar conexão serial de depuração.
---

# Construção e Gravação de Firmware do Raspberry Pi Pico (DWire32A)

Esta skill guia a compilação do firmware do Raspberry Pi Pico que atua como emulador de hardware do protocolo Matsushita DWire32A.

## Pré-requisitos
- Raspberry Pi Pico conectado via cabo micro-USB.
- Pico SDK configurado (ou binário pré-compilado disponível em `dist_bin/`).

## Procedimento de Gravação (Drag-and-Drop)
1. Pressione e mantenha pressionado o botão **BOOTSEL** no Raspberry Pi Pico ao conectá-lo à porta USB do computador.
2. O Pico aparecerá como uma unidade de armazenamento removível (`RPI-RP2`).
3. Copie o binário compilado `.uf2` para a unidade:
```powershell
Copy-Item "dist_bin/panadiga_pico_dwire32a.uf2" -Destination "E:\" # (substitua E:\ pela letra da unidade montada)
```
4. A unidade reinicializará automaticamente e iniciará a comunicação serial COM no computador.

## Verificação e Telemetria
No Panadiga Desktop Workbench, acesse a aba **Pico Bench** para:
- Detectar a porta serial COM.
- Testar a linha de sinal e pinagem DWire32A.
- Executar o protocolo de Cold Break-In.
