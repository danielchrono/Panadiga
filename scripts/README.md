# Scripts Legados em Python (Status: Arquivados & 100% Substituídos por Rust)

> [!NOTE]
> Todos os scripts utilitários em Python contidos anteriormente neste diretório foram migrados integralmente para a crate nativa em Rust **`panadiga`** (`crates/panadiga`) e arquivados em `scripts/legacy_python/`.
> A suíte prioriza nativamente código compilado em Rust com acesso zero-copy via `memmap2`, ausência de runtime externo e velocidade em nível de hardware.

---

## Tabela de Equivalência Oficial (Rust Native vs Python Legado)

| Funcionalidade | Implementação Legada (Arquivada) | Comando Oficial Rust (`panadiga`) | Benefícios da Versão Nativa em Rust |
| :--- | :--- | :--- | :--- |
| **Cirurgia NVRAM U99** | `clean_spansion_nvram.py` | `panadiga nvram clean <DUMP.bin> [-o <CLEAN.bin>]` | Auditoria atômica dos 8 micro-setores `SA134..SA141`, backup automático e cálculo formal SHA-256. |
| **Forensic MEIHDFS HDD** | `inspect_hdd_image.py` | `panadiga hdd inspect <IMAGEM.bin> [--density]` | Zero-copy `memmap2` (sem alocar dezenas de megabytes na RAM) e densidade visual em ASCII. |
| **Extração de Partições** | `parse_panasonic_hdd_partitions.py` | `panadiga hdd extract <IMAGEM.bin> [--outdir <DIR>]` | Validação de MBR principal e espelho (LBA 57344), journal e manifesto criptográfico `SHA256SUMS.txt`. |
| **Inspeção de Cabeçalhos** | `inspect_firmware_headers.py` | `panadiga headers dump <FILE> -o <OFF> -l <LEN>` | Validação de magic signatures de 32MB/64MB/FRM e inspeção com alinhamento de LBA. |
| **Varredura Forense** | `analyze_deep.py` | `panadiga scan <FILE> [--keyword <KW>]` | Busca de strings vetorial multithread em dumps brutos sem carregar o arquivo em memória. |
| **Indexação de Manuais** | `search_manuals.py` | `panadiga manuals [--query <QUERY>]` | Motor PDF nativo construído com `lopdf`, sem dependências de CPython ou PyMuPDF. |
| **Scripts para Ghidra** | `ghidra_am34_memory_map.py` | `panadiga ghidra export-map` | Gerado diretamente a partir de `constants.rs` em Rust (Single Source of Truth). |
