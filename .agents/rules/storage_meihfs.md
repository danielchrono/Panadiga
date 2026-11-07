# Invariantes Forenses de Storage MEIHFS (Panadiga)

## 1. Identificação de MBR Proprietário e Setores Chave
- **LBA 0 (Setor 0, Offset 0x000000)**: MBR proprietário Panasonic. Deve conter a tag ASCII `"SH"` nos bytes `0x0000-0x0001` e a Disk GUID.
- **LBA 57344 (Offset 28 MB / 0x01C00000)**: Setor espelho de redundância idêntico ao LBA 0.
- **LBA 16384 (Offset 8 MB / 0x00800000)**: Bloco de gerenciamento de journal de 10 blocos de alocação de vídeo e metadados.
- **Partições de Firmware**:
  - Offset 32 MB (`0x02000000`): Partição de Firmware Principal (Assinatura mágica: `51 87 13 8D 5B 0A FB 22`).
  - Offset 64 MB (`0x04000000`): Partição de Firmware Secundária / Recovery (Mesma assinatura mágica).

## 2. Preservação de Dados
- Toda extração de partição deve gerar o arquivo correspondente e atualizar o manifesto `SHA256SUMS.txt`.
- Nunca truncar ou modificar partições brutas sem validação criptográfica.
