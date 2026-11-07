# Invariantes de Segurança de Hardware e Bancada (Panadiga)

## 1. Conector de Depuração In-Circuit CN55001 (Placa VEP79138)
- **Tensão Limite**: 3.3V LVTTL.
- **Perigo**: O barramento é conectado diretamente ao SoC Matsushita UniPhier MN2DS0016 / AM34. Aplicação de 5V ou sinais descontrolados queima as portas do SoC irreversivelmente.
- **Pinos Críticos**:
  - Pino 1: GND (Referência comum obrigatória).
  - Pino 2: DWIRE_DATA (Bidirecional half-duplex síncrono).
  - Pino 3: DWIRE_CLK (Clock de sincronismo).
  - Pino 4: RESET# (Active LOW, pull-up externo de 10k para 3.3V).
  - Pino 5: 3.3V (VCC Sensing).

## 2. Memória Flash Spansion S29JL064H (IC55007 - TSOP-48)
- **Capacidade**: 8 MB (64 Megabits, 8.388.608 bytes).
- **Mapeamento de Barramento AM34**: `0x80000000` a `0x807FFFFF`.
- **Faixa de IPL/Bootloader Protegida**: `0x80000000` a `0x807EFFFF` (Offset `0x000000` a `0x7EFFFF`). **ESCRITA PROIBIDA**.
- **Faixa de NVRAM / EEMUL**: `0x807F0000` a `0x807FFFFF` (Offset `0x7F0000` a `0x7FFFFF` = 65.536 bytes). **Permitida apenas para reset cirúrgico de U99 (preenchimento com 0xFF)**.
