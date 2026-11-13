# Invariantes de Segurança de Hardware e Bancada (Panadiga)

## 1. Conector de Serviço P7501 (UART Assíncrona 115200 8N1) — Placa VEP79138
- **Tensão Limite**: 3.3V LVTTL nos pinos lógicos do SoC `AM34` / `MN103S`.
- **Perigo**: Aplicação de 5V ou sinais push-pull em `XRESET` / `NSW5V` danifica o supervisor de reset e o SoC Matsushita `AM34` irreversivelmente.
- **Pinagem Verificada `DMR-EH55 P7501` (`B6B-ZR-3R4` 6 pinos — Interface UART 115200 8N1)**:
  - Pino 1: `SW_UART` (`GP26/ADC0`) — `LOW` força `serial boot` (`XFWE`, corta `Q7514`); `Hi-Z` habilita TX UART normal e medição VDD.
  - Pino 2: `XRESET` (`GP5`) — Reset em dreno aberto (**Open-Drain apenas**, nunca Push-Pull).
  - Pino 3: `UARTM2P_T` (`GP4`) — TX Pico -> RX AM34 via transistor `Q7514` (`R7570 10kΩ` pull-up; requer `SW_UART=Hi-Z` para conduzir).
  - Pino 4: `GND` (`Pin 38`) — Referência comum `0.0Ω` obrigatória.
  - Pino 5: `NSW5V` (`GP3`) — **SEMPRE `Hi-Z Input` sem pulls (NUNCA dirigir push-pull!)** — originado em `IC1506` (`~5.15V` upstream via `D7510`, ~`3.29V` estático em placa).
  - Pino 6: `UARTP2M_T` (`GP2`) — RX Pico <- TX AM34 `115200 8N1` (`R7572 10kΩ`).

## 2. Memória Flash Spansion S29JL064H (IC5001 - TSOP-48)
- **Capacidade**: 8 MB (64 Megabits, 8.388.608 bytes).
- **Mapeamento de Barramento AM34**: `0x80000000` a `0x807FFFFF`.
- **Faixa de IPL/Bootloader Protegida**: `0x80000000` a `0x807EFFFF` (Offset `0x000000` a `0x7EFFFF`). **ESCRITA PROIBIDA**.
- **Faixa de NVRAM / EEMUL**: `0x807F0000` a `0x807FFFFF` (Offset `0x7F0000` a `0x7FFFFF` = 65.536 bytes). **Permitida apenas para reset cirúrgico de U99 (preenchimento com 0xFF)**.
