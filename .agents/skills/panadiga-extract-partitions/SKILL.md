---
name: panadiga-extract-partitions
description: >-
  Executa a análise forense e extração das partições do disco rígido Panasonic (MEIHFS, MBR 'SH', mirrors e partições de firmware de 32MB/64MB).
  Use esta skill para inspecionar dumps de HDD, extrair partições binárias ou validar manifestos SHA-256.
---

# Extração e Análise Forense de Armazenamento MEIHFS

Esta skill documenta o procedimento de mapeamento e extração de imagens de disco Panasonic DIGA.

## Execução via CLI Rust Nativo
```bash
cargo run -- extract --image "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin" --outdir firmware_and_images/extracted_partitions
```

## Execução via Script Python
```bash
python scripts/parse_panasonic_hdd_partitions.py --extract --outdir firmware_and_images/extracted_partitions --json firmware_and_images/extracted_partitions/hdd_analysis_report.json
```

## Estruturas Extraídas e Mapeadas:
1. `mbr_sector0_sh.bin`: Setor LBA 0 (512 bytes) com assinatura `"SH"` e Disk GUID.
2. `mbr_sector57344_mirror.bin`: Setor espelho redundante localizado em 28 MB (LBA 57344).
3. `journal_allocation_8mb.bin`: Bloco de catalogação e journal em 8 MB (LBA 16384).
4. `firmware_32mb_partition.bin`: Partição de firmware primária iniciada em `0x02000000` (Magic: `51 87 13 8D 5B 0A FB 22`).
5. `firmware_64mb_partition.bin`: Partição de firmware secundária/backup em `0x04000000`.
6. `SHA256SUMS.txt`: Manifesto criptográfico gerado automaticamente para garantia de custódia forense.
