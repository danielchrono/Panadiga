# Central de Engenharia Reversa, Firmware e Diagnóstico de Hardware — Panasonic DIGA DMR-EH55

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![SoC](https://img.shields.io/badge/SoC-Matsushita%20MN2DS0016%20(AM34)-blue.svg)]()
[![Flash](https://img.shields.io/badge/NOR%20Flash-Spansion%20S29JL064H%20(8MB)-green.svg)]()
[![Storage](https://img.shields.io/badge/Storage-MEIHDFS%20(IDE%2FPATA)-yellow.svg)]()
[![License](https://img.shields.io/badge/License-Proprietary%20Research%20%2F%20Open%20Engineering-lightgrey.svg)]()

> **Documento Mestre de Referência Técnica de Sistemas e Hardware**  
> **Classificação:** Engenharia Forense / Plataforma Unificada de Recuperação e Ferramentas  
> **Dispositivo-Alvo:** Gravador Digital de Vídeo DVD/HDD **Panasonic DMR-EH55** (Chassi DIGA / PCB Principal `VEP79138`)  
> **Processador:** Matsushita UniPhier `MN2DS0016` (Núcleo RISC de 32 bits `AM34` / `MN103S`)  
> **Memória Flash NOR:** Spansion `S29JL064H70TFI00` (8 MBytes / 64 Mbits, encapsulamento TSOP-48)  
> **Subsistema de Armazenamento:** Disco Rígido PATA/IDE com particionamento e sistema de arquivos proprietário **MEIHDFS**  
> **Interface de Depuração:** Barramento serial síncrono proprietário **Matsushita DWire32A / On-Chip Debug (OCD)** no conector `CN55001`  

---

## 1. Visão Geral e Apresentação Executiva do Projeto

O **Panasonic DIGA DMR-EH55** representa o ápice da engenharia japonesa de eletrônicos de consumo da era 2005–2007. Projetado pela **Matsushita Electric Industrial Co., Ltd.** (atual Panasonic Corporation), o aparelho integra processamento de mídia de altíssimo desempenho baseado no SoC proprietário **UniPhier MN2DS0016**, executando o conjunto de instruções **Matsushita AM34** (família MN10300).

Diferente de sistemas embarcados convencionais (baseados em Linux ou VxWorks genéricos sobre processadores MIPS/ARM), a arquitetura DIGA é **estritamente verticalizada e proprietária**:
1. **Bootloader Híbrido em Duas Fases:** A memória NOR Flash Spansion (`IC55007`, 8 MB) carrega apenas o estágio inicial de inicialização (*IPL - Initial Program Loader*). O sistema operacional principal, drivers de aceleração gráfica OSD, microcódigos do servo óptico e codecs residem em setores reservados nos primeiros 100 MB do disco rígido IDE interno.
2. **Sistema de Armazenamento Proprietário MEIHDFS:** O disco rígido não utiliza partição MBR padrão PC nem sistemas de arquivos FAT/ext. O layout baseia-se no formato **MEIHDFS** (*Matsushita Electric Industrial HDD File System*), com superbloco na assinatura ASCII `"SH"` no LBA 0, espelho de segurança no LBA 57.344 (28 MB), journal transacional de 10 blocos e partições de firmware cifradas.
3. **Criptografia e Proteção de Propriedade Intelectual:** O contêiner de distribuição óptica (`PANA_DVD.FRM`) e as partições executáveis do HD em 32 MB e 64 MB são blindados com cifras de bloco simétricas de 64 bits em modo ECB, associadas a esquemas CPRM (*Content Protection for Recordable Media*) e DivX DRM.
4. **A Vulnerabilidade Crítica do Erro `U99`:** A calibração de canais, estado de montagem do HDD e parâmetros do gravador são mantidos nos últimos 64 KB da Flash Spansion (`0x007F0000` a `0x007FFFFF`), emulando uma EEPROM (`EEMUL`). Desgaste de células, desligamento abrupto durante gravação ou parâmetros corrompidos provocam falha de validação na inicialização. Como consequência, o firmware Matsushita entra em pânico e trava irreversivelmente com o código **`U99`** no display VFD frontal, rejeitando comandos do controle e travando o mecanismo óptico.

Este repositório consolida o **ecossistema definitivo de engenharia reversa, recuperação física e análise forense** do Panasonic DMR-EH55, disponibilizando a suíte de ferramentas de alta performance em Rust **`panadiga`**, interface de bancada desktop e mobile Android (`.apk`), firmware C11/PIO para microcontroladores **Raspberry Pi Pico (RP2040) e Pico 2 (RP2350)** com suporte a **Bluetooth Low Energy (BLE)** e USB-C OTG, documentação técnica dos 15 manuais originais de fábrica e o acervo completo de 13 grandes dossiês técnicos bilíngues de referência.

---

## 2. Diagrama Mermaid de Arquitetura Ponta-a-Ponta

O fluxo abaixo descreve as relações físicas, elétricas e lógicas entre o host de desenvolvimento/bancada, as interfaces do SoC Matsushita, a memória Flash externa e o barramento de armazenamento:

```mermaid
flowchart LR
    subgraph Host["Bancada Técnica & Host de Engenharia"]
        direction TB
        PC["Estação Desktop (Win/Linux/macOS) & Mobile Android (.apk)<br/>HAL Nativa em Rust (Zero Kotlin / Zero WebUSB)"]
        RP2040["Raspberry Pi Pico (RP2040)<br/>Dual Cortex-M0+ @ 133 MHz<br/>Firmware C & Máquina PIO<br/>Transceptor DWire32A Half-Duplex"]
        PROG["Gravador Universal TSOP-48<br/>(XGecu T48 / SVOD 4 / TL866II)<br/>Soquete ZIF v3"]
        PC <-->|USB CDC / UART / Cargo| RP2040
        PC <-->|USB 2.0 High Speed| PROG
    end

    subgraph Interface["Interface de Depuração & Hardware"]
        direction TB
        CN55001["Conector CN55001 (6 Pinos)<br/>Placa Digital VEP79138<br/>R_damp = 100 Ω | R_pu = 4.7 kΩ<br/>Open-Drain em nRESET | SCLK = 1.0 MHz"]
        WP_PAD["Ilha SMD Pino 14 (WP#/ACC)<br/>Hardware Unlock Spansion<br/>Injeção 3.3V via 1 kΩ"]
        RP2040 -->|SDATA, SCLK, EXTRG, nRESET| CN55001
        RP2040 -.->|Sinal 3.3V de Desbloqueio| WP_PAD
    end

    subgraph SoC["SoC Matsushita MN2DS0016 (UniPhier)"]
        direction TB
        OCD["Unidade On-Chip Debug (OCD)<br/>Captura Cold Break-In (32 Pulsos)<br/>DCSR / Breakpoint Trace"]
        AM34["Núcleo Matsushita AM34 RISC<br/>32-bit @ 108 / 162 MHz<br/>Cache L1 I/D (16 KB / 16 KB)"]
        SRAM["SRAM Interna do SoC<br/>Espaço Físico Uncached<br/>Hospedagem do RAM Loader"]
        ATA_CTRL["Controlador PATA / IDE<br/>Registradores MMIO (0xD4000000)<br/>Transferência DMA / UDMA"]
        
        OCD <-->|Depuração & Halt| AM34
        AM34 <-->|Injeção de Microcódigo JEDEC| SRAM
        AM34 <-->|Controle de E/S de Armazenamento| ATA_CTRL
    end

    subgraph Flash["Memória NOR Flash On-Board"]
        direction TB
        SPAN["Spansion S29JL064H (IC55007)<br/>8 MB (64 Mbit) TSOP-48<br/>Barramento 16-bit (Word Mode)"]
        IPL["Setores SA0..SA133 (0x000000..0x7EFFFF)<br/>Initial Program Loader (IPL)<br/>Código MN103S Nativo Uncached<br/>ROOT-OF-TRUST EM TEXTO PLANO"]
        NVRAM["Setores SA134..SA141 (0x7F0000..0x7FFFFF)<br/>8 Micro-setores de 8 KB (64 KB Total)<br/>Partição EEMUL (Calibrações / Parâmetros)<br/>ORIGEM DO ERRO FATAL U99"]
        SPAN --- IPL
        SPAN --- NVRAM
    end

    subgraph Storage["Subsistema de Disco Rígido"]
        direction TB
        HDD["Disco Rígido PATA/IDE (80-500 GB)<br/>ou SATA SSD c/ Adaptador Marvell 88SA8052<br/>Área de Serviço: 100 MB Reservados"]
        LBA0["LBA 0 (Offset 0x00000000)<br/>MBR Proprietário Matsushita<br/>Tag ASCII 'SH' + Disk Volume GUID"]
        MIRROR["LBA 57.344 (Offset 0x01C00000 / 28 MB)<br/>Espelho Bit-a-Bit do LBA 0"]
        JOURNAL["LBA 16.384..19.199 (Offset 0x00800000 / 8 MB)<br/>Journal Transacional (10 Blocos de 128 KB)"]
        PART1["LBA 65.536 (Offset 0x02000000 / 32 MB)<br/>Partição Firmware 1 (0x5E3250 = ~5.89 MB)<br/>Assinatura 51 87 13 8D 5B 0A FB 22<br/>AM34 Executive & Codecs de Mídia"]
        PART2["LBA 131.072 (Offset 0x04000000 / 64 MB)<br/>Partição Firmware 2 (0xB0000 = ~704 KB)<br/>Assinatura 51 87 13 8D 5B 0A FB 22<br/>Microcódigo Sub-sistema & Fontes OSD"]
        MEIHDFS["LBA 204.800+ (Offset 0x06400000+)<br/>Volume Contínuo MEIHDFS<br/>VR_MOVIE.VRO / VR_MANGR.IFO"]
        
        HDD --- LBA0
        HDD --- MIRROR
        HDD --- JOURNAL
        HDD --- PART1
        HDD --- PART2
        HDD --- MEIHDFS
    end

    CN55001 <-->|Protocolo Síncrono DWire32A| OCD
    AM34 <==>|Barramento 16-bit Uncached (0x80000000)| SPAN
    PROG -.->|Dessolda & Regravação Externa| SPAN
    ATA_CTRL <==>|Barramento ATA-66/100 (40 Vias)| HDD
```

---

## 3. Árvore Completa do Repositório e Descrição Semântica dos Arquivos

A topologia de arquivos do repositório foi projetada seguindo as melhores práticas de engenharia de software e hardware, sem acoplamento a caminhos absolutos locais:

```text
Panadiga/
│
├── Cargo.toml                                         <-- Configuração mestre do Workspace Rust (Manifesto raiz unificado: Pacote panadiga + Workspace Tauri)
├── Cargo.lock                                         <-- Árvore de dependências determinísticas compiladas
├── README.md                                          <-- Este documento mestre de engenharia e referência técnica
├── README_ECOSSISTEMA.md                              <-- Visão executiva inicial e resumo de transição
│
├── crates/                                            <-- Ferramental de engenharia de software nativo em Rust
│   └── panadiga/                                      <-- Suíte CLI de alta performance para análise forense e correção
│       ├── Cargo.toml                                 <-- Manifesto do pacote: clap 4.5, memmap2, sha2, md-5, serde
│       └── src/
│           ├── main.rs                                <-- Ponto de entrada CLI (nvram clean, hdd inspect/extract/decode-descriptors, frm analyze/dump-toc/codebook)
│           ├── lib.rs                                 <-- Exportação da API de biblioteca interna
│           ├── constants.rs                           <-- Definição tipada de limites de memória, offsets, tags e codebooks
│           ├── nvram.rs                               <-- Módulo de auditoria e limpeza cirúrgica da Flash Spansion
│           ├── hdd.rs                                 <-- Parser zero-copy memmap2, extrator e decodificador bit-invertido (XOR 0xFF)
│           └── crypto.rs                              <-- Criptoanálise forense, desconstrução de TOC/módulos e Codebook ECB de 64 bits
│
├── docs/                                              <-- Dossiês aprofundados de engenharia e auditoria técnica
│   ├── ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md       <-- Engenharia reversa do disco de 100 MB e layout MEIHDFS
│   ├── BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md          <-- Esquema elétrico, condicionamento de sinais e pinagens
│   ├── DESCOBERTAS_FORENSES_MEIHDFS_ALOCACAO.md       <-- Decodificação forense de descritores One's Complement (XOR 0xFF)
│   ├── DESCONSTRUCAO_CONTAINER_PANA_DVD_FRM.md        <-- Desconstrução do contêiner .FRM, TOC de 48B e Codebook ECB de 64 bits
│   ├── DOSSIE_CRIPTOGRAFIA_E_FIRMWARE_PANA_DVD.md     <-- Criptoanálise do container .FRM, DRM e Root-of-Trust
│   └── INVENTARIO_TECNICO_E_GUIA_DE_OPERACAO.md       <-- Inventário criptográfico SHA-256, rotinas e matriz FMEA
│
├── dwire32a_jtag_research/                            <-- Engenharia reversa da interface On-Chip Debug Matsushita
│   ├── PANASONIC_DWIRE32A_JTAG_RECONSTRUCTION.md      <-- Blueprint completo do protocolo DWire32A / OCD
│   ├── dwire32a.pio                                   <-- Transceptor síncrono Half-Duplex em assembly PIO (RP2040)
│   ├── dwire32a_pico_main.c                           <-- Firmware C do host Raspberry Pi Pico (Cold Break-In)
│   ├── ram_loader.c                                   <-- Driver JEDEC nativo AM34 injetável na SRAM interna
│   └── CMakeLists.txt                                 <-- Script de compilação do Raspberry Pi Pico C/C++ SDK
│
├── firmware_and_images/                               <-- Imagens binárias e dumps brutos do sistema
│   ├── panasonic DMR EH-55 Firmware hdd_100.bin       <-- Dump original de 100 MB do HD IDE (204.800 setores LBA)
│   ├── PANA_DVD.FRM                                   <-- Contêiner oficial cifrado de atualização por mídia óptica
│   ├── UPDATE_EH55.zip                                <-- Pacote compactado oficial de fábrica contendo PANA_DVD.FRM
│   └── extracted_partitions/                          <-- Partições extraídas pela suíte panadiga
│       ├── hdd_mbr_master_lba0.bin                    <-- Setor MBR mestre (512 bytes) com assinatura ASCII "SH"
│       ├── hdd_mbr_mirror_lba57344.bin                <-- Setor espelho de redundância (512 bytes em 28 MB)
│       ├── hdd_allocation_descriptors_0x00003000.bin  <-- Tabela de descritores de blocos (LBAs 24..31, 4096 bytes)
│       ├── hdd_journal_blocks_0x00820000.bin          <-- 10 blocos de 128 KB da área transacional (1.310.720 bytes)
│       ├── hdd_partition_firmware_0x02000000.bin      <-- Partição 1 do Firmware AM34 (payload útil de 6.173.264 bytes)
│       ├── hdd_partition_firmware_0x02000000_sector_aligned.bin <-- Partição 1 alinhada em setores (6.173.696 bytes)
│       ├── hdd_partition_firmware_0x04000000.bin      <-- Partição 2 de Microcódigo/OSD (payload de 720.896 bytes)
│       ├── hdd_analysis_report.json                   <-- Relatório estruturado de metadados em formato JSON
│       └── SHA256SUMS.txt                             <-- Manifesto criptográfico auditado de todos os blocos extraídos
│
├── manuals_and_schematics/                            <-- Biblioteca de 15 Manuais Oficiais e Esquemas Elétricos
│   ├── c2c7f1.pdf                                     <-- Manual de Serviço Completo com Esquemas Elétricos e PCB
│   ├── 9e8a23.pdf                                     <-- Manual de Serviço Vol. 1 (Diagnóstico, U99 e Desmontagem)
│   ├── panasonic_dmr-eh55ec.pdf                       <-- Manual de Serviço Oficial (Europa Continental / Alemanha)
│   ├── panasonic_dmr-eh55ee.pdf                       <-- Manual de Serviço Oficial (Leste Europeu / Rússia)
│   ├── panasonic_dmr-eh55p9_eh55pc9.pdf               <-- Manual de Serviço Oficial (Estados Unidos / Canadá)
│   ├── Panasonic dmr-eh55 installation.pdf            <-- Guia Oficial de Instalação e Atualização de Firmware CD-R
│   ├── 22767039.pdf                                   <-- Guia de Conexões HDMI e VIERA Link (HDAVI Control)
│   ├── 2Panasonic_DIGA_DMR-EH55_DVD_Recorder_User_Manual.pdf <-- Manual de Operação DIGA DMR-EH55 (Inglês)
│   ├── 6097ea.pdf                                     <-- Manual de Operação e Configuração de Gravação XP/SP/LP/EP
│   ├── a5e678.pdf                                     <-- Manual de Operação DMR-EH55 / DMR-EH56 (Edição Europa)
│   ├── cb6cee.pdf                                     <-- Manual de Operação DMR-EH55 (Edição Australásia)
│   ├── dmreh55.pdf                                    <-- Especificações Técnicas e Guia de Referência Rápida
│   ├── Panasonic_DIGA_DMR-EH55_DVD_Recorder_User_Manual.pdf <-- Manual de Operação Completo do Usuário (Global)
│   ├── panasonic_dmr-eh55,dmr-eh56_dvd_user_manual.pdf <-- Manual do Usuário com Tabela de Resolução de Problemas
│   └── philips-dmr-eh55.pdf                           <-- Guia de Manutenção e Chassi Compatível DIGA
│
├── scripts/                                           <-- Utilitários legados em Python (Auditados e Mantidos)
│   ├── clean_spansion_nvram.py                        <-- Sanitizador cirúrgico dos 64 KB de NVRAM da Spansion
│   ├── parse_panasonic_hdd_partitions.py              <-- Parser e extrator de partições MEIHDFS
│   ├── inspect_hdd_image.py                           <-- Mapeador de densidade de blocos de 1 MB
│   ├── inspect_firmware_headers.py                    <-- Inspetor hexadecimal de headers em 32MB, 64MB e FRM
│   ├── analyze_deep.py                                <-- Scanner de strings e assinaturas de alta entropia
│   └── search_manuals.py                              <-- Indexador automatizado de termos técnicos no acervo PDF
│
└── tools/                                             <-- Ferramentas portáteis de análise de baixo nível
    ├── Abrir_HxD.bat                                  <-- Launcher portátil de 1 clique para o HxD Hex Editor
    └── HxD/                                           <-- HxD Hex Editor v2.5 Portable (32-bit e 64-bit)
```

---

### 3.1. Scripts de Inicialização e Build Multiplataforma (Linux, macOS, Windows)

O ecossistema dispõe de automações de ciclo de vida completo compatíveis com Windows (PowerShell/CMD), Linux e macOS:

| Script | Plataformas | Finalidade e Capacidades |
| :--- | :--- | :--- |
| **`build_all.sh`** / **`build_all.ps1`** | Linux, macOS, Windows | **Build Mestre:** Compilação paralela com autodetecção de núcleos de CPU (`nproc`/`sysctl`), cache `sccache`, compilação da CLI Rust, compilação da interface Vite/React e do Tauri Desktop, empacotando em `dist_bin/` e gerando releases `.tar.gz`/`.zip` com hashes SHA-256. |
| **`run.sh`** / **`run.ps1`** | Linux, macOS, Windows | **Inicializador Rápido:** Lança instantaneamente a interface Desktop (`--dev` para hot-reload, `--build` para recompilar), ou atua como atalho para comandos da CLI (`run.sh rag query ...`, `run.sh nvram clean ...`). |
| **`validate_all.sh`** / **`validate_all.ps1`** | Linux, macOS, Windows | **Portão Universal de Qualidade:** Executa os 9 Quality Gates obrigatórios de commit e verificação anti-regressão. |

---

## 4. Guia de Uso Completo da Ferramenta Rust `panadiga` (`src/`)

A ferramenta **`panadiga`** foi desenvolvida para substituir scripts legados por uma implementação industrial em **Rust moderno (edição 2021/2024)**. O software emprega mapeamento de memória em modo zero-copy (`memmap2`), tratamento de erros sem pânico (`anyhow`/`thiserror`), tipagem estrita contra números mágicos (`constants.rs`) e serialização estruturada em JSON.

### 4.1 Compilação em Modo Release

Certifique-se de possuir o compilador Rust instalado (`rustc --version` $\ge 1.85$). Na raiz do repositório, execute:

```bash
cargo build --release
```

O binário altamente otimizado será gerado em:
* Windows: `target/release/panadiga.exe`
* Linux: `target/release/panadiga`

---

### 4.2 Comando 1: Limpeza Cirúrgica da NVRAM (`nvram clean`)

Quando a Flash Spansion `S29JL064H` (8 MB) é lida via gravador físico, este comando realiza a correção imediata do erro **`U99`**:

```bash
cargo run --release -- nvram clean <DUMP_ORIGINAL.bin> [-o <DUMP_LIMPO.bin>]
```

#### Exemplo Prático Real:
```bash
cargo run --release -- nvram clean DMR_EH55_FLASH_ORIGINAL.bin -o DMR_EH55_FLASH_PATCHED.bin
```

#### O que o comando executa rigorosamente:
1. **Validação de Geometria e Suporte Multi-Modelos:** Detecta e suporta automaticamente dumps de **2 MB** (16 Mbit - ex: `DMR-E50`/`E55`), **4 MB** (32 Mbit - ex: `DMR-E85H`/`E95H`), **8 MB** (64 Mbit - ex: `DMR-EH50`/`EH55`/`EH56`/`EH60`/`EH65`) e **16 MB** (128 Mbit - ex: `DMR-EX75`/`EX85`), calculando o offset da NVRAM em `len - 64 KB`.
2. **Extração de Segurança:** Isola os $65.536\text{ bytes}$ da área de calibração/NVRAM (`0x007F0000` a `0x007FFFFF`) e salva em `nvram_corrupt_backup.bin` para auditoria forense.
3. **Auditoria Micro-Setorial:** Analisa a ocupação de dados não-virgens (bytes diferentes de `0xFF`) nos 8 micro-setores protegidos de 8 KB da Spansion Model 00:
   * `SA134` (`0x007F0000`), `SA135` (`0x007F2000`), `SA136` (`0x007F4000`), `SA137` (`0x007F6000`)
   * `SA138` (`0x007F8000`), `SA139` (`0x007FA000`), `SA140` (`0x007FC000`), `SA141` (`0x007FE000`)
4. **Sanitização `0xFF`:** Preenche integralmente os 64 KB com bytes `0xFF` (estado padrão de célula NOR apagada).
5. **Auditoria Criptográfica de Imutabilidade:** Verifica se os $8.323.072\text{ bytes}$ do bootloader e sistema operacional (`0x00000000` a `0x007EFFFF`) permaneceram **100% inalterados bit-a-bit**, emitindo os hashes MD5 e SHA-256 do arquivo antes e depois.

---

### 4.3 Comando 2: Inspeção Estrutural do Disco Rígido (`hdd inspect`)

Realiza a análise profunda do dump de 100 MB do HD interno sem carregar o arquivo na RAM (acesso instantâneo via `memmap2`):

```bash
cargo run --release -- hdd inspect [CAMINHO_IMAGEM.bin] [--density] [--json relatorio.json]
```

#### Exemplo Prático Real:
```bash
cargo run --release -- hdd inspect "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin" --density
```

#### Saída Técnica Produzida:
```text
================================================================================
 PANASONIC DMR-EH55 HDD FORENSIC PARSER & STORAGE ARCHITECTURE AUDITOR          
================================================================================
 Arquivo da Imagem : firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin
 Tamanho Total     :   104857600 bytes (100.00 MB)
 Total de Setores  :      204800 LBAs (512 bytes/setor)
 MD5 Global        : ef4453fc8651903b0a4b0c1873d950d4
 SHA-256 Global    : 9d292d5b46f190775f359e30af11d074021fb321f040bf701d0ecf0f2056db00
--------------------------------------------------------------------------------

[+] ESTRUTURA DO MASTER BOOT RECORD & TABELA ESPELHO (LBA 0 & LBA 57344)
  * LBA 0 (Offset 0x00000000):
    - Assinatura Bruta  : 0000000001005348628C6EF51B2C3666
    - Versao de Formato : Rev 1.0 (0x0001)
    - Magic Tag ASCII   : 'SH' (System Storage Header)
    - Disk Volume GUID  : 628C6EF51B2C3666 (Identificador de Formatacao)
    - MBR Padrao 0x55AA : NAO (Assinatura Proprietaria Panasonic MEIHDFS RAW)
    - Zeros em 16..511  : 100% Zeros limpos
    - SHA-256 LBA 0     : 61162c24c9240408ac807cd4307c9c5833d440715a982126c536aa10aa7530f2
  * LBA 57344 (Offset 0x01C00000 = 28.00 MB):
    - Status do Espelho : IDENTICO BIT-A-BIT AO LBA 0 (Integridade Confirmada)

[+] METADADOS DE ALOCACAO & JOURNAL TRANSACIONAL
  * Descritores Setores 24..31 (0x00003000, 4096 bytes):
    - Espelho em LBA 57368 (0x01C03000): 100% IDENTICO (Integridade Confirmada)
  * LBA 16384 (Offset 0x00800000 = 8.00 MB):
    - Catalogo Journal     : 10 blocos de 128 KB (Transações ativas)
  * LBA 49152 (Offset 0x01800000 = 24.00 MB):
    - Marcador de Transicao: 0C0000000000CA25000085B50000E90F0000010000000100

[+] PARTIÇÕES DE FIRMWARE PROPRIETÁRIAS (Magic: 51 87 13 8D 5B 0A FB 22)
  * Particao 1 (32 MB / LBA 65536): 6.173.264 bytes | Entropia: 7.9990 bits/byte
  * Particao 2 (64 MB / LBA 131072):  720.896 bytes | Entropia: 7.9929 bits/byte
```

---

### 4.4 Comando 3: Extração Cirúrgica de Partições (`hdd extract`)

Extrai as estruturas de metadados, superblocos e as imagens de firmware contidas no disco para arquivos independentes, acompanhadas de um manifesto de hashes para verificação formal:

```bash
cargo run --release -- hdd extract [CAMINHO_IMAGEM.bin] [--out-dir <DIR>] [--json <RELATORIO.json>]
```

#### Exemplo Prático Real:
```bash
cargo run --release -- hdd extract --out-dir firmware_and_images/extracted_partitions
```

#### Artefatos Gerados:
1. `hdd_mbr_master_lba0.bin` (512 bytes): Superbloco MBR com cabeçalho `"SH"`.
2. `hdd_mbr_mirror_lba57344.bin` (512 bytes): Cópia espelhada a 28 MB.
3. `hdd_allocation_descriptors_0x00003000.bin` (4.096 bytes): Tabela de descritores físicos.
4. `hdd_journal_blocks_0x00820000.bin` (1.310.720 bytes): 10 blocos de journaling de transações.
5. `hdd_partition_firmware_0x02000000.bin` (6.173.264 bytes): Kernel operacional AM34.
6. `hdd_partition_firmware_0x02000000_sector_aligned.bin` (6.173.696 bytes): Partição 1 alinhada em setores.
7. `hdd_partition_firmware_0x04000000.bin` (720.896 bytes): Microcódigo de periféricos e OSD.
8. `SHA256SUMS.txt`: Manifesto criptográfico SHA-256 de todos os binários extraídos.

---

### 4.5 Comando 4: Criptoanálise Forense do Contêiner Óptico (`frm analyze`)

Analisa a entropia de Shannon e executa a detecção automatizada de vazamento estrutural da cifra de bloco em modo ECB:

```bash
cargo run --release -- frm analyze [CAMINHO_FRM] [--hdd <IMAGEM_HDD>] [--top <N>]
```

#### Exemplo Prático Real:
```bash
cargo run --release -- frm analyze firmware_and_images/PANA_DVD.FRM --hdd "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"
```

#### Resultados Revelados pelo Comando:
* Demonstra que `PANA_DVD.FRM` possui **entropia de $7.99958\text{ bits/byte}$**.
* Exibe a tabela dos blocos de 64 bits (8 bytes) mais frequentes, identificando que o bloco `03 50 65 C8 7B B8 E0 90` repete-se **3.812 vezes** (preenchimento virgem `0xFF`), e o bloco `E5 D1 F3 4A 51 8B 26 90` repete-se **196 vezes** (preenchimento de zeros nulos `0x00`).
* Realiza a **correlação cruzada com o disco rígido**, comprovando matematicamente que os blocos cifrados do HD nos offsets `0x02000038`, `0x04000030` e `0x04000038` são rigorosamente idênticos à palavra nula do CD óptico, provando o uso da mesma chave e cifra.

---

### 4.6 Comando 5: Decodificação de Descritores Bit-Invertidos MEIHDFS (`hdd decode-descriptors`)

Decodifica a tabela de alocação física nos setores **LBA 24..31** (offset `0x00003000`, 4.096 bytes) aplicando a operação bitwise XOR `0xFF` (`b ^ 0xFF`), audita as 6 categorias de tags estruturadas de 16 bytes e verifica a integridade do prefixo `0x5D` nos 10 blocos de journal em `0x00820000`:

```bash
cargo run --release -- hdd decode-descriptors [CAMINHO_IMAGEM.bin]
```

#### Exemplo Prático Real:
```bash
cargo run --release -- hdd decode-descriptors "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"
```

#### Saída Técnica Produzida:
```text
================================================================================
 PANASONIC MEIHDFS BIT-INVERTED (XOR 0xFF) DESCRIPTOR DECODER                  
================================================================================
 Arquivo da Imagem : firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin
 Offset LBA 24..31 : 0x00003000 (4,096 bytes)
 Espelho LBA 57368 : 0x01C03000 (28.00 MB)
--------------------------------------------------------------------------------
 FUNDAMENTO FORENSE DE BAIXO NÍVEL:
 Os setores LBA 24..31 são armazenados em complemento de 1 (XOR 0xFF) no HD.
 Isso assegura compatibilidade semântica com memórias Flash/EPROM virgens,
 onde bytes 0xFF indicam blocos não alocados ou apagados.
 Ao aplicar (b ^ 0xFF), emergem 256 registros estruturados de 16 bytes.
--------------------------------------------------------------------------------

[+] DISTRIBUIÇÃO SEMÂNTICA DAS TAGS (256 REGISTROS DE 16 BYTES):
    +------+-----------+--------------------------------------------------------+
    | Tag  | Registros | Categoria Semântica & Função no MEIHDFS                |
    +------+-----------+--------------------------------------------------------+
    | 0x5D |        65 | Extents / Contagem de Setores e Ponteiros LBA          |
    | 0x55 |        64 | Sincronismo / Continuação de Extent Alocado            |
    | 0x57 |        47 | Ponteiros de LBA e Vetores de Endereçamento            |
    | 0xE0 |        42 | Entradas de Diretório e Metadados de Arquivos          |
    | 0xE1 |        26 | Nós Virtuais / Inodes do Filesystem MEIHDFS            |
    | 0x5F |        12 | Descritores de Geometria de Bloco / Pool de Mídia      |
    +------+-----------+--------------------------------------------------------+

[+] AMOSTRAGEM DOS PRIMEIROS REGISTROS DECODIFICADOS (OFFSET 0x00..0x80):
    0x0000: 5D0006630000020000FFFFFED0000026 | TAG_5D (Extent/Sectors)
    0x0010: 55AC06630800000000000000BC000024 | TAG_55 (Sync/Cont)
    0x0020: 5D001A630000D60033E606BCD6000028 | TAG_5D (Extent/Sectors)
    0x0030: 55AC1A630800020000000000BE000024 | TAG_55 (Sync/Cont)
    0x0040: 5D001A630000BD93000040008800001A | TAG_5D (Extent/Sectors)
    0x0050: 55101A630400080000000000CE000026 | TAG_55 (Sync/Cont)
    0x0060: 5D201E63000000001800FFFFFFFF3C01 | TAG_5D (Extent/Sectors)
    0x0070: 5F00D9950008200000000000D4000028 | TAG_5F (BlockDesc)

[+] VERIFICAÇÃO DO ESPELHO EM LBA 57368 (28 MB):
    - Status da Tabela Espelho : 100% IDENTICO BIT-A-BIT (Integridade Confirmada)

[+] AUDITORIA DOS 10 BLOCOS DE JOURNAL EM 0x00820000:
    A região transacional de 10 blocos de 128 KB utiliza o prefixo 0x5D
    para registrar operações atômicas de áudio/vídeo:
    - Bloco  0 [0x00820000 / LBA 16640]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  1 [0x00840000 / LBA 16896]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  2 [0x00860000 / LBA 17152]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  3 [0x00880000 / LBA 17408]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  4 [0x008A0000 / LBA 17664]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  5 [0x008C0000 / LBA 17920]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  6 [0x008E0000 / LBA 18176]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  7 [0x00900000 / LBA 18432]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  8 [0x00920000 / LBA 18688]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
    - Bloco  9 [0x00940000 / LBA 18944]: Header: 5DFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF | Prefixo 0x5D: SIM (Válido)
================================================================================
 DECODIFICAÇÃO DE DESCRITORES CONCLUÍDA COM TOTAL SUCESSO                       
================================================================================
```

---

### 4.7 Comando 6: Dump e Desconstrução da TOC e Módulos do `.FRM` (`frm dump-toc`)

Disseca os descritores periódicos de **48 bytes (`0x30`)** da Tabela de Conteúdos (TOC) no cabeçalho do arquivo óptico `PANA_DVD.FRM`, mapeando as entradas para os 3 processadores heterogêneos do Panasonic DMR-EH55 e demonstrando a segmentação do payload nos 3 módulos de firmware alinhados por preenchimento virgem `03 50 65 C8 7B B8 E0 90` (`0xFF * 8`):

```bash
cargo run --release -- frm dump-toc [CAMINHO_FRM]
```

#### Exemplo Prático Real:
```bash
cargo run --release -- frm dump-toc firmware_and_images/PANA_DVD.FRM
```

#### Saída Técnica Produzida:
```text
================================================================================
 PANASONIC PANA_DVD.FRM TOC DUMP & 3-MODULE CONTAINER DECONSTRUCTION           
================================================================================
 Arquivo Analisado : firmware_and_images/PANA_DVD.FRM
 Tamanho Total     : 7531744 bytes (7.18 MB)
--------------------------------------------------------------------------------

[+] ESTRUTURA PERIÓDICA DA TABELA DE CONTEÚDOS (TOC - 48 BYTES / 0x30 CADA):
  * Entrada 0 (Offset 0x0000..0x0030):
    - Destino de Hardware : Main CPU (AM34 / DIGA OS Kernel)
    - Sub-bloco Comum     : EFDC1CDA13881BAB (Offset 0x0010)
    - Hexadecimal Bruto   : 492F0560CEDE508CE3C08AFD49B64A9AEFDC1CDA13881BAB
                            74A78ACD789B84FF2A95C0DDCD1C5FF1696A334256D61E54
  * Entrada 1 (Offset 0x0030..0x0060):
    - Destino de Hardware : DVD Drive Servo (MN102 / DSP Laser Sub-system)
    - Sub-bloco Comum     : EFDC1CDA13881BAB (Offset 0x0040)
    - Hexadecimal Bruto   : 212C29514C08699AC00EBF203F432332EFDC1CDA13881BAB
                            3B7E7EDBB6C57A5323CB732B8CDAA69DA126CE849FEF6DB3
  * Entrada 2 (Offset 0x0060..0x0090):
    - Destino de Hardware : Panel / Timer Controller (MN103S / Front Display)
    - Sub-bloco Comum     : EFDC1CDA13881BAB (Offset 0x0070)
    - Hexadecimal Bruto   : 20861ED612D3E904ADB8D5CA7B848D76EFDC1CDA13881BAB
                            74A78ACD789B84FF40DEDBF6662D4CB11AE6ADB49339BA00

[+] DESCONSTRUÇÃO DO PAYLOAD EM 3 MÓDULOS DE FIRMWARE:
    O contêiner alinha suas imagens por blocos de preenchimento virgem (0xFF * 8):
    Cifra ECB: '03 50 65 C8 7B B8 E0 90'
    +----------+-----------------------+-------------+----------------------------------------------+
    | Módulo   | Faixa de Offsets      | Tamanho     | Destino & Correlação com o Disco Rígido      |
    +----------+-----------------------+-------------+----------------------------------------------+
    | Módulo 1 | 0x00000090..0x000AF000| ~720 KB     | Corresponde à Partição 2 do HD (704 KB)      |
    | Módulo 2 | 0x000AF000..0x000F1800| ~240 KB     | Firmware do Drive Óptico (Servo / Laser)     |
    | Módulo 3 | 0x000F1800..0x0072EC00| ~6.17 MB    | Corresponde à Partição 1 do HD (5.89 MB)     |
    +----------+-----------------------+-------------+----------------------------------------------+

[+] VALIDAÇÃO DAS CORRELAÇÕES COM O SUBSISTEMA DE ARMAZENAMENTO:
  1. O Módulo 1 contém o microcódigo periférico e fontes OSD gravados a 64 MB (LBA 131072).
  2. O Módulo 2 é enviado via interface ATAPI para a Flash interna do gravador de DVD.
  3. O Módulo 3 contém o kernel operacional AM34 e codecs gravados a 32 MB (LBA 65536).
================================================================================
 DUMP DE TOC E ANÁLISE DE MÓDULOS CONCLUÍDOS COM SUCESSO                        
================================================================================
```

---

### 4.8 Comando 7: Consulta e Auditoria do Codebook ECB de 64 bits (`frm codebook`)

Exibe o dicionário de substituição de 64 bits derivado do ataque de texto-plano conhecido (*Known-Plaintext Attack - KPA*), mapeando os blocos cifrados do contêiner para seu conteúdo físico real em hardware e comprovando a identidade criptográfica com o disco rígido:

```bash
cargo run --release -- frm codebook [CAMINHO_FRM]
```

#### Exemplo Prático Real:
```bash
cargo run --release -- frm codebook firmware_and_images/PANA_DVD.FRM
```

#### Saída Técnica Produzida:
```text
================================================================================
 PANASONIC 64-BIT ECB CODEBOOK — KNOWN-PLAINTEXT ATTACK (KPA) DICTIONARY       
================================================================================
 Arquivo Analisado : firmware_and_images/PANA_DVD.FRM
 Total de Blocos   : 941468 blocos de 64 bits (8 bytes cada)
--------------------------------------------------------------------------------
 PRINCÍPIO MATEMÁTICO DO CODEBOOK ECB:
 Em modo Electronic Codebook (ECB): C_i = E_K(P_i).
 Blocos de texto-plano idênticos SEMPRE produzem blocos cifrados idênticos.
 Ao correlacionar preenchimentos conhecidos de hardware (Flash virgem e zeros),
 obtém-se pares comprovados (Plaintext <-> Ciphertext) para decodificação.
--------------------------------------------------------------------------------

[+] MAPEAMENTO FORMAL DO CODEBOOK (PARES COMPROVADOS):
    +-------------------------+-------------------------+------------+------------------------------------+
    | Bloco Cifrado (Hex)     | Texto-Plano Conhecido   | Ocorrências| Interpretação Forense              |
    +-------------------------+-------------------------+------------+------------------------------------+
    | 035065C87BB8E090        | FF FF FF FF FF FF FF FF |   3812 (30496 B)| Células Virgens de Flash / EPROM (0xFF * 8) |
    | E5D1F34A518B2690        | 00 00 00 00 00 00 00 00 |    196 ( 1568 B)| Preenchimento Nulo / Padding de Zeros (0x00 * 8) |
    | 4D3C5DBB0C357777        | 51 87 13 8D 5B 0A FB 22 |   1280 (10240 B)| Cabeçalho Universal de Partição & Tabelas |
    +-------------------------+-------------------------+------------+------------------------------------+

[+] CORRELAÇÃO CRUZADA IRREFUTÁVEL COM O DISCO RÍGIDO (HDD):
    * Offset 0x02000038 (Partição 1 / 32 MB): Bloco 'E5 D1 F3 4A 51 8B 26 90'
    * Offset 0x04000030 (Partição 2 / 64 MB): Bloco 'E5 D1 F3 4A 51 8B 26 90'
    * Offset 0x04000038 (Partição 2 / 64 MB): Bloco 'E5 D1 F3 4A 51 8B 26 90'
    -> PROVA DEFINITIVA: O HD e o CD óptico compartilham o mesmo algoritmo e chave!
================================================================================
 CONSULTA AO CODEBOOK CONCLUÍDA COM SUCESSO                                    
================================================================================
```

---

### 4.9 Comando 8: Inspeção de Cabeçalhos e Hex Dumper (`headers`)

Substitui e expande o utilitário legado `inspect_firmware_headers.py`. Exibe formatação hexadecimal + ASCII dos blocos críticos do sistema (cabeçalhos de firmware do HDD em 32 MB e 64 MB, e cabeçalho do container óptico `PANA_DVD.FRM`), validando a assinatura mágica de partição Panasonic (`51 87 13 8D 5B 0A FB 22`). Também permite executar hex dump arbitrário de qualquer arquivo em qualquer offset e comprimento:

```bash
# Inspeção automática dos cabeçalhos do ecossistema DMR-EH55
cargo run --release -- headers [--length 128]

# Hex dump pontual de qualquer binário em offset específico
cargo run --release -- headers dump <ARQUIVO> --offset <OFFSET_HEX_OU_DEC> --length <TAMANHO>
```

---

### 4.10 Comando 9: Varredura Forense de Strings e Palavras-Chave (`scan`)

Substitui e expande o script `analyze_deep.py`. Executa varredura profunda e instantânea de sequências de texto ASCII imprimíveis utilizando acesso direto à memória mapeada (`memmap2`), filtrando automaticamente contra o dicionário forense do sistema (`panasonic`, `diga`, `mn2ds`, `am34`, `meihdfs`, `u99`, etc.) ou termos personalizados:

```bash
# Varredura forense padrão contra os dumps do HD e FRM
cargo run --release -- scan [--limit 50] [--min-len 6]

# Varredura direcionada em arquivo específico com keywords customizadas e exportação JSON
cargo run --release -- scan <ARQUIVO> --keyword u99 --keyword error --json relatorio_strings.json
```

---

### 4.11 Comando 10: Indexação e Busca em Manuais de Serviço e Esquemas Técnicos (`manuals`)

Substitui e expande o script `search_manuals.py`. Utiliza o parser nativo em Rust `lopdf` para indexar todo o acervo de 15 manuais de serviço, guias de desmontagem e esquemas elétricos em `manuals_and_schematics/`, identificando páginas exatas e contagens de ocorrências para circuitos integrados, pinagens e procedimentos de diagnóstico:

```bash
# Busca global de termos diagnósticos principais no acervo de manuais
cargo run --release -- manuals

# Busca filtrada por manual específico (ex: esquemas e manuais norte-americanos)
cargo run --release -- manuals --filter dmr-eh55p9 --term U99 --term VEP79138
```

---

### 4.12 Comando 11: Consulta ao Motor RAG Zero-Error e Guardrails Físicos (`rag query`)

Executa a consulta semântica ao motor de inteligência técnica embutido em Rust (`src/rag/`), percorrendo as 6 camadas de defesa em sub-milissegundos (< 1 ms). Protege a bancada contra alucinações técnicas vetando automaticamente consultas ou respostas que violem os limites de 3.3V LVTTL ou que sugiram modificações na região de bootloader Spansion (IPL):

```bash
# Consulta técnica de bancada fundamentada com citações dos manuais oficiais
cargo run --release -- rag query "Qual a pinagem de CN55001 e a tensao maxima de bancada?"

# Saída estruturada em JSON (usada nativamente pelo aplicativo Desktop Tauri)
cargo run --release -- rag query "Quais sao os micro-setores de NVRAM para recuperacao de U99?" --json
```

---

## 5. Especificação Elétrica e Pinout da Interface DWire32A (`CN55001`)

A interface de diagnóstico localizada no conector SMD de 6 pinos **`CN55001`** na Placa Digital Principal (`VEP79138`) não é uma UART assíncrona comum. Trata-se da porta física do protocolo síncrono proprietário **Matsushita DWire32A / On-Chip Debug (OCD)**, conectado diretamente às linhas de depuração em nível de silício do SoC **MN2DS0016**.

### 5.1 Pinout Elétrico, Mapeamento Físico de Bancada e Conexão Universal RP2040

Na placa digital `VEP79138`, o conector de depuração (marcado no esquemático oficial da Panasonic como **`P7501 [B6B-ZR-3R4]`**, legenda *"TO WRITE JIGU"*) situa-se fisicamente entre o cabo flat de 10 vias (`FP50003`) e o de 8 vias (`FP50004`).

> [!CAUTION]
> **DISCLAIMER CRÍTICO DE BANCADA — IDENTIFICAÇÃO DO TERRA (`GND`):**
> * **No Manual de Serviço Oficial da Panasonic (Esquemático do Conector P7501):** O **`GND` é ESTRITAMENTE O PINO 4**.
> * **Na Contagem Física Visual (Do cabo flat de 10 vias `FP50003` para o de 8 vias `FP50004`):** O **`GND` corresponde ao PINO 3**.
> * **A regra canônica e correta de engenharia é a do MANUAL (Pino 4 = `GND`).** O desenhista do CAD da Panasonic numerou o conector no esquema em ordem espelhada em relação à contagem visual da esquerda para a direita ($\text{Pino Físico} = 6 - \text{Pino Esquemático} + 1$).
> * **NUNCA conecte o terra no Pino 6** (o que causaria curto-circuito destrutivo com a linha de +3.3V VDD Sense) nem sem validação prévia. Antes de ligar o Raspberry Pi Pico ou qualquer ferramenta de bancada, **sempre valide com o multímetro o bipe sonoro de $0.0\,\Omega$ de continuidade direta entre o pino de terra e a carcaça de lata metálica desprovida de tinta!**

![Topografia Física e Pinagem VEP79138](docs/images/panasonic_dmr_eh55_pcb_p7501_orientation.png)

```text
       ORIENTAÇÃO FÍSICA NA PLACA VEP79138 (CONTAGEM DE 10V PARA 8V)
            ┌───────────┐
     Pino 1 │ ● EXTRG   │ Gatilho Externo OCD / Breakpoint [Pino 6 do Manual]
     Pino 2 │ ● SCLK    │ Clock Síncrono DWire32A (Master 1.0 MHz) [Pino 5 do Manual]
     Pino 3 │ ● GND     │ Ground Comum (0.0Ω para o Chassi Metálico!) [Pino 4 do Manual]
     Pino 4 │ ● SDATA   │ Dados Half-Duplex (Pull-up 4.7kΩ p/ 3.3V) [Pino 3 do Manual]
     Pino 5 │ ● nRESET  │ Linha de Reset Mestre (Controle Open-Drain) [Pino 2 do Manual]
     Pino 6 │ ● VDD     │ Target Power Sense (+3.3V Referência) [Pino 1 do Manual]
            └───────────┘
```

![Esquemático Oficial Panasonic P7501](docs/images/schematic_p7501_panasonic_official.png)

> [!NOTE]
> **Compatibilidade de Hardware:** Qualquer microcontrolador equipado com o silício **RP2040** (Raspberry Pi Pico, Pico W, Arduino Nano RP2040 Connect, Waveshare RP2040-Zero, Seeed XIAO RP2040) é 100% suportado, pois a temporização crítica é gerada internamente pelas máquinas de estados **PIO0**.

| Pino Físico (10V $\rightarrow$ 8V) | Pino Esquemático P7501 (Manual) | Sinal Matsushita | Pino RP2040 | Componente de Proteção / Condicionamento | Função Técnica Detalhada |
| :---: | :---: | :---: | :---: | :---: | :--- |
| **Pino 1** | Pino 6 | `EXTRG` | `GP2` (Pino 4) | Resistor $100\,\Omega$ em série | Gatilho externo para forçar parada (*Halt Request*) na CPU no Cold Break-In. |
| **Pino 2** | Pino 5 | `SCLK` | `GP3` (Pino 5) | Resistor $100\,\Omega$ em série | Clock serial síncrono master (Operação em $1.0\text{ MHz}$). |
| **Pino 3** | **Pino 4** | **`GND`** | **`GND` (Pino 38)** | Ligação direta de baixa impedância | **Referência de 0V ($0.0\,\Omega$ com a carcaça/chassi metálico).** |
| **Pino 4** | Pino 3 | `SDATA` | `GP4` (Pino 6) | Resistor $100\,\Omega$ + Pull-up $4.7\text{ k}\Omega$ p/ 3.3V | Barramento de dados síncrono half-duplex bidirecional. |
| **Pino 5** | Pino 2 | `nRESET` | `GP5` (Pino 7) | Resistor $100\,\Omega$ em série (**Modo Open-Drain**) | Linha de Reset mestre do SoC (`XRST` / `D_RESET`). **Nunca 3.3V push-pull!** |
| **Pino 6** | Pino 1 | `VDD` | `GP26 / ADC0` | Resistor $1.0\text{ k}\Omega$ em série | Monitoramento de tensão do target (Validação de $+3.3\text{V}$ estável). |

![Esquema Elétrico de Montagem da Sonda Pico](docs/images/dwire32a_pico_probe_assembly_schematic.png)

---

### 5.2 Regras Críticas de Engenharia e Prevenção de Danos

> [!CAUTION]
> **PERIGO DE QUEIMA POR CURTO-CIRCUITO EM `nRESET`:**
> A linha de reset do gravador é controlada por um circuito integrado supervisor de reset na placa da Panasonic, que ancora a malha a `GND` durante transições. O GPIO `GP5` do Raspberry Pi Pico **JAMAIS deve operar como saída Push-Pull dirigindo 3.3V em nível alto**.
> * **Para Resetar o Alvo (0V):** Configure `GP5` como **Saída em nível 0**.
> * **Para Liberar o Reset (3.3V):** Configure `GP5` como **Entrada de Alta Impedância (High-Z)**, permitindo que o resistor de pull-up original da placa eleve a tensão suavemente.

> [!IMPORTANT]
> **DESBLOQUEIO DE GRAVAÇÃO DO PINO 14 (`WP#/ACC`) DA FLASH SPANSION:**
> Na arquitetura da memória Flash Spansion `S29JL064H70TFI00` (Top-Boot, Model 00), o pino 14 (`WP#`) protege contra escrita e apagamento os micro-setores `SA140` e `SA141` (`0x007FC000` a `0x007FFFFF`) se estiver em nível baixo (`0V`).
> Antes de executar qualquer rotina in-circuit:
> 1. Localize a **ilha quadrada estanhada SMD** adjacente ao pino 14 na borda inferior direita do chip `IC55007`.
> 2. Meça a tensão com o aparelho energizado. Se apresentar `3.3V`, a escrita já está destravada.
> 3. Se apresentar `0V`, injete temporariamente `3.3V` nessa ilha através de um resistor de $1.0\text{ k}\Omega$ durante a execução do comando de apagamento.

---

### 5.3 Diagrama de Temporização e Sinais DWire32A

O protocolo DWire32A opera no modo Half-Duplex síncrono. O clock é gerado unicamente pelo Master (RP2040), e a alternância de transmissão (Turnaround) ocorre mantendo a linha em flutuação (Hi-Z) no intervalo de borda:

```text
               DIAGRAMA DE SINAIS DO PROTOCOLO SÍNCRONO DWIRE32A
 nRESET  ──┐                                                    ┌─────────────────
 (GP5)     │                                                    │ (Liberação Reset)
           └────────────────────────────────────────────────────┘
             |<---------- Janela de Reset (50 ms) ------------->|
 SCLK    ───────────┐  ┌──┐  ┌──┐  ┌──┐                         ┌──┐  ┌──┐  ┌──┐  ┌─
 (GP3)              └──┘  └──┘  └──┘  └── ... (32 Pulsos) ──────┘  └──┘  └──┘  └──┘
                    |<-- Cold Break-In Wakeup -->|              |<-- Frame Dados -->|
 SDATA   ───────────┐                                      ┌────┬─────┬─────┬─────┬──
 (GP4)              └──────────────────────────────────────┘    │ B7  │ B6  │ B5  │ ...
                                                                ▲ Amostragem na subida
                                                                (Setup na borda de descida)
```

---

## 6. Mapa de Memória Físico da Arquitetura Matsushita AM34 (`MN2DS0016`)

O processador Matsushita **AM34** adota um espaço de endereçamento plano linear de 32 bits ($4\text{ Gigabytes}$). O subsistema de gerenciamento de barramento divide o mapa em janelas espelhadas cacheadas e acessos diretos não cacheados (*uncached*):

| Faixa de Endereços Físicos | Tamanho | Tipo de Acesso | Dispositivo / Mapeamento de Hardware na PCB `VEP79138` |
| :--- | :---: | :---: | :--- |
| `0x00000000` – `0x007FFFFF` | 8 MB | Cacheado L1 | **Espelho de Boot:** NOR Flash Spansion `S29JL064H` (Execução de código IPL com aceleração de cache). |
| `0x80000000` – `0x807FFFFF` | 8 MB | **Uncached Direto** | **Acesso Físico Direto à Spansion Flash (`CS0` / `SB0`):** Barramento de 16 bits (*Word Mode*). **Região mandatória para comandos JEDEC!** |
| `0x8C000000` – `0x8FFFFFFF` | 64 MB | Uncached Direto | Memória SDRAM Externa Elpida (`IC53202` / `IC53203` - 512 Mbit DDR/SDR). |
| `0x90000000` – `0x93FFFFFF` | 64 MB | Cacheado L1 | Espelho acelerado de execução da memória SDRAM do sistema. |
| `0xC0000000` – `0xD3FFFFFF` | 320 MB | Uncached MMIO | Registradores de Controle de Barramento do SoC, Timers, DMA e Decodificadores MPEG-2. |
| **`0xD4000000` – `0xD4000FFF`** | 4 KB | Uncached MMIO | **Controladores Seriais Matsushita (`SC0`, `SC1`, `SC2`):** UARTs internas de diagnóstico. |
| **`0xD5000000` – `0xD5000FFF`** | 4 KB | Uncached MMIO | **Controlador ATA/IDE:** Barramento PATA para transferência de dados do disco rígido. |
| `0xD6000000` – `0xD7FFFFFF` | 32 MB | Uncached MMIO | Motores Gráficos de OSD, Vídeo DACs, Interface HDMI e CODECs de Áudio. |
| `0xFF000000` – `0xFFFFFFFF` | 16 MB | Registradores Core | **Unidade On-Chip Debug (OCD) & CPU Core:** `DCSR` (Debug Control), `IBCR` (Breakpoint Registers). |

---

### 6.1 Regra de Ouro JEDEC para o RAM Loader Injetado

> [!CRITICAL]
> **TRANSLATAÇÃO DE ENDEREÇOS JEDEC EM MODO PALAVRA (16 BITS):**
> Quando a CPU AM34 acessa a Flash em modo de 16 bits (`BYTE# = Alto`), as linhas de endereço JEDEC da Spansion (`0x555` e `0x2AA`) são transladadas pelo barramento através de um deslocamento à esquerda de 1 bit (`<< 1`):
> * **Ciclo JEDEC 1:** Endereço `0x80000000 + (0x555 << 1)` = **`0x80000AAA`** (Dado: `0x00AA`)
> * **Ciclo JEDEC 2:** Endereço `0x80000000 + (0x2AA << 1)` = **`0x80000554`** (Dado: `0x0055`)
> 
> Comandos de escrita e apagamento **jamais podem ser emitidos no espelho cacheado `0x00000000`**, pois ficariam retidos nas linhas de cache L1 da CPU e os ciclos de comando não chegariam aos pinos da Spansion!

---

## 7. Síntese das Descobertas Criptográficas e Forenses (Forensic Cryptanalysis & Storage Deep-Dive)

A investigação forense realizada sobre os arquivos binários do ecossistema revelou aspectos cruciais da segurança da plataforma Matsushita:

### 7.1 O Modo ECB e o Vazamento Estrutural ("ECB Leak")

O contêiner óptico `PANA_DVD.FRM` ($7.531.744\text{ bytes}$) possui entropia quase perfeita de **$7.99958\text{ bits/byte}$** e ausência total de strings legíveis (*zero plain text*). No entanto, a análise de frequência em múltiplos de 8 bytes ($64\text{ bits}$) expôs um vazamento criptográfico característico do **modo ECB (*Electronic Codebook*)**:

$$\text{Bloco Cifrado } C_i = E_K(\text{Bloco Plano } P_i)$$

Como blocos idênticos em texto plano resultam em blocos cifrados idênticos no modo ECB (sem difusão inter-blocos CBC), a análise matemática isolou as seguintes assinaturas:

| Bloco Cifrado (Hexadecimal) | Ocorrências em `PANA_DVD.FRM` | Volume Total | Identificação Criptográfica do Texto Plano |
| :---: | :---: | :---: | :--- |
| **`03 50 65 C8 7B B8 E0 90`** | **3.812 vezes** | $30.496\text{ bytes}$ | **Cifragem de 8 bytes virgens de Flash (`FF FF FF FF FF FF FF FF`).** |
| **`E5 D1 F3 4A 51 8B 26 90`** | **196 vezes** | $1.568\text{ bytes}$ | **Cifragem de 8 bytes nulos de padding (`00 00 00 00 00 00 00 00`).** |
| **`4D 3C 5D BB 0C 35 77 77`** | **1.280 vezes** | $10.240\text{ bytes}$ | **Estruturas repetidas de cabeçalhos internos e tabelas alinhadas (`51 87 13 8D 5B 0A FB 22`).** |

---

### 7.2 Codebook ECB de 64 bits e Ataque de Texto-Plano Conhecido (Known-Plaintext Attack - KPA)

A ausência de encadeamento no modo ECB possibilita a construção de um **Codebook de 64 bits** via ataque de texto-plano conhecido (*Known-Plaintext Attack*):
1. **Pares Comprovados de Hardware:** Células virgens de silício (`0xFF * 8`) e preenchimentos nulos (`0x00 * 8`) são gerados aos milhares durante o processo de compilação e alinhamento de memória.
2. **Decodificação Direta Sem Chave:** Ao mapear esses blocos cifrados diretamente para seus equivalentes em texto plano, é possível inspecionar visualmente a estrutura de contêineres e limites de arquivos sem necessidade de quebrar o algoritmo da cifra (AES-128 ou Feistel de 64 bits).
3. **Auditoria Automatizada:** O comando `panadiga frm codebook` realiza essa varredura em tempo real sobre qualquer imagem de atualização compatível.

---

### 7.3 Desconstrução do Contêiner `PANA_DVD.FRM` em 3 Módulos de Firmware

A análise das estruturas internas do contêiner óptico expõe a arquitetura multiprocessada do chassi DIGA:

1. **Tabela de Conteúdos Periódica (TOC) de 48 Bytes (`0x30`):**
   Os primeiros 512 bytes de `PANA_DVD.FRM` contêm 3 registros descritores com alinhamento estrito:
   * **Entrada 0 (Offset `0x0000`..`0x002F`, Sub-bloco `0x0010` = `EF DC 1C DA 13 88 1B AB`):** Main CPU (Matsushita AM34 / DIGA OS Kernel).
   * **Entrada 1 (Offset `0x0030`..`0x005F`, Sub-bloco `0x0040` = `EF DC 1C DA 13 88 1B AB`):** DVD Drive Servo (DSP MN102 para controle do feixe óptico).
   * **Entrada 2 (Offset `0x0060`..`0x008F`, Sub-bloco `0x0070` = `EF DC 1C DA 13 88 1B AB`):** Panel / Timer Controller (Microcontrolador MN103S `IC7001` do display VFD frontal).

2. **Payload Alinhado em 3 Imagens por Blocos Virgens `03 50 65 C8 7B B8 E0 90`:**
   O arquivo intercala blocos maciços de `0xFF * 8` para alinhar as três imagens de firmware:
   * **Módulo 1 (~720 KB, Offset `0x00000090` a `0x000AF000`):** Corresponde rigorosamente à **Partição 2 do HD (704 KB em 64 MB / LBA 131072)** — microcódigo de periféricos e fontes OSD.
   * **Módulo 2 (~240 KB, Offset `0x000AF000` a `0x000F1800`):** Microcódigo de controle do mecanismo óptico, gravado na Flash interna do drive de DVD.
   * **Módulo 3 (~6,17 MB, Offset `0x000F1800` a `0x0072EC00`):** Corresponde rigorosamente à **Partição 1 do HD (5,89 MB em 32 MB / LBA 65536)** — kernel executável AM34 e codecs de vídeo MPEG-2/DivX.

A correlação física e lógica é validada pelo comando `panadiga frm dump-toc`.

---

### 7.4 Vínculo Criptográfico Irrefutável entre o CD (`.FRM`) e o HD (`hdd_100.bin`)

Ao examinar as duas partições de firmware no disco rígido IDE:
* **Partição 1 (Offset `0x02000000` / 32 MB):** Inicia com a assinatura mágica `51 87 13 8D 5B 0A FB 22`. No offset `0x02000038`, surge o bloco de 8 bytes: **`E5 D1 F3 4A 51 8B 26 90`**.
* **Partição 2 (Offset `0x04000000` / 64 MB):** Inicia com a mesma assinatura `51 87 13 8D 5B 0A FB 22`. Nos offsets `0x04000030` e `0x04000038`, surgem **dois blocos seguidos de `E5 D1 F3 4A 51 8B 26 90`**.

**Conclusão Forense:** O bloco `E5 D1 F3 4A 51 8B 26 90` é exatamente o padrão de preenchimento nulo que aparece 196 vezes no arquivo de atualização do CD. Isso comprova que **o disco rígido armazena suas partições de firmware cifradas com a mesmíssima cifra de bloco de 64 bits e a mesmíssima chave simétrica utilizada no disco óptico**.

---

### 7.5 Arquitetura Bit-Invertida (One's Complement / XOR 0xFF) do MEIHDFS

A análise de baixo nível da tabela de descritores de alocação física nos setores **LBA 24..31** (offset `0x00003000`, 4.096 bytes) revelou um dos segredos mais profundos do sistema de arquivos MEIHDFS:

1. **Alinhamento Semântico com Flash/EPROM (0xFF = Não Alocado):**
   Em memórias de estado sólido não voláteis (NOR Flash / EPROM), células apagadas possuem todos os bits em nível alto (`0xFF`). O MEIHDFS inverte deliberadamente os bytes em disco (`b ^ 0xFF`) para que o valor bruto `0x00` represente células livres na lógica interna do gravador, evitando escritas desnecessárias em mídias de calibração.
2. **Estrutura Decodificada de 256 Registros de 16 Bytes:**
   Ao aplicar XOR `0xFF` sobre a tabela, emergem 6 categorias de tags mestras que cobrem 100% dos registros:
   * **Tags `0x5D`, `0x55`, `0x57`, `0x5F`:** Gerenciamento de extents físicos, contagem de setores alocados, sincronismo de fragmentos e vetores de ponteiros LBA.
   * **Tags `0xE0`, `0xE1`:** Entradas de diretório de arquivos e nós virtuais (*V-Nodes* / Inodes) do sistema de arquivos.
3. **Journaling com Prefixo `0x5D`:**
   A área transacional em `0x00820000` (10 blocos de 128 KB cada) utiliza o prefixo `0x5D` em seus cabeçalhos para registrar operações atômicas de áudio e vídeo, garantindo imunidade a corrupção durante desligamentos forçados.

O comando `panadiga hdd decode-descriptors` executa a auditoria completa dessa arquitetura.

---

### 7.6 O Paradoxo do Root-of-Trust e a Flash Externa Spansion

Embora o CD óptico e as partições de disco rígido sejam fortemente criptografados, **o hardware físico da Panasonic DMR-EH55 não possui descodificação criptográfica on-the-fly no barramento de memória externa da placa digital**.

Para que a CPU AM34 consiga buscar sua primeira instrução no vetor de reset (`0x80000000`), a memória Flash física Spansion **`IC55007` TEM QUE CONTER CÓDIGO NATIVO MN103S EM TEXTO PLANO NÃO CRIPTOGRAFADO**. Portanto:
* A chave mestra, as rotinas de decifragem das partições de firmware e os algoritmos de expansão estão todos presentes no código do bootloader na Spansion.
* A extração direta da Flash externa contorna 100% da criptografia do sistema.

---

## 8. Guia Prático de Resolução do Erro U99 em 3 Rotas de Engenharia

O erro **`U99`** é o travamento mais temido da linha Panasonic DIGA. Ele ocorre quando o autoteste do firmware detecta inconsistência irrecuperável na área de calibração `EEMUL` da Flash Spansion (`IC55007`). Abaixo estão as três rotas de resolução técnica homologadas:

```mermaid
flowchart TD
    Falha["Gravador Panasonic DMR-EH55 com Travamento U99"] --> Triagem{"Recursos Disponíveis na Bancada?"}
    
    Triagem -->|"Estação de Retrabalho SMD & Gravador Universal"| RotaA["Rota A: Dessoldagem Física TSOP-48 (100% de Sucesso Imediato)"]
    Triagem -->|"Raspberry Pi Pico & Sem Dessolda"| RotaB["Rota B: Injeção DWire32A In-Circuit (Método Não Invasivo)"]
    Triagem -->|"Falha no HD / Troca por SSD"| RotaC["Rota C: Clonagem MEIHDFS de 100 MB via dd"]

    RotaA --> ProcedimentoA["1. Dessoldar IC55007 (345°C)<br/>2. Ler 8 MB no XGecu T48 / SVOD 4<br/>3. Executar 'panadiga nvram clean'<br/>4. Regravar dump limpo e ressoldar"]
    RotaB --> ProcedimentoB["1. Conectar 6 vias no CN55001<br/>2. Verificar 3.3V no Pino 14 (WP#)<br/>3. Cold Break-In via RP2040 PIO<br/>4. Injetar ram_loader na SRAM e apagar SA134..SA141"]
    RotaC --> ProcedimentoC["1. Conectar novo HD IDE ou SATA SSD<br/>2. Gravar hdd_100.bin nos primeiros 100 MB<br/>3. Iniciar e formatar via Menu de Serviço"]

    ProcedimentoA --> Sucesso["Aparelho 100% Restaurado e Operacional"]
    ProcedimentoB --> Sucesso
    ProcedimentoC --> Sucesso
```

---

### Rota A: Gravação Física Externa TSOP-48 via SVOD 4 / XGecu T48 (Gravação de ROM Completa)

É o método padrão industrial mais rápido, seguro e irrestrito. Permite regravar **100% da ROM Flash (todos os 8 MB, incluindo Bootloader/IPL, fontes OSD e NVRAM)** diretamente no soquete ZIF do gravador universal, sem qualquer trava de segurança in-circuit:
1. **Proteção Térmica:** Aplique fita Kapton sobre os circuitos integrados vizinhos (SDRAMs Elpida `IC53202`/`IC53203` e SoC `IC55001`).
2. **Remoção do Componente:** Aplique fluxo *no-clean* (Amtech NC-559-ASM) nos terminais do chip `IC55007` (Spansion `S29JL064H70TFI00`). Remova o componente utilizando bocal largo de ar quente regulado a $340^\circ\text{C}–345^\circ\text{C}$.
3. **Limpeza dos Pinos:** Limpe os terminais do chip e as ilhas da PCB utilizando malha de cobre trançada e álcool isopropílico 99.8%.
4. **Leitura e Backup:** Insira o chip no adaptador TSOP-48 ZIF v3 do gravador universal (**XGecu T48**, **SVOD 4** ou **TL866II-Plus**). Efetue a leitura completa dos $8.388.608\text{ bytes}$ e salve como `DMR_EH55_ORIGINAL.bin`.
5. **Limpeza Cirúrgica com `panadiga`:**
   ```bash
   cargo run --release -- nvram clean DMR_EH55_ORIGINAL.bin -o DMR_EH55_CLEAN.bin
   ```
6. **Regravação e Verificação:** Grave o arquivo `DMR_EH55_CLEAN.bin` no chip com o recurso *Verify* ativado.
7. **Ressoldagem:** Posicione o chip na placa com alinhamento rigoroso dos 48 terminais, solde com ponta faca/chanfrada e inspecione sob microscópio para garantir ausência de curtos.
8. **Boot:** Ao ligar o aparelho, o bootloader detectará os 64 KB preenchidos com `0xFF`, recriará as estruturas padrão de calibração e inicializará o display frontal sem erro.

---

### Rota B: In-Circuit via DWire32A com RP2040 e Injeção de RAM Loader

Método não invasivo desenvolvido pela engenharia deste projeto para recuperação sem dessoldar o chip:
1. **Conexão Elétrica:** Conecte 6 fios flexíveis AWG 30 (Kynar) entre o conector `CN55001` da placa digital e os pinos do Raspberry Pi Pico conforme a especificação da Seção 5.
2. **Verificação do Pino 14 (`WP#`):** Meça a tensão na ilha quadrada adjacente ao pino 14 do `IC55007`. Caso meça `0V`, injete `3.3V` através de um resistor de $1.0\text{ k}\Omega$.
3. **Compilação e Carga do Firmware Pico:** Compile o firmware em `dwire32a_jtag_research/` utilizando o Pico SDK e carregue o arquivo `.uf2` no RP2040.
4. **Captura Cold Break-In:** Conecte o terminal serial USB do RP2040 ($115200\text{ bps}$). Ligue a chave de alimentação do DMR-EH55. O RP2040 puxará `nRESET` para nível baixo, emitirá 32 pulsos em `SCLK`, liberará o reset em modo Open-Drain e congelará a CPU AM34 em modo de depuração OCD antes da execução do bootloader corrompido.
5. **Injeção do RAM Loader:** O RP2040 injeta o código executável compilado de `ram_loader.c` na SRAM interna do SoC (`0x00004000`).
6. **Apagamento JEDEC:** O processador executa nativamente o algoritmo JEDEC em modo palavra no endereço físico sem cache `0x807F0000`, disparando os ciclos de *Sector Erase* nos 8 micro-setores `SA134` a `SA141`.
7. **Reinicialização:** O comando `pi` de breakpoint sinaliza a conclusão. O RP2040 libera o barramento e o DMR-EH55 inicializa limpo.

---

### Rota C: Substituição do HD por SSD via `dd` Preservando os 100 MB do MEIHDFS

Utilizada quando o disco rígido original de 80 GB ou 160 GB apresenta setores defeituosos (*bad sectors*), travamentos em `HDDERR` ou ruído mecânico:
1. **Seleção de Hardware:** Utilize um novo disco rígido PATA/IDE de 3.5" (de 80 GB a 500 GB) ou um SSD SATA de 2.5" acoplado a um adaptador PATA/SATA de alta velocidade baseado no chipset **Marvell 88SA8052** (evite conversores genéricos de baixa qualidade que causam timeout no barramento ATA Matsushita).
2. **Gravação da Área Reservada de Boot (100 MB):** Conecte o novo disco/SSD a uma estação Linux e grave a imagem binária mestre do repositório a partir do setor LBA 0:
   ```bash
   sudo dd if="firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin" of=/dev/sdX bs=1M count=100 status=progress conv=fsync
   ```
   *(Substitua `/dev/sdX` pelo identificador correto da unidade de destino; **atenção extrema para não sobrescrever unidades do sistema!**).*
3. **Instalação Mecânica:** Conecte o novo disco ao cabo flat IDE de 40 vias e ao conector de alimentação Molex da fonte de alimentação do DMR-EH55.
4. **Formatação pelo Menu de Serviço:**
   * Ligue o aparelho com a tecla `STOP` do painel frontal e a tecla `OPEN/CLOSE` pressionadas simultaneamente por 5 segundos para entrar no modo de auto-diagnóstico.
   * Navegue até o menu de serviço de disco rígido e execute a inicialização do volume. O firmware manterá a área intacta de 100 MB gravada pelo `dd` e reconstruirá os catálogos contínuos de áudio e vídeo MEIHDFS no restante da capacidade da unidade.

---

## 9. Tabela de Consulta Rápida e Indexação dos 15 Manuais Técnicos em PDF

O diretório [`manuals_and_schematics/`](manuals_and_schematics/) contém o acervo oficial completo de 15 volumes de manuais de serviço, guias de montagem, cadernos de esquemáticos e manuais de instrução:

| Arquivo PDF | Páginas | Categoria Técnica | Modelos Cobertos | Conteúdo de Engenharia e Destaques |
| :--- | :---: | :--- | :--- | :--- |
| **`c2c7f1.pdf`** | 177 | **Esquemas Elétricos e PCB** | DMR-EH55EC, DMR-EH55EP, DMR-EH56EG | **Caderno de Esquemas Elétricos Mestre:** Diagramas esquemáticos completos de todas as PCBs, barramentos de memória do SoC `MN2DS0016`, circuitos de fonte e pinagem completa do conector `CN55001`. |
| **`9e8a23.pdf`** | 82 | **Manual de Serviço Vol. 1** | Linha DMR-EH55 Global | **Manual de Diagnóstico e Desmontagem:** Tabela de códigos de auto-diagnóstico (`U99`, `U61`, `U88`, `F01` a `F99`), desmontagem do mecanismo do drive óptico e ajustes de alinhamento. |
| **`panasonic_dmr-eh55ec.pdf`** | 96 | **Manual de Serviço Regional** | DMR-EH55EC (Alemanha / Europa Continental) | Especificações do sintonizador de TV analógico PAL/SECAM B/G/D/K, matriz de componentes e listas de peças de reposição. |
| **`panasonic_dmr-eh55ee.pdf`** | 92 | **Manual de Serviço Regional** | DMR-EH55EE (Rússia / Leste Europeu / Oriente Médio) | Calibração de canais regionais, padrões SECAM/OIRT e circuitos de entrada de áudio e vídeo estéreo. |
| **`panasonic_dmr-eh55p9_eh55pc9.pdf`** | 104 | **Manual de Serviço Regional** | DMR-EH55P9 / DMR-EH55PC9 (América do Norte - EUA/Canadá) | **Especificações DivX DRM e NTSC:** Detalhamento do registro DivX DRM (página 15), modulação RF NTSC e arquitetura do chassi norte-americano. |
| **`Panasonic dmr-eh55 installation.pdf`** | 4 | **Procedimento de Atualização** | Linha DMR-EH55 Global | **Guia Oficial de Gravação de Firmware:** Instruções de queima do CD-R em ISO9660 Disc-At-Once e sequência de mensagens no painel (`BYE` $\rightarrow$ `L1 P01..P05` $\rightarrow$ `COMPLETE`). |
| **`22767039.pdf`** | 12 | **Guia Técnico de Integração** | Família DIGA / VIERA Link | Especificação técnica de comandos HDAVI Control e sinalização de controle CEC via barramento HDMI. |
| **`2Panasonic_DIGA_DMR-EH55_DVD_Recorder_User_Manual.pdf`** | 84 | **Manual de Operação (Inglês)** | DMR-EH55 Global | Manual completo de operação em inglês internacional, tabelas de modos de gravação e gerenciamento de títulos. |
| **`6097ea.pdf`** | 76 | **Manual de Instruções** | Família DMR-EH55 | Guia de conexões de cabos de vídeo componente, áudio óptico digital e modos de gravação XP (10 Mbps), SP (5 Mbps), LP (2.5 Mbps) e EP (1.2 Mbps). |
| **`a5e678.pdf`** | 80 | **Manual do Usuário (Europa)** | DMR-EH55 / DMR-EH56 | Instruções de operação multilíngue para os mercados europeus e configuração de canais via Q-Link. |
| **`cb6cee.pdf`** | 88 | **Manual do Usuário (Australásia)** | DMR-EH55 (Austrália e Nova Zelândia) | Especificações elétricas de alimentação em 240V, sintonização de canais VHF/UHF e funções de Time-Slip. |
| **`dmreh55.pdf`** | 36 | **Guia de Referência Rápida** | DMR-EH55 | Especificações nominais de consumo de energia, compatibilidade de mídias DVD-RAM, DVD-R, DVD-RW e DVD+R. |
| **`Panasonic_DIGA_DMR-EH55_DVD_Recorder_User_Manual.pdf`** | 88 | **Manual de Operação Completo** | DMR-EH55 Global | Guia exaustivo de funções de edição não-linear, divisão de títulos e cópia bidirecional HDD $\leftrightarrow$ DVD. |
| **`panasonic_dmr-eh55,dmr-eh56_dvd_user_manual.pdf`** | 84 | **Manual de Operação Unificado** | DMR-EH55 / DMR-EH56 | Instruções detalhadas de gravação em alta velocidade (High-Speed Dubbing) e tabelas de mensagens de advertência. |
| **`philips-dmr-eh55.pdf`** | 68 | **Manual de Engenharia Cruzada** | Chassi DIGA Compartilhado | Documentação técnica de manutenção de componentes compartilhados entre gravadores de DVD de arquitetura idêntica. |

---

## 10. Links Diretos para o Acervo de Dossiês Técnicos Especializados (`docs/`)

Para aprofundamento específico em cada área de engenharia, consulte o acervo oficial de dossiês técnicos bilíngues do projeto:

* 🏛️ **[Arquitetura de Sistemas e Blueprint de Engenharia](docs/pt-BR/ARQUITETURA_E_BLUEPRINT_DE_ENGENHARIA.md)** ([English Mirror](docs/en-US/SYSTEM_ARCHITECTURE_AND_ENGINEERING_BLUEPRINT.md))  
  *Especificação mestre do ecossistema: decomposição de monólitos, Pure FP domain (`&[u8]`), MVVM rigoroso com ADTs, GDB RSP introspectivo com anel circular de telemetria sem perdas, logging unificado e invariantes do SoC MN2DS0016.*
* 📱 **[Arquitetura Mobile Android e Conectividade Sem Fio BLE](docs/pt-BR/ARQUITETURA_MOBILE_ANDROID_E_BLUETOOTH.md)** ([English Mirror](docs/en-US/ANDROID_APK_AND_BLUETOOTH_ARCHITECTURE.md))  
  *Especificação técnica do APK Android (`aarch64-linux-android`), HAL nativa em Rust (BLE Nordic UART Service / USB CDC OTG), temporização em nanossegundos vs latência BLE, proibição absoluta de alimentar o Pico pelo conector P7501 e pipeline CI/CD.*
* 🧪 **[Arquitetura de Testes, Matriz de Rastreabilidade e Conformidade](docs/pt-BR/ARQUITETURA_DE_TESTES_E_CONFORMIDADE.md)** ([English Mirror](docs/en-US/TESTING_ARCHITECTURE_AND_COMPLIANCE.md))  
  *Pirâmide de testes em 5 camadas (Pure FP, HAL/GDB, MVVM/UI, CLI SSOT, E2E CI/CD), matriz de rastreabilidade completa, política de 0% de duplicação (`jscpd`) e relatório dos 9 Quality Gates.*
* ⚡ **[Blueprint de Engenharia de Hardware e Circuitos de Bancada](docs/pt-BR/BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md)** ([English Mirror](docs/en-US/BLUEPRINT_BENCH_WIRING_AND_SCHEMATIC.md))  
  *Esquema elétrico de bancada detalhado, especificações de cabos, condicionamento de sinais, pinagem completa do conector P7501/CN55001, Spansion S29JL064H TSOP-48 e procedimentos cirúrgicos de soldagem.*
* 💾 **[Arquitetura de Storage, Filesystem MEIHDFS e Partições de Disco](docs/pt-BR/ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md)** ([English Mirror](docs/en-US/STORAGE_ARCHITECTURE_MEIHDFS_PARTITIONS.md))  
  *Engenharia reversa completa do disco de 100 MB, MBR com tag ASCII "SH", tabela espelho no LBA 57.344 (28 MB), journal transacional de 10 blocos e decodificação forense do volume contínuo MEIHDFS.*
* 🔐 **[Dossiê de Engenharia Reversa e Criptoanálise de Firmware](docs/pt-BR/DOSSIE_CRIPTOGRAFIA_E_FIRMWARE_PANA_DVD.md)** ([English Mirror](docs/en-US/DOSSIER_CRYPTOGRAPHY_AND_FIRMWARE_PANA_DVD.md))  
  *Criptoanálise aprofundada da cifra de bloco de 64 bits em modo ECB, estudo de entropia de PANA_DVD.FRM, correlação cruzada de partições e desmistificação do Root-of-Trust em texto plano da Flash.*
* 🎯 **[Estratégia de Integração Ghidra SLED e Servidor GDB RSP](docs/pt-BR/ESTRATEGIA_INTEGRACAO_GHIDRA_GDB.md)** ([English Mirror](docs/en-US/GHIDRA_GDB_INTEGRATION_STRATEGY.md))  
  *Módulo SLED AM34/MN103S para Ghidra, scripts de segmentação e ponte TCP `:3333` do GDB Remote Serial Protocol.*
* 🪛 **[Guia de Montagem e Fotos da Sonda de Bancada JST ZR](docs/pt-BR/GUIA_MONTAGEM_E_FOTOS_PROBE_BANCADA.md)** ([English Mirror](docs/en-US/BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md))  
  *Procedimento ilustrado de montagem, chicote JST ZR 1.5mm de 6 vias (B6B-ZR-3R4) e teste de terra obrigatório (Pino 4 GND).*
* 📘 **[Inventário Técnico, Hashes Criptográficos e Guia de Operação](docs/pt-BR/INVENTARIO_TECNICO_E_GUIA_DE_OPERACAO.md)** ([English Mirror](docs/en-US/TECHNICAL_INVENTORY_AND_OPERATIONS_MANUAL.md))  
  *Inventário criptográfico exaustivo de todos os artefatos com hashes SHA-256 fixados, indexação dos 15 manuais e roteiro de desbrick U99.*
* 🔍 **[Relatório de Auditoria e Modularização de Código](docs/pt-BR/RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md)** ([English Mirror](docs/en-US/FINAL_AUDIT_AND_MODULARIZATION_REPORT.md))  
  *Relatório de modularização cross-stack (Rust, C, TypeScript, Python), conformidade arquitetural e homologação dos 9 Quality Gates.*
* 📡 **[Especificação Técnica do Subsistema de Logging e Telemetria](docs/pt-BR/SUBSISTEMA_LOGGING_E_TELEMETRIA.md)** ([English Mirror](docs/en-US/LOGGING_AND_TELEMETRY_SUBSYSTEM.md))  
  *Arquitetura do barramento de logs, eliminação de println e console.log, sinks desacoplados e telemetria serial em tempo real.*
* 🔬 **[Teoria e Prática do Desbrick do Erro U99 e Criptografia Matsushita](docs/pt-BR/TEORIA_E_PRATICA_UNBRICK_U99_E_CRIPTOGRAFIA_MATSUSHITA.md)** ([English Mirror](docs/en-US/THEORY_AND_PRACTICE_U99_UNBRICK_MATSUSHITA_CRYPTO.md))  
  *Física de semicondutores da Flash NOR Spansion, motor EEMUL de nivelamento de desgaste e sanitização dos 64 KB de NVRAM.*
* 🛡️ **[Dossiê do Motor RAG Zero-Error e Segurança Física](docs/pt-BR/ZERO_ERROR_RAG_ENGINE_E_SEGURANCA_FISICA.md)** ([English Mirror](docs/en-US/ZERO_ERROR_RAG_ENGINE_AND_PHYSICAL_SAFETY.md))  
  *Arquitetura do motor RAG nativo em Rust, 6 camadas ativas de validação determinística e guardrails contra danos de bancada.*
* 🔬 **[Especificação Técnica e Reconstrução da Interface DWire32A / OCD](dwire32a_jtag_research/PANASONIC_DWIRE32A_JTAG_RECONSTRUCTION.md)**  
  *Documento fundacional do protocolo DWire32A, temporizações do barramento síncrono half-duplex e código assembly da máquina PIO.*
* 🐛 **[Guia de Depuração Interativa de Hardware: GDB, Ghidra e IDA Pro](dwire32a_jtag_research/GUIA_DEPURACAO_GDB_GHIDRA_IDA.md)**  
  *Tutorial prático para conectar depuradores interativos à CPU Matsushita AM34 via ponte TCP 'panadiga pico gdb-bridge'.*

---

## 11. Equipamentos e Consumíveis Recomendados de Laboratório

Para executar as intervenções físicas e lógicas com padrão de excelência de engenharia, recomenda-se a seguinte instrumentação:

* **Programador Universal de Memórias:** XGecu T48 (ou SVOD 4 / TL866II-Plus) equipado com soquete ZIF v3 para TSOP-48.
* **Microcontrolador Host de Bancada:** Raspberry Pi Pico (RP2040) alimentado via USB com chicote de fios AWG 30 (< 15 cm).
* **Estação de Retrabalho SMD:** Estação de ar quente digital com fluxo de ar calibrado e controle estável de temperatura ($340^\circ\text{C}–350^\circ\text{C}$).
* **Ferro de Solda Profissional:** Ponta chanfrada (*knife* ou *hoof*) com controle de temperatura PID a $320^\circ\text{C}$.
* **Insumos Químicos:** Fluxo pastoso *no-clean* tipo Amtech NC-559-ASM, fita térmica Kapton de poliimida e malha dessoldadora trançada com fluxo embutido.
* **Inspeção Óptica:** Microscópio estéreo trinocular com aumento de $7\times$ a $45\times$ para conferência de pontes de solda em circuitos TSOP e SMD.
* **Editor Hexadecimal:** Suíte portátil **HxD Hex Editor** incluída em [`tools/Abrir_HxD.bat`](tools/Abrir_HxD.bat).

---

## 12. Qualidade Total, Pirâmide de Testes e Proteção Anti-Regressão (9 Quality Gates)

Para impedir regressões e garantir que nenhum commit quebre a codebase, o repositório é blindado por uma esteira com **9 Portões de Validação Contínua (9 Quality Gates)** executados via `validate_all.ps1`:

```text
                               PIRÂMIDE DE TESTES DA SUÍTE PANADIGA
                                             ▲
                                            / \
                                           / E2E \   ◄── 8 Testes Playwright (Chromium Headless)
                                         /─────────\      Navegação, Bancada DWire32A, Cirurgia S29JL064H
                                        /  VIEW-    \ ◄── 33 Testes Vitest (jsdom + Testing Library)
                                       /   MODELS    \    useSerialConsole, useInCircuitErase, useGdbBridge, Logger
                                      /───────────────\
                                     / PYTHON & TOOLS  \ ◄── 27 Testes Pytest + Typer CLI + Mypy Strict + MCP Server
                                    /                   \    tools/panadiga_cli.py, RAG Engine, panadiga_mcp_server.py
                                   /─────────────────────\
                                  /   RUST CORE ENGINE    \ ◄── 41 Testes (23 unitários lib/RAG + 18 integração)
                                 /                         \    MEIHDFS Pure FP, Entropia, Cripto FRM, CLI Matrix, SSOT
                                /___________________________\
```

### 12.1. Os 9 Quality Gates Integrados e Ativos

1. **Rust Workspace Check & Tests:** `cargo check --workspace --all-targets` e `cargo test --workspace --verbose` cobrindo **41 testes** (23 unitários lib/RAG + 18 testes de integração) com memory-safety estrita e funções puras sobre slices `&[u8]`.
2. **TypeScript Strict Type-Check:** `npm run typecheck` (`tsc --noEmit`) em `apps/desktop` com zero erros de tipagem.
3. **Vitest Frontend & ViewModels:** `npm test` em `apps/desktop` com **33 testes unitários e de componentes** cobrindo hooks, barramento de log e ViewModels estritos baseados em ADTs.
4. **Copy/Paste Detector (jscpd):** Auditoria cross-language em Rust, TypeScript, Python e C com **limiar estrito de 0% de duplicação** (**0 clones permitidos** em 10.000+ linhas).
5. **UI/UX Completeness Linter (uxlint):** Auditoria automatizada de completude de estados de UI no React (loading, error, empty, feedback) com pontuação **100/100**.
6. **Playwright E2E:** **8 testes end-to-end** com Chromium headless cobrindo navegação, conexão de bancada e simulações de hardware.
7. **Ghidra Native Script Generator Gate:** Exportação idempotente de scripts a partir da SSOT nativa em Rust (`panadiga ghidra export-all`).
8. **Pico C11 Firmware & RSP Server Host Tests:** Compilação e execução dos testes unitários do servidor GDB RSP com GCC `-Werror` (**7 assertions validadas**).
9. **Python Typer CLI, Mypy Strict & Pytest:** Compilação de sintaxe, tipagem estrita com `mypy --strict` (0 erros) e suíte pytest cobrindo **27 testes unitários** (`tools/tests` e `tools/rag_engine/tests`).

### 12.2. Execução da Validação Completa Local

Para executar todos os 9 gates localmente antes de qualquer commit:

```powershell
# No Windows PowerShell:
powershell -ExecutionPolicy Bypass -File validate_all.ps1

# No Windows CMD:
validate_all.bat

# No Linux / macOS / WSL:
./validate_all.sh
```

Para ativar a execução automática pré-commit via Git Hooks:
```bash
git config core.hooksPath .githooks
```

---

## 13. Governança de Inteligência Artificial, Servidor MCP e Logging Estruturado

Para garantir previsibilidade, reprodutibilidade e conformidade física estrita durante o desenvolvimento assistido por modelos generativos (como Claude, Gemini e GPT-4), o ecossistema incorpora infraestrutura completa de agentes e telemetria:

### 13.1. Matriz de Subagentes Especialistas (`.agents/agents/`)
Seis personas especializadas com responsabilidades compartimentadas e guardrails específicos:
* **`panadiga-architect`**: Orquestrador macro, decisões de barramento e integridade do SSOT de constantes em Rust.
* **`panadiga-storage-forensics`**: Engenharia reversa MEIHFS, parsing de MBR "SH", recuperação de partições e journal.
* **`panadiga-firmware-re`**: Desmontagem Matsushita AM34 (MN103S), scripts Ghidra e criptoanálise CPRM/ECB.
* **`panadiga-hardware-embedded`**: Firmware C11/PIO do RP2040, Cold Break-In e circuito elétrico CN55001.
* **`panadiga-desktop-ui`**: Interface Desktop Tauri 2, React 18, Tailwind CSS e testes Vitest/Playwright.
* **`panadiga-qa-auditor`**: Auditoria dos 9 Quality Gates, tolerância zero a duplicação (`jscpd` 0 clones) e segurança.

### 13.2. Servidor Nativo Model Context Protocol (`tools/panadiga_mcp_server.py`)
Servidor JSON-RPC 2.0 stdio configurado em `.mcp.json` e `.agents/mcp_config.json`, expondo ferramentas determinísticas:
* `panadiga_memory_map`: Consulta de regiões e permissões do espaço de endereçamento AM34.
* `panadiga_inspect_hdd`: Inspeção de imagens de disco rígido MEIHFS e conferência de partições.
* `panadiga_clean_nvram`: Sanitização dos 64 KB de NVRAM Spansion para correção do erro U99.
* `panadiga_safety_check`: Validação física estrita (limite de 3.3V LVTTL e bloqueio de escrita no bootloader).
* `panadiga_search_manuals`: Busca automatizada nos 15 manuais de serviço e cadernos de esquemáticos.
* `panadiga_rag_query`: Consulta com guardrails semânticos e citações verificadas.

### 13.3. Subsistema de Logging e Telemetria Unificada (`src/logger.rs`)
Substituição integral de `println!` por telemetria padronizada:
* Filtragem atômica de zero overhead (`is_enabled`) configurável via variável de ambiente `PANADIGA_LOG`.
* Níveis semânticos: `Error`, `Warn`, `Info`, `Debug`, `Trace`, `Off`.
* Suporte a *Custom Sinks* para roteamento em tempo real para janelas de terminal na interface Desktop ou buffers de teste.
* Dossiê de referência técnica disponível em [`docs/SUBSISTEMA_LOGGING_E_TELEMETRIA.md`](docs/SUBSISTEMA_LOGGING_E_TELEMETRIA.md).

---

### 13.4. Interface Desktop Tauri 2 & Arquitetura Visual (100% Offline)
O aplicativo gráfico de bancada Desktop (`apps/desktop`) disponibiliza 7 módulos visuais dedicados, operando inteiramente em modo local e sem qualquer dependência de nuvem:
* **Dashboard:** Telemetria em tempo real do sistema e atalhos rápidos.
* **NVRAM Surgery:** Gráfico interativo dos micro-setores e geração do dump limpo para unbrick do erro `U99`.
* **Storage MEIHFS:** Parser visual de partições de disco rígido, LBA 0 "SH", mirror em 28 MB e journal.
* **Pico Workbench:** Controle de bancada do RP2040, pinagem do conector CN55001, Cold Break-In e console serial.
* **FRM Crypto:** Criptoanálise da TOC de 48B, módulos de firmware ópticos e dicionário ECB de 64 bits.
* **Manuals & Zero-Error RAG AI Assistant:** Consulta com citações diretas dos 15 manuais de serviço com selo verde de validação física e latência < 1 ms.
* **Hex Inspector:** Inspetor hexadecimal em tempo real com alinhamento de setores LBA.

### 13.5. Modelo de Defesa em Duplo Anel (RAG Cognitivo vs. Runtime Hard Defense)
* **Anel 1 (Cognitivo / RAG):** Protege o operador e os agentes de IA de gerarem procedimentos suicidas (bloqueia tensões > 3.3V LVTTL e comandos destrutivos no bootloader).
* **Anel 2 (Runtime Físico / Firmware & Rust):** Protege o circuito caso o operador tente executar comandos manuais destrutivos. O firmware C11 do RP2040 (`gdb_rsp.c`) rejeita tentativas de escrita no IPL com erro `E05`, o motor Rust garante imutabilidade via hashes SHA-256 e os dumps são abertos em modo `ReadOnly` via `memmap2`.

---

## 14. Conformidade e Integridade de Engenharia

O projeto cumpre rigorosamente os princípios de engenharia de software e hardware:
1. **Zero Magic Numbers:** Todos os offsets, tamanhos de bloco, assinaturas e máscaras são estritamente documentados e tipados em [`src/constants.rs`](src/constants.rs).
2. **Portabilidade:** Todos os comandos e scripts utilizam caminhos relativos robustos baseados na raiz do projeto.
3. **Auditoria Criptográfica Contínua:** Todas as modificações em dumps e extrações de partição são validadas através de somas de verificação SHA-256 e MD5.
4. **Arquitetura Modular Limpa:** Decomposição estrita de god-modules no Rust Core ([`src/hdd/`](src/hdd/), [`src/crypto/`](src/crypto/), [`src/pico/`](src/pico/)) e no Desktop Tauri ([`apps/desktop/src/components/`](apps/desktop/src/components/)).
5. **Internacionalização Universal (`i18next`):** Interface 100% internacionalizada com tabelas JSON canônicas ([`en-US.json`](apps/desktop/src/locales/en-US.json) e [`pt-BR.json`](apps/desktop/src/locales/pt-BR.json)).
6. **Acessibilidade Universal (a11y):** Conformidade com WCAG 2.1 AA (navegação por teclado, leitor de tela, `aria-live="polite"` no terminal) e pontuação máxima 100/100 no `uxlint`.
7. **Dossiê Completo de Auditoria:** Detalhado na íntegra em [`docs/RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md`](docs/RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md).

---

**Equipe de Engenharia Reversa, Firmware e Sistemas Proprietários**  
*Panasonic DMR-EH55 Hardware & Systems Engineering Core*  
*Documento homologado para referência e operação de bancada em nível internacional.*
