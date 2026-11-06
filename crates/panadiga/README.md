# panadiga — Panasonic DIGA Engineering Suite

`panadiga` é uma suíte de engenharia reversa de sistemas, análise forense de armazenamento e utilitários de firmware desenvolvida especificamente para a família de gravadores digitais **Panasonic DIGA (DMR-EH55)**.

---

## 🚀 Capacidades

- **`nvram` (Sanitização da Flash Spansion S29JL064H / TSOP-48):**
  - Isolamento dos 64 KB da área NVRAM/EEMUL (`0x007F0000` a `0x007FFFFF`).
  - Geração de backup automático (`nvram_corrupt_backup.bin`).
  - Limpeza cirúrgica com `0xFF` para eliminação do travamento irreversível de inicialização **`U99`**.
  - Auditoria de integridade imutável dos 8,32 MB do bootloader via hashes SHA-256 e MD5.
- **`hdd` (Engenharia Reversa do MEIHDFS):**
  - Mapeamento e auditoria estrutural do LBA 0 (tag ASCII `"SH"`) e tabela espelho em 28 MB (LBA 57344).
  - Análise dos descritores de alocação física (LBAs 24..31) e decodificação one's complement (XOR `0xFF`).
  - Inspeção dos 10 blocos de journaling em 8 MB e marcadores de transição em 24 MB.
  - Extração cirúrgica das partições de firmware de 32 MB e 64 MB com manifesto `SHA256SUMS.txt`.
- **`frm` (Criptoanálise do Contêiner Óptico PANA_DVD.FRM):**
  - Cálculo de entropia de Shannon de ponto flutuante ($7.99958\text{ bits/byte}$).
  - Detecção de vazamento de cifras simétricas de bloco em modo ECB (Known Plaintext Attack).
  - Correlação cruzada com os offsets e palavras nulas do disco rígido.
  - Dissecção das entradas da Tabela de Conteúdo (TOC) de 48 bytes e extração do dicionário/codebook.
- **`headers` (Inspeção de Cabeçalhos e Hex Dump):**
  - Exibição formatada em Hexadecimal + ASCII dos blocos críticos do sistema (32 MB, 64 MB e FRM).
  - Validação da assinatura mágica Matsushita (`51 87 13 8D 5B 0A FB 22`).
  - Ferramenta de hex dump arbitrário para qualquer arquivo binário em qualquer offset e comprimento.
- **`scan` (Varredura Forense de Strings em Memória Mapeada):**
  - Extração zero-copy de strings ASCII utilizando `memmap2`.
  - Filtro contra dicionário forense proprietário (`panasonic`, `diga`, `mn2ds`, `am34`, `meihdfs`, `u99`, etc.).
  - Suporte a exportação de relatório estruturado em JSON.
- **`manuals` (Indexador e Busca em Manuais de Serviço PDF):**
  - Varredura de múltiplos PDFs simultâneos via parser nativo `lopdf`.
  - Busca automatizada de componentes e códigos de erro (`S29JL064`, `MN2DS`, `CN55001`, `U99`, `SERVICE MODE`, etc.).

---

## 📦 Como Usar

### Compilação
```bash
cargo build --release
```

### Comandos Principais
```bash
# 1. Limpeza da NVRAM para recuperar erro U99
cargo run --release -- nvram clean DMR_EH55_ORIGINAL.bin -o DMR_EH55_CLEAN.bin

# 2. Inspeção do HD MEIHDFS com gráfico de densidade
cargo run --release -- hdd inspect "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin" --density

# 3. Extração das partições do HD
cargo run --release -- hdd extract --outdir firmware_and_images/extracted_partitions

# 4. Decodificação dos descritores de alocação física do MEIHDFS
cargo run --release -- hdd decode-descriptors

# 5. Criptoanálise do PANA_DVD.FRM
cargo run --release -- frm analyze firmware_and_images/PANA_DVD.FRM --hdd "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"

# 6. Inspeção de cabeçalhos de firmware
cargo run --release -- headers

# 7. Varredura forense de strings
cargo run --release -- scan --limit 50

# 8. Busca em manuais de serviço técnicos
cargo run --release -- manuals --filter dmr-eh55p9
```
