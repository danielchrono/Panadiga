# Scripts Legados em Python (Status: Migrados para Rust)

> [!NOTE]
> Todos os scripts utilitários em Python contidos neste diretório foram integralmente migrados para a suíte nativa em Rust **`panadiga`** (localizada em `crates/panadiga`).
> Recomenda-se utilizar os comandos nativos do Rust para maior velocidade (zero-copy via `memmap2`), auditoria com tipagem estrita e ausência de dependências de runtime do Python.

---

## Tabela de Correspondência: Python $\rightarrow$ Rust (`panadiga`)

| Script Python Legado | Comando Equivalente em Rust (`panadiga`) | Status | Benefícios da Versão em Rust |
| :--- | :--- | :---: | :--- |
| `clean_spansion_nvram.py` | `cargo run -- nvram clean <DUMP.bin> [-o <CLEAN.bin>]` | **Migrado (100%)** | Auditoria estruturada dos 8 micro-setores `SA134..SA141`, backup automático e cálculo formal de SHA-256. |
| `inspect_hdd_image.py` | `cargo run -- hdd inspect [IMAGEM.bin] --density` | **Migrado (100%)** | Zero-copy `memmap2` (sem carregar 100 MB na RAM), gráfico de densidade de blocos em ASCII e relatório JSON. |
| `parse_panasonic_hdd_partitions.py` | `cargo run -- hdd inspect`<br/>`cargo run -- hdd extract --outdir <DIR>` | **Migrado (100%)** | Validação bit-a-bit do MBR mestre e espelho (LBA 57344), journal de alocação e extração de binários limpos com manifesto `SHA256SUMS.txt`. |
| `inspect_firmware_headers.py` | `cargo run -- headers`<br/>`cargo run -- headers dump <FILE> -o <OFF> -l <LEN>` | **Migrado (100%)** | Inspeção dos blocos de 32 MB, 64 MB e FRM com validação de magic signature e ferramenta genérica de hex dump. |
| `analyze_deep.py` | `cargo run -- scan [FILE] [--keyword <KW>] [--limit <N>]` | **Migrado (100%)** | Varredura de strings com `memmap2` ultra-rápida e filtro automatizado de keywords forenses. |
| `search_manuals.py` | `cargo run -- manuals [--dir <DIR>] [--filter <NAME>]` | **Migrado (100%)** | Indexador nativo de PDFs com `lopdf`, sem necessidade de bibliotecas C ou PyMuPDF/fitz. |
