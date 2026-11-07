---
name: panadiga-nvram-unbrick
description: >-
  Procedimento de cirurgia de NVRAM e recuperação do erro fatal U99 na Flash Spansion S29JL064H (IC55007).
  Use esta skill quando o usuário solicitar reparar o erro U99, limpar os 64 KB de NVRAM ou sanitizar um dump da Flash.
---

# Procedimento Cirúrgico de Recuperação do Erro U99

O erro `U99` ocorre devido à corrupção na emulação de EEPROM (EEMUL/NVRAM) armazenada nos últimos 64 KB da memória Flash física Spansion `S29JL064H70TFI00` (8 MB / 8.388.608 bytes).

## Passos de Execução

### Opção 1: Via CLI Rust Nativo (Recomendado)
```bash
cargo run -- nvram clean --input caminho/do/dump_original.bin --output caminho/do/dump_corrigido.bin
```

### Opção 2: Via Script Python Dedicado
```bash
python scripts/clean_spansion_nvram.py caminho/do/dump_original.bin caminho/do/dump_corrigido.bin
```

## O Que a Operação Executa:
1. **Preservação de IPL**: Verifica que os primeiros `8.323.072 bytes` (`0x000000` a `0x7EFFFF`) permanecem **100% idênticos**, calculando e comparando hashes SHA-256.
2. **Backup Cirúrgico**: Extrai os 64 KB corrompidos (`0x7F0000` a `0x7FFFFF`) para `nvram_corrupt_backup.bin`.
3. **Reset de Estado de Fábrica**: Preenche os 64 KB com `0xFF` no arquivo corrigido, forçando o processador UniPhier a recriar a estrutura de NVRAM limpa no próximo boot.
4. **Auditoria de Integridade**: Emite relatório formal com hashes de entrada e saída.
