<p align="center">
  <a href="README.md">English</a> | <strong>Português (Brasil)</strong>
</p>

# Central de Engenharia Reversa, Firmware e Diagnóstico de Hardware — Gravadores Panasonic DIGA

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![SoC](<https://img.shields.io/badge/SoC-Matsushita%20UniPhier%20(AM34%2FMN103S)-blue.svg>)]()
[![Flash](<https://img.shields.io/badge/NOR%20Flash-Spansion%20S29JL%20%2F%20Am29LV%20(2MB--16MB)-green.svg>)]()
[![Storage](<https://img.shields.io/badge/Storage-MEIHDFS%20(IDE%2FPATA%2FSATA)-yellow.svg>)]()
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> **Documento Mestre de Referência Técnica de Sistemas e Hardware**
> **Classificação:** Engenharia Forense / Plataforma Unificada de Recuperação e Ferramentas
> **Dispositivo-Alvo Principal:** Gravador Digital de Vídeo DVD/HDD **Panasonic DMR-EH55** (Chassi DIGA / PCB Principal `VEP79138`)
> **Compatibilidade do Ecossistema:** Linha Panasonic DIGA (**Séries DMR-E, DMR-EH, DMR-ES, DMR-EX, DMR-BW**)
> **Processador:** Matsushita UniPhier `MN2DS0016` / `MN2DS0018` / `MN103S` (Núcleo RISC de 32 bits `AM34` / `MN10300`)
> **Faixa de Memória Flash NOR:** 2 MB (16 Mbit) a 16 MB (128 Mbit) TSOP-48 / TSOP-56
> **Subsistema de Armazenamento:** Discos Rígidos PATA/IDE e SATA com particionamento e sistema de arquivos proprietário **MEIHDFS**
> **Interface de Depuração:** Barramento serial síncrono proprietário **Matsushita DWire32A / On-Chip Debug (OCD)** no conector `P7501` (`CN55001`)

---

## 1. Visão Geral e Apresentação Executiva do Projeto

A linha de gravadores **Panasonic DIGA** representa o ápice da engenharia japonesa de eletrônicos de consumo da era 2001–2008. Projetada pela **Matsushita Electric Industrial Co., Ltd.** (atual Panasonic Corporation), essa família de aparelhos integra processamento de áudio e vídeo de alto desempenho baseado na família de SoCs proprietários **UniPhier**, executando o conjunto de instruções **Matsushita AM34** (família MN10300).

Embora o foco inicial de laboratório deste projeto seja o modelo amplamente difundido **DMR-EH55**, os métodos de engenharia, algoritmos e ferramentas desenvolvidos no Panadiga abrangem **todo o ecossistema da linha Panasonic DIGA**:

1. **Bootloader Híbrido em Duas Fases:** A memória NOR Flash Spansion (`IC55007`, 2 MB a 16 MB dependendo da geração) carrega apenas o estágio inicial de inicialização (*IPL - Initial Program Loader*). O sistema operacional principal, drivers de aceleração gráfica OSD, microcódigos do servo óptico e codecs residem em setores reservados nos primeiros 100 MB do disco rígido interno.
2. **Sistema de Armazenamento Proprietário MEIHDFS:** O disco rígido não utiliza partição MBR padrão PC nem sistemas de arquivos FAT/ext. O layout baseia-se no formato **MEIHDFS** (*Matsushita Electric Industrial HDD File System*), com superbloco na assinatura ASCII `"SH"` no LBA 0, espelho de segurança no LBA 57.344 (28 MB), journal transacional de 10 blocos e partições de firmware cifradas.
3. **Criptografia e Proteção de Propriedade Intelectual:** O contêiner de distribuição óptica (`PANA_DVD.FRM`) e as partições executáveis do HD são blindados com cifras de bloco simétricas de 64 bits em modo ECB, associadas a esquemas CPRM (*Content Protection for Recordable Media*) e DivX DRM.
4. **A Vulnerabilidade Crítica do Erro `U99`:** A calibração de canais, estado de montagem do HDD e parâmetros do gravador são mantidos nos últimos 64 KB da Flash, emulando uma EEPROM (`EEMUL`). Desgaste de células, desligamento abrupto durante gravação ou parâmetros corrompidos provocam falha de validação na inicialização. Como consequência, o firmware Matsushita entra em pânico e trava irreversivelmente com o código **`U99`** no display VFD frontal, rejeitando comandos do controle e travando o mecanismo óptico.

Este repositório consolida o **ecossistema definitivo de engenharia reversa, recuperação física e análise forense** para a linha Panasonic DIGA, disponibilizando a suíte de ferramentas de alta performance em Rust **`panadiga`**, interface de bancada desktop e mobile Android (`.apk`), firmware C11/PIO para microcontroladores **Raspberry Pi Pico (RP2040) e Pico 2 (RP2350)** com suporte a **Bluetooth Low Energy (BLE)** e USB-C OTG, 30 manuais técnicos de fábrica, esquemas elétricos e datasheets de silício, e o acervo completo de 13 grandes dossiês técnicos bilíngues de referência.

---

## 2. Diagrama Mermaid de Arquitetura Ponta-a-Ponta

```mermaid
%%{init: {
  'theme': 'base',
  'themeVariables': {
    'background': '#1c2128',
    'primaryColor': '#282e37',
    'primaryTextColor': '#f0f6fc',
    'primaryBorderColor': '#444c56',
    'lineColor': '#8b949e',
    'secondaryColor': '#1c2128',
    'tertiaryColor': '#1c2128',
    'clusterBkg': '#1c2128',
    'clusterBorder': '#30363d',
    'defaultLinkColor': '#8b949e',
    'titleColor': '#f0f6fc',
    'edgeLabelBackground': '#1c2128'
  }
}}%%
flowchart TD
    subgraph MASTER[" "]
        direction TB

        subgraph LAB_BENCH["1. BANCADA DE LABORATÓRIO E COMUNICAÇÃO"]
            direction LR
            USER["Engenheiro Forense / Operador"]
            BLE["Bluetooth Low Energy (NUS) / USB-C OTG"]
            PICO["Raspberry Pi Pico (RP2040 / RP2350)<br/>Firmware C11 + Máquinas PIO DWire32A"]
            USER <--> BLE <--> PICO
        end

        subgraph SOFTWARE_SUITE["2. ECOSSISTEMA DE SOFTWARE PANADIGA"]
            direction TB
            subgraph UI_TIER["Interfaces de Usuário e Controle"]
                direction LR
                GUI["Panadiga Desktop / Android GUI<br/>(Tauri 2.0 + React 19 + TypeScript)"]
                CLI["Motor Central Unificado Panadiga<br/>(Binário Nativo Rust 1.85+)"]
                GUI <--> CLI
            end

            subgraph CORE_MODULES["Subsistemas do Motor Nativo em Rust"]
                direction LR
                RAG["Motor RAG Zero-Error<br/>(6 Invariantes Físicas)"]
                DW_HAL["HAL DWire32A<br/>(115200 / Sincronismo PIO)"]
                GDB_SRV["Servidor GDB RSP (:3333)<br/>(Cache de Registradores)"]
                MEIHDFS["Parser MEIHDFS<br/>(Storage Forensics)"]
                CRYPTO["Motor de Criptoanálise<br/>(ECB 64-bit & KPA)"]
                NVRAM["Sanitizador de NVRAM<br/>(Limpeza Segura a 0xFF)"]
            end
        end

        subgraph TARGET_HARDWARE["3. HARDWARE ALVO FAMÍLIA PANASONIC DIGA"]
            direction LR
            P7501["Porta P7501 (JST ZR-6)<br/>Pins: VDD, nRESET, SDATA, GND, SCLK, EXTRG"]
            AM34["SoC Matsushita UniPhier MN2DS0016<br/>CPU 32-bit AM34 RISC"]
            FLASH["Flash NOR Paralela (2MB -- 16MB)<br/>IPL + 64 KB NVRAM (len - 64KB)"]
            HDD["Disco Rígido PATA/IDE ou SATA<br/>LBA 0 Superblock (SH) + Partições"]
            P7501 <--> AM34
            AM34 <--> FLASH
            AM34 <--> HDD
        end

        PICO <-->|"Barramento DWire32A 1.0 MHz"| P7501
        UI_TIER <--> CORE_MODULES
        BLE <--> GUI
        CLI <--> DW_HAL
    end

    style MASTER fill:#1c2128,stroke:#30363d,stroke-width:1px
    style LAB_BENCH fill:#1c2128,stroke:#30363d,stroke-width:1px,color:#f0f6fc
    style SOFTWARE_SUITE fill:#1c2128,stroke:#30363d,stroke-width:1px,color:#f0f6fc
    style UI_TIER fill:#1c2128,stroke:#30363d,stroke-width:1px,color:#f0f6fc
    style CORE_MODULES fill:#1c2128,stroke:#30363d,stroke-width:1px,color:#f0f6fc
    style TARGET_HARDWARE fill:#1c2128,stroke:#30363d,stroke-width:1px,color:#f0f6fc

    classDef greyBox fill:#282e37,stroke:#444c56,color:#f0f6fc,stroke-width:1px;
    class USER,BLE,PICO,GUI,CLI,RAG,DW_HAL,GDB_SRV,MEIHDFS,CRYPTO,NVRAM,P7501,AM34,FLASH,HDD greyBox;
```

---

## 3. Árvore Completa do Repositório e Descrição Semântica dos Arquivos

```text
Panadiga/
├── run.cmd                                           <-- Inicializador Rápido Desktop GUI & CLI (Windows)
├── run.sh                                            <-- Inicializador Rápido Desktop GUI & CLI (Linux/macOS)
├── build.cmd                                         <-- Pipeline Mestre de Compilação Paralela (Windows)
├── build.sh                                          <-- Pipeline Mestre de Compilação Paralela (Linux/macOS)
├── validate_all.cmd                                  <-- Script de Execução dos 9 Quality Gates (CMD)
├── validate_all.ps1                                  <-- Script de Validação dos 9 Quality Gates (PowerShell)
├── build-android.cmd                                 <-- Pipeline de Empacotamento do APK Android (Windows)
├── build-android.sh                                  <-- Pipeline de Empacotamento do APK Android (Unix)
├── Cargo.toml                                        <-- Especificação Mestre do Workspace Rust (Edição 2024)
├── tauri.conf.json                                   <-- Configuração Desktop & Mobile Tauri 2.0
├── src/                                              <-- Motor Nativo em Rust de Alta Performance
│   ├── main.rs                                       <-- Ponto de Entrada Unificado (CLI & GUI Tauri)
│   ├── lib.rs                                        <-- Biblioteca Crate Exportando os Módulos Centrais
│   ├── logger.rs                                     <-- Subsistema Unificado de Logging Estruturado
│   ├── manuals.rs                                    <-- Indexador de Manuais & Scanner Regex Ultrarrápido
│   ├── nvram/                                        <-- Parser Multi-Geometria de NVRAM & Emulação EEMUL
│   ├── hdd/                                          <-- Motor de Armazenamento MEIHDFS & Tabela de Partições
│   ├── frm/                                          <-- Motor de Criptoanálise do Contêiner PANA_DVD.FRM
│   ├── rag/                                          <-- Motor RAG Zero-Error & 6 Camadas de Defesa
│   └── desktop/                                      <-- Comandos Nativos Tauri 2.0 & Ponte GDB RSP
├── apps/desktop/                                     <-- Interface Gráfica de Bancada (React 19 + Vite)
│   ├── src/                                          <-- Componentes & ViewModels em Arquitetura MVVM Rigorosa
│   │   ├── components/                               <-- Painéis de Interface Pura (GDB, DWire32A, HexViewer)
│   │   ├── services/                                 <-- Transporte Tipado Tauri IPC & Pontes Nativas
│   │   └── viewmodels/                               <-- Máquinas de Estados Algébricas (Lógica Pure FP)
│   ├── package.json                                  <-- Dependências Node.js & Scripts de Automação
│   └── playwright.config.ts                          <-- Configuração de Testes de Navegador End-to-End
├── docs/                                             <-- Acervo Oficial de Documentação Técnica
│   ├── INDEX.md                                      <-- Portal e Índice Gateway de Documentação
│   ├── en-US/                                        <-- Dossiês Técnicos Canônicos em Inglês (13 Dossiês)
│   ├── pt-BR/                                        <-- Dossiês Técnicos de Bancada em Português (13 Dossiês)
│   └── images/                                       <-- Esquemáticos em Alta Resolução, Fotos e Pinouts
├── manuals_and_schematics/                           <-- Acervo Técnico (30 Manuais, Esquemas & Datasheets)
│   ├── README.md                                     <-- Catálogo Mestre Indexado por Modelo e Disciplina
│   ├── datasheets/                                   <-- Datasheets de Silício (MN103, RP2040, RP2350, Spansion)
│   ├── dmr-eh55/                                     <-- Manuais DMR-EH55 (Serviço, Esquemas, Usuário, Firmware)
│   ├── dmr-eh56/                                     <-- Manual de Operação Europeu DMR-EH56
│   ├── dmr-eh50/                                     <-- Manual de Operação e Uso DMR-EH50
│   ├── dmr-eh65/                                     <-- Manual de Uso da Variante 250 GB DMR-EH65
│   ├── dmr-eh80v/                                    <-- Manual de Serviço Combo VHS+DVD+HDD DMR-EH80V
│   ├── dmr-es10/                                     <-- Manual de Serviço DMR-ES10 (Código U99 & Esquemas)
│   ├── dmr-es30v/                                    <-- Manual de Operação Combo VHS+DVD DMR-ES30V
│   ├── dmr-e55/                                      <-- Manual de Operação Gravador Óptico 2MB DMR-E55
│   ├── dmr-e85h/                                     <-- Manual Pioneiro HDD/DVD 4MB Flash DMR-E85H
│   ├── dmr-e500h/                                    <-- Manual do Gravador Massivo 400 GB DMR-E500H
│   └── dmr-ex98v/                                    <-- Manual de Serviço DVB-T/HDMI Vol 1 DMR-EX98V
├── tools/                                            <-- Ferramentas de Laboratório, Ghidra & Scripts
│   ├── Abrir_HxD.cmd                                 <-- Inicializador Rápido do Editor Hexadecimal HxD
│   ├── conectar_gdb_multiarch.cmd                    <-- Conexão GDB Multiarch ao Alvo AM34
│   ├── executar_openocd.cmd                          <-- Daemon OpenOCD para Depuração AM34
│   ├── iniciar_ponte_gdb.cmd                         <-- Encaminhador TCP do Servidor GDB RSP (:3333)
│   ├── panadiga_cli.py                               <-- CLI Forense Complementar em Python Typer
│   ├── panadiga_mcp_server.py                        <-- Servidor Model Context Protocol (MCP) de Laboratório
│   └── ghidra_am34_memory_map.py                     <-- Carregador de Mapa de Memória e Símbolos Ghidra
└── dwire32a_jtag_research/                           <-- Firmware C11 & Máquinas PIO para RP2040/RP2350
    ├── dwire32a.pio                                  <-- Motor PIO Síncrono Half-Duplex a 1.0 MHz
    ├── dwire32a_hw.c                                 <-- Driver de Hardware e Temporização (100% C11)
    ├── gdb_rsp.c                                     <-- Parser Determinístico do Protocolo GDB RSP
    └── build_pico_firmware.cmd                       <-- Pipeline de Compilação Bare-Metal RP2040/RP2350
```

---

### 3.1. Scripts de Inicialização e Build Multiplataforma (Linux, macOS, Windows)

| Script                          | Plataforma    | Modo                    | Descrição                                                                                                                                                                                                      |
| :------------------------------ | :------------ | :---------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`run.cmd`**           | Windows       | Execução / Auto-Build | Executa o aplicativo Desktop diretamente de `dist/` ou `target/release/`. Caso não compilado, aciona o `build.cmd` automaticamente. Suporta `--dev`, `--cli`, `--validate`.                         |
| **`run.sh`**            | Linux / macOS | Execução / Auto-Build | Equivalente Unix do `run.cmd`. Valida artefatos, compila se necessário e inicia a interface ou CLI.                                                                                                           |
| **`build.cmd`**         | Windows       | Build Release Completo  | Compila bundle React, exporta scripts Ghidra do SSOT, compila o binário Rust otimizado (`%NUMBER_OF_PROCESSORS%` threads), compila firmware do Pico e empacota `dist/releases/panadiga-windows-x86_64.zip`. |
| **`build.sh`**          | Linux / macOS | Build Release Completo  | Pipeline Unix paralelo completo de compilação, empacotamento e hashes SHA-256.                                                                                                                                 |
| **`validate_all.cmd`**  | Windows       | 9 Quality Gates         | Executa a bateria de 9 quality gates via CMD nativo.                                                                                                                                                             |
| **`validate_all.ps1`**  | Windows       | 9 Quality Gates         | Runner PowerShell avançado com saída colorida e verificação estrita de regressões.                                                                                                                          |
| **`build-android.cmd`** | Windows       | APK Mobile Android      | Compila o target Android (`aarch64-linux-android`) via Tauri 2 Mobile com suporte nativo a BLE e USB-C OTG.                                                                                                    |
| **`build-android.sh`**  | Linux / macOS | APK Mobile Android      | Versão Unix do pipeline de compilação Android.                                                                                                                                                                |

---

## 4. Guia de Uso Completo da Ferramenta Rust `panadiga` (`src/`)

O binário compilado `panadiga` opera tanto como servidor gráfico de bancada Desktop quanto utilitário de linha de comando de alta velocidade.

### 4.1 Compilação em Modo Release

```bash
# Compilação do binário unificado de produção
cargo build --release -p panadiga

# Execução direta com o launcher unificado:
./run.cmd --help       # Windows
./run.sh --help        # Linux / macOS
```

### 4.2 Comando 1: Limpeza Cirúrgica da NVRAM Multi-Geometria (`nvram clean`)

Executa a sanitização do setor circular de emulação EEPROM (`EEMUL`) dentro da memória Flash NOR sem alterar o bootloader (*IPL*). Detecta dinamicamente a geometria do arquivo (2 MB, 4 MB, 8 MB, 16 MB) e calcula o setor de 64 KB em `len - 64 KB`.

```bash
# Dump de 8 MB (DMR-EH50 / EH55 / EH56 / EH60 / EH65 / EH80V):
panadiga nvram clean --input dump_flash_8mb.bin --output flash_recuperada.bin

# Dump de 4 MB (DMR-E85H / E95H / ES20):
panadiga nvram clean --input dump_flash_4mb.bin --output flash_recuperada.bin

# Dump de 2 MB (DMR-E50 / E55 / ES10):
panadiga nvram clean --input dump_flash_2mb.bin --output flash_recuperada.bin

# Dump de 16 MB (DMR-EX75 / EX85 / EX98V):
panadiga nvram clean --input dump_flash_16mb.bin --output flash_recuperada.bin
```

* **Byte de Sanitização:** Estado virgem `0xFF` (padrão de células apagadas em memórias Flash NOR).
* **Invariante de Silício:** Jamais modifica os primeiros setores (`0x00000000` até `len - 65537`), preservando integralmente o IPL de fábrica.

### 4.3 Comando 2: Inspeção Estrutural do Disco Rígido (`hdd inspect`)

Analisa a geometria e integridade do particionamento MEIHDFS, localiza o superbloco `"SH"` no LBA 0, valida o espelho de segurança em 28 MB (LBA 57.344) e detalha as fronteiras das partições.

```bash
panadiga hdd inspect --input imagem_hd.bin
```

### 4.4 Comando 3: Extração Cirúrgica de Partições (`hdd extract`)

Extrai as partições executáveis brutas do disco rígido MEIHDFS para análise forense no Ghidra ou IDA Pro.

```bash
panadiga hdd extract --input imagem_hd.bin --output-dir particoes/
```

### 4.5 Comando 4: Criptoanálise Forense do Contêiner Óptico (`frm analyze`)

Audita o contêiner óptico de atualização `PANA_DVD.FRM`, avaliando a entropia por blocos, registros da TOC e padrões de vazamento do modo ECB.

```bash
panadiga frm analyze --input firmware_and_images/PANA_DVD.FRM
```

### 4.6 Comando 5: Decodificação de Descritores Bit-Invertidos MEIHDFS (`hdd decode-descriptors`)

Decodifica os descritores de partição gravados com inversão bit a bit (Complemento de Um / XOR 0xFF) no superbloco MEIHDFS.

```bash
panadiga hdd decode-descriptors --input hdd_100.bin
```

### 4.7 Comando 6: Dump e Desconstrução da TOC e Módulos do `.FRM` (`frm dump-toc`)

Desconstrói o contêiner `PANA_DVD.FRM` em seus três módulos funcionais: Sistema Operacional Principal da CPU (AM34), Servo Controlador Óptico (DSP MN102) e Camada Gráfica OSD.

```bash
panadiga frm dump-toc --input firmware_and_images/PANA_DVD.FRM
```

### 4.8 Comando 7: Consulta e Auditoria do Codebook ECB de 64 bits (`frm codebook`)

Cataloga os blocos de 64 bits de maior frequência entre o contêiner óptico e as partições do HD para comprovar a identidade das chaves criptográficas.

```bash
panadiga frm codebook --frm firmware_and_images/PANA_DVD.FRM --hdd firmware_and_images/hdd_100.bin
```

### 4.9 Comando 8: Inspeção de Cabeçalhos e Hex Dumper (`headers`)

```bash
panadiga headers --input firmware_and_images/hdd_100.bin
```

### 4.10 Comando 9: Varredura Forense de Strings e Palavras-Chave (`scan`)

```bash
panadiga scan --input firmware_and_images/PANA_DVD.FRM --terms U99,FIRMWARE,PANASONIC
```

### 4.11 Comando 10: Indexação e Busca em Manuais de Serviço e Esquemas Técnicos (`manuals`)

Pesquisa nos 30 manuais técnicos de serviço, esquemas elétricos e datasheets através de busca recursiva ultrarrápida.

```bash
panadiga manuals --query "P7501"
panadiga manuals --query "U99" --filter "dmr-es10"
panadiga manuals --query "MN103" --filter "datasheets"
```

### 4.12 Comando 11: Consulta ao Motor RAG Zero-Error e Guardrails Físicos (`rag query`)

Executa consultas técnicas fundamentadas com proteção estrita contra alucinações e limites elétricos garantidos por 6 invariantes de silício.

```bash
panadiga rag query --question "Qual o pino de terra canônico do conector P7501?"
panadiga rag query --question "What is the canonical ground pin of P7501?" --json
```

---

## 5. Especificação Elétrica e Pinout da Interface DWire32A (`P7501`)

### 5.1 Pinout Elétrico Canônico, Mapeamento Físico de Bancada e Conexão Universal RP2040

Na placa digital `VEP79138` (e placas da família `VEP79xxx` nos gravadores da série DMR-EH), o conector de depuração (marcado no esquemático oficial da Panasonic como **`P7501 [B6B-ZR-3R4]`**, legenda *"TO WRITE JIGU"*) situa-se fisicamente na borda superior da placa, posicionado entre o cabo flat de 10 vias (`FP50003`) à esquerda e o de 8 vias (`FP50004`) à direita.

> [!CAUTION]
> **REGRA DE SEGURANÇA ELÉTRICA — VALIDAÇÃO MANDATÓRIA DO TERRA (`GND`):**
>
> * **No Manual de Serviço Oficial da Panasonic (Esquemático do Conector P7501):** O **`GND` é ESTRITAMENTE O PINO 4**.
> * **Pino 1 (VDD Sense) situa-se junto ao cabo flat de 8 vias (`FP50004`).**
> * **Pino 6 (EXTRG) situa-se junto ao cabo flat de 10 vias (`FP50003`), caracterizado pelo pad quadrado na serigrafia da PCB.**
> * **NUNCA conecte o terra no Pino 1 ou Pino 6** (o que causaria curto-circuito na linha de alimentação de 3.3V ou no gate de interrupção da CPU). Antes de ligar o Raspberry Pi Pico ou qualquer ferramenta de bancada, **sempre valide com o multímetro o bipe sonoro de $0.0\,\Omega$ de continuidade direta entre o Pino 4 e a carcaça de lata metálica do chassi!**

<p align="center">
  <img src="docs/images/panasonic_dmr_eh55_pcb_p7501_orientation.png" alt="Topografia Física e Orientação do Conector P7501 na Placa VEP79138" width="720" />
  <br/>
  <em>Figura 5.1: Topografia física da PCB digital VEP79138 delimitando o conector P7501 entre os cabos flat FP50003 (10 vias) e FP50004 (8 vias).</em>
</p>

```text
       ORIENTAÇÃO MECÂNICA CANÔNICA DO CONECTOR P7501 [B6B-ZR-3R4] (ESQUEMÁTICO PANASONIC)
       Parede Traseira voltada para o interior da placa (acima) | Janela de trava voltada para a borda frontal (abaixo)
     
       Cabo Flat 10 Vias (FP50003)                                         Cabo Flat 8 Vias (FP50004)
       [À Esquerda]                                                                      [À Direita]
            ┌────────────────────────────────────────────────────────────────────────┐
     Pino 6 │ ■ EXTRG   [Pad Quadrado] Gatilho Externo OCD / Breakpoint              │
     Pino 5 │ ● SCLK    [Pad Redondo]  Clock Síncrono DWire32A (Master 1.0 MHz)      │
     Pino 4 │ ● GND     [Pad Redondo]  Terra Canônico (0.0Ω Contínuo ao Chassi!)     │
     Pino 3 │ ● SDATA   [Pad Redondo]  Dados Half-Duplex (Pull-up 4.7kΩ p/ 3.3V)     │
     Pino 2 │ ● nRESET  [Pad Redondo]  Linha de Reset Mestre (Controle Open-Drain)   │
     Pino 1 │ ● VDD     [Pad Redondo]  Target Power Sense (+3.3V Referência)         │
            └────────────────────────────────────────────────────────────────────────┘
```

<p align="center">
  <img src="docs/images/schematic_p7501_panasonic_official.png" alt="Esquemático Oficial Panasonic P7501" width="460" />
  <br/>
  <em>Figura 5.2: Recorte do esquemático oficial da Panasonic DMR-EH55 (VEP79138) detalhando o barramento P7501 [B6B-ZR-3R4] "TO WRITE JIGU".</em>
</p>

> [!NOTE]
> **Compatibilidade de Hardware RP2040 / RP2350:** Qualquer placa equipada com **RP2040** (Pico, Pico W) ou **RP2350** (Pico 2) é 100% suportada. A temporização de nanossegundos do barramento DWire32A é executada diretamente no hardware pelas máquinas de estados **PIO0**.

| Pino P7501 (Canônico) | Sinal Matsushita |     Pino RP2040 / Pico     |            Componente de Proteção / Condicionamento            |        Tipo de Sinal        | Veredito de Bancada & Função Técnica                                                                                                         |
| :--------------------: | :---------------: | :-------------------------: | :--------------------------------------------------------------: | :--------------------------: | :---------------------------------------------------------------------------------------------------------------------------------------------- |
|    **Pino 1**    |   `VDD Sense`   |  `GP26 / ADC0` (Pino 31)  |       Resistor $1.0	ext{ k}\Omega$ em série (Opcional)       |      Entrada Analógica      | Monitoramento da presença de $+3.3	ext{V}$ da placa do gravador. **Proibido alimentar o Pico por este pino ($I < 10	ext{ mA}$).**    |
|    **Pino 2**    |    `nRESET`    |      `GP5` (Pino 7)      |                Resistor $100\,\Omega$ em série                | **Open-Drain Estrito** | Linha de reset mestre da CPU (`XRST` / `D_RESET`). **Drenar para 0V para resetar; deixar em Hi-Z para operar. Nunca 3.3V Push-Pull!** |
|    **Pino 3**    |     `SDATA`     |      `GP4` (Pino 6)      | Resistor $100\,\Omega$ + Pull-up $4.7	ext{ k}\Omega$ p/ 3.3V |    Bidirecional Síncrono    | Barramento de dados DWire32A em modo half-duplex. Subida rápida garantida pelo pull-up em cabos longos.                                        |
|    **Pino 4**    | **`GND`** | **Pino 38 (`GND`)** |         **Ligação direta de baixa impedância**         | **Referência de 0V** | **Equipotencial mandatória: $0.0\,\Omega$ com a carcaça de lata metálica desprovida de tinta do chassi!**                            |
|    **Pino 5**    |     `SCLK`     |      `GP3` (Pino 5)      |                Resistor $100\,\Omega$ em série                |    Saída de Clock Master    | Clock serial síncrono mestre gerado pelo Pico PIO (Frequência operacional: $1.0	ext{ MHz}$).                                                |
|    **Pino 6**    |     `EXTRG`     |      `GP2` (Pino 4)      |                Resistor $100\,\Omega$ em série                |  Saída Digital de Trigger  | Gatilho externo para forçar parada de execução (*Halt Request*) do núcleo AM34 no Cold Break-In.                                          |

<p align="center">
  <img src="docs/images/dwire32a_pico_probe_assembly_schematic.png" alt="Esquema Elétrico Normatizado de Montagem da Sonda Pico DWire32A" width="760" />
  <br/>
  <em>Figura 5.3: Esquema elétrico normatizado (IEEE Std 315 / IEC 60617) para montagem da sonda de bancada conectando o Raspberry Pi Pico ao conector P7501.</em>
</p>

---

### 5.2 Regras Críticas de Engenharia e Prevenção de Danos

1. **Regra 1 (Lógica Estrita LVCMOS 3.3V):** As portas do processador MN2DS0016 toleram tensão absoluta máxima de $+3.6	ext{V}$. Sondas TTL de 5.0V (como gravadores CH341A sem mod de 3.3V ou Arduinos convencionais) queimam as portas de entrada instantaneamente.
2. **Regra 2 (Open-Drain no `nRESET`):** O acionamento de `nRESET` via push-pull de 3.3V danifica o circuito integrado supervisor de reset (`IC55005`). Deve-se aterrar para resetar e manter em alta impedância (Hi-Z) para liberar a execução.
3. **Regra 3 (Validação Mandatória de Continuidade do Terra):** Meça $0.0\,\Omega$ entre o Pino 4 do conector e o chassi metálico antes de conectar qualquer cabo serial ou de alimentação.
4. **Regra 4 (Dispensa de Dessolda):** Conecte diretamente no conector P7501 com soquete fêmea JST ZR de 1.5mm. Dessoldar o encapsulamento TSOP-48 em placa de multicamadas acarreta severo risco de delaminação das trilhas.
5. **Regra 5 (Sequência de Cold Break-In):** Para interceptar a CPU antes do disparo do watchdog do IPL, ative `EXTRG` em nível alto e pulse `nRESET` em 0V por $10	ext{ ms}$, entrando em modo OCD no ciclo zero de clock.

---

### 5.3 Diagrama de Temporização e Sinais DWire32A

```text
Clock Cycle:       1     2     3     4     5     6     7     8
                 _____       _____       _____       _____
SCLK   _________/     \_____/     \_____/     \_____/     \_____  (Master 1.0 MHz)
     
SDATA  ===< CMD[7] >< CMD[6] >< CMD[5] >...========[ Hi-Z Turnaround ]===> DATA[7] >
                 ^           ^
                 |           |-- Dado amostrado na borda de SUBIDA do SCLK
                 |-------------- Dado comutado na borda de DESCIDA do SCLK

Restrições de Temporização:
  * t_CLK_PERIOD = 1000 ns (Frequência nominal: 1.0 MHz)
  * t_SETUP      = 250 ns mínimo antes da borda de subida do SCLK
  * t_HOLD       = 250 ns mínimo após a borda de subida do SCLK
  * t_TURNAROUND = 2 ciclos completos de SCLK (2000 ns) para inversão de sentido de barramento
```

---

## 6. Mapa de Memória Físico da Arquitetura Matsushita AM34 (`MN2DS0016`)

```text
+-------------------+-------------------+--------------------+-------------------------------------------+
| Faixa Física      | Faixa Espelhada   | Tipo de Barramento | Descrição Funcional                       |
+-------------------+-------------------+--------------------+-------------------------------------------+
| 0x00000000..0FFF  | 0x40000000..0FFF  | I-RAM/SRAM Interna | 4 KB Scratchpad e Vetores de Exceção      |
| 0x00400000..07FF  | 0x40400000..07FF  | D-RAM Interna      | 64 KB Memória Rápida de Dados do Sistema  |
| 0x04000000..07FF  | 0x44000000..07FF  | Controlador DDR    | 64 MB Memória Principal do Sistema (DDR)  |
| 0x08000000..0FFF  | 0x48000000..0FFF  | Framebuffer MPEG   | 32 MB Memória de Vídeo Dedicada / OSD     |
| 0x10000000..10FF  | 0x50000000..10FF  | Mestre IDE / PATA  | Registradores ATA/ATAPI-5 (HDD e Drive DVD)|
| 0x18000000..187F  | 0x58000000..187F  | CS0 Externo (NOR)  | Flash Spansion S29JL064H (8 MB - IPL)     |
| 0x187F0000..187F  | 0x587F0000..187F  | CS0 Externo (NV)   | 64 KB Setor de NVRAM/EEMUL (Alvo do U99)  |
| 0x1F000000..1FFF  | 0x5F000000..1FFF  | MMIO Interno SoC   | Registradores de Periféricos, Timers e OCD|
+-------------------+-------------------+--------------------+-------------------------------------------+
```

### 6.1 Regra de Ouro JEDEC para o RAM Loader Injetado

Ao carregar payloads executáveis diretamente na memória DDR-SDRAM via comandos DWire32A OCD:

* **Endereço de Carga do Payload:** `0x04010000` (offset seguro de buffer DDR, evitando a página zero).
* **Ponteiro de Pilha Inicial:** `SP = 0x0040FFF0` (dentro da SRAM de alta velocidade interna do chip).
* **Desativação do Watchdog:** Escrever `0x00000000` no registrador MMIO `0x1F000104` em menos de $100	ext{ ms}$ após o boot.

---

## 7. Síntese das Descobertas Criptográficas e Forenses (Forensic Cryptanalysis & Storage Deep-Dive)

### 7.1 O Modo ECB e o Vazamento Estrutural ("ECB Leak")

A análise estatística do arquivo `PANA_DVD.FRM` e dos dumps do disco rígido revelou que a Matsushita adotou criptografia de bloco simétrica de 64 bits sem encadeamento de blocos (CBC). Blocos idênticos de 8 bytes no texto claro geram blocos idênticos no texto cifrado, expondo tabelas e estruturas internas.

### 7.2 Codebook ECB de 64 bits e Ataque de Texto-Plano Conhecido (Known-Plaintext Attack - KPA)

| Bloco Cifrado (Hex) | Significado em Texto Claro | Ocorrências FRM | Ocorrências HD | Relevância Forense                                 |
| :------------------: | :-------------------------: | :--------------: | :-------------: | :-------------------------------------------------- |
| `035065C87BB8E090` | `FF FF FF FF FF FF FF FF` |      3.812      |      1.280      | Células virgens de Flash NOR apagada / Padding     |
| `E5D1F34A518B2690` | `00 00 00 00 00 00 00 00` |       196       |        3        | Padding nulo de zeros / Zeros estruturais           |
| `4D3C5DBB0C357777` |    Cabeçalho de Tabela    |      1.280      |        0        | Quadro descritor de módulo de partição interna   |
| `EFDC1CDA13881BAB` |   Cabeçalho de Família   |        3        |        0        | TOC: Marcador de Família de Processadores UniPhier |
| `74A78ACD789B84FF` |     Seletor de Core CPU     |        2        |        0        | TOC: Identificador de Núcleo AM34 / MN103S         |
| `3B7E7EDBB6C57A53` |     Seletor de Core DSP     |        1        |        0        | TOC: DSP do Servo do Drive Óptico MN102            |

### 7.3 Desconstrução do Contêiner `PANA_DVD.FRM` em 3 Módulos de Firmware

O contêiner do disco óptico de atualização divide-se em três módulos precedidos por um cabeçalho global de 16 bytes (`492F0560CEDE508CE3C08AFD49B64A9A`) e uma TOC de 144 bytes (três registros de 48 bytes):

1. **Módulo 0 (`MOD_0`):** Kernel do Sistema Operacional para o processador AM34 (SO, GUI, drivers).
2. **Módulo 1 (`MOD_1`):** Microcódigo do servo óptico para o processador MN102 DSP.
3. **Módulo 2 (`MOD_2`):** Recursos gráficos OSD, tabelas de fontes, primitivas e textos de menus.

### 7.4 Vínculo Criptográfico Irrefutável entre o CD (`.FRM`) e o HD (`hdd_100.bin`)

A presença de blocos cifrados idênticos `E5D1F34A518B2690` (zeros) e `035065C87BB8E090` (Flash virgem) tanto no contêiner óptico quanto nas partições do HD comprova que:

1. O firmware mantido no HD é cifrado com o **mesmo algoritmo e mesma chave** do contêiner óptico.
2. Não é possível reparar um HD simplesmente injetando dados em texto-claro; a estrutura precisa respeitar a chave de cifragem e geometria do MEIHDFS.

### 7.5 Arquitetura Bit-Invertida (One's Complement / XOR 0xFF) do MEIHDFS

O superbloco no LBA 0 armazena a geometria com inversão bit a bit (`XOR 0xFF`):

* O identificador ASCII `"SH"` aparece em hexadecimal como `0xAE 0xB7` (`~0x51 0x48`).
* Contadores de LBA e clusters são igualmente invertidos, impedindo que utilitários de partição convencionais de PC identifiquem ou corrompam o disco.

---

## 8. Guia Prático de Resolução do Erro U99 em 3 Rotas de Engenharia

```text
                   +---------------------------------------------+
                   |       PANASONIC DMR-EH55 ERRO U99          |
                   |      (Pânico Permanente no Display VFD)     |
                   +----------------------+----------------------+
                                          |
                +-------------------------+-------------------------+
                |                         |                         |
                v                         v                         v
     [ROTA A: LABORATÓRIO]      [ROTA B: IN-CIRCUIT OCD]   [ROTA C: CLONE FORENSE]
     Dessoldagem TSOP-48 Flash  Conexão RP2040 no P7501    Clone bit a bit do HDD
     Gravador T48 / SVOD4       Injeção de RAM Loader      Limpeza Setor 0x7F0000
     Limpeza Direta 0x7F0000    Limpeza Cirúrgica em RAM   Preservação 100 MB Sistema
```

### Rota A: Gravação Física Externa TSOP-48 via SVOD 4 / XGecu T48

* Dessolde `IC55007` (Spansion S29JL064H70TFI00).
* Faça leitura de backup de 8 MB.
* Execute `panadiga nvram clean --input dump.bin --output limpo.bin`.
* Regrave `limpo.bin` no chip e solde de volta na placa.

### Rota B: In-Circuit via DWire32A com RP2040 e Injeção de RAM Loader

* Conecte a sonda de 6 fios do Pico ao conector `P7501` (confirmando pino 4 GND).
* Execute `panadiga pico flash --unbrick`.
* O Pico interrompe a CPU AM34, injeta o RAM loader e redefine a NVRAM para `0xFF`.

### Rota C: Substituição do HD por SSD Preservando os 100 MB do MEIHDFS

* Conecte o disco IDE a um adaptador USB com alimentação dedicada.
* Extraia a partição do sistema: `panadiga hdd extract --input /dev/sdb --output-dir backup/`.
* Grave em SSD compatível e reinsira no aparelho.

---

## 9. Matriz Completa de Compatibilidade e Comparação Arquitetural da Linha Panasonic DIGA

A tabela a seguir especifica detalhadamente a arquitetura de silício, capacidade de Flash, endereçamento da NVRAM e métodos de recuperação aplicáveis em todas as 5 gerações de gravadores Panasonic DIGA:

| Família & Modelos de Gravadores                                                                                | Geração / Ano                                      | Processador SoC & Arquitetura                                                     | Memória Flash NOR (Tamanho & Chip)                     | Faixa de Endereço da NVRAM (EEMUL)         | Subsistema de Armazenamento                       | Conector de Depuração DWire / OCD                                                                | Causa do Erro U99 & Solução Panadiga                                                                                         |
| :-------------------------------------------------------------------------------------------------------------- | :--------------------------------------------------- | :-------------------------------------------------------------------------------- | :------------------------------------------------------ | :------------------------------------------ | :------------------------------------------------ | :------------------------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------- |
| **DMR-E20, DMR-E30, DMR-E50, DMR-E55, DMR-E60**                                                           | **Geração 1** (2001–2003)                   | Matsushita MN10300 (16/32-bit AM32 / AM33)                                        | **2 MB** (16 Mbit) TSOP-48 (Am29LV160)            | `0x001F0000..0x001FFFFF` (Últimos 64 KB) | Gravador Óptico DVD-RAM / DVD-R (UDF 1.5 direto) | Conector OCD de 5 ou 6 pinos (3.3V)                                                                | Corrupção de wear-leveling na Flash. Limpeza via `panadiga nvram clean` (auto-detecta 2 MB).                               |
| **DMR-E80H, DMR-E85H, DMR-E95H, DMR-E100H, DMR-E500H, DMR-ES20**                                          | **Geração 2** (2003–2004)                   | Matsushita MN103S / Primeiros UniPhier                                            | **4 MB** (32 Mbit) TSOP-48 (Am29LV320)            | `0x003F0000..0x003FFFFF` (Últimos 64 KB) | HDD PATA/IDE (40–160 GB) + MEIHDFS-V1            | Barra de pinos OCD de 6 vias (3.3V LVCMOS)                                                         | Berço do erro `U99` histórico. Limpeza do setor `0x3F0000` via `panadiga nvram clean` (auto-detecta 4 MB).             |
| **DMR-EH50, DMR-EH52, DMR-EH55, DMR-EH56, DMR-EH60, DMR-EH65, DMR-EH75V, DMR-EH80V, DMR-ES10, DMR-ES30V** | **Geração 3** (2005–2007) [Plataforma Base] | Matsushita UniPhier `MN2DS0016` (32-bit RISC AM34)                              | **8 MB** (64 Mbit) TSOP-48 (Spansion S29JL064H)   | `0x007F0000..0x007FFFFF` (Últimos 64 KB) | HDD PATA/IDE (80–250 GB) + MEIHDFS-V2            | Conector Canônico **`P7501`** (JST ZR 1.5mm 6 vias: VDD, nRESET, SDATA, GND, SCLK, EXTRG) | Erro clássico `U99` por dessincronia do anel EEMUL. Solução via `panadiga nvram clean` ou in-circuit via Pico DWire32A. |
| **DMR-EX75, DMR-EX85, DMR-EX95V, DMR-EX98V, DMR-EH67, DMR-EH68**                                          | **Geração 4** (2006–2008)                   | Matsushita UniPhier 2ª Geração (`MN2DS0018` / `MN2DS0022` / `MN2DS0024`) | **16 MB** (128 Mbit) TSOP-56 (Spansion S29GL128N) | `0x00FF0000..0x00FFFFFF` (Últimos 64 KB) | HDD SATA / PATA (160–320 GB) + MEIHDFS-V2+       | Barra DWire32A de 6 vias (JST ZR 1.5mm)                                                            | Dessincronia de tabelas de canal DVB-T gerando U99. Limpeza via `panadiga nvram clean` (auto-detecta 16 MB).                 |
| **DMR-BW500, DMR-BR500, DMR-BW700, DMR-BW800, DMR-BWT700**                                                | **Geração 5** (2008–2011)                   | Matsushita UniPhier Dual-Core (`MN2WS0040` / `MN2WS0050`)                     | **32 MB -- 64 MB** Flash NOR/NAND                 | Alocação dinâmica por volume             | HDD SATA 3.5" (250 GB -- 1 TB) + Drive BD-RE      | Interface híbrida JTAG / DWire                                                                    | Geração Blu-ray. MEIHDFS-V3 com cifras AES. Extração de partições via `panadiga hdd extract`.                          |

### 9.1 Por Que Funciona em Todos os Modelos DIGA ("Invariantes de Silício")

1. **Invariante Matemática de Dimensionamento de Flash (`len - 64 KB`):**Os engenheiros de firmware da Matsushita alocaram consistentemente o anel de emulação de EEPROM (`EEMUL`) no último setor de 64 KB da Flash NOR principal em todas as gerações. O comando `panadiga nvram clean` avalia dinamicamente `offset = len - 65536`. Portanto, seja em dumps de 2 MB (`0x1F0000`), 4 MB (`0x3F0000`), 8 MB (`0x7F0000`) ou 16 MB (`0xFF0000`), a limpeza cirúrgica atua com 100% de precisão sem risco ao IPL.
2. **Invariância do Protocolo DWire32A:**O módulo de On-Chip Debug (OCD) serial integrado à família de núcleos AM3x/MN103 segue a mesma máquina de estados síncrona: clock mestre de $1.0	ext{ MHz}$ (`SCLK`), 250 ns de setup, 500 ns de hold, linha de dados bidirecional half-duplex com pull-up (`SDATA`) e reset open-drain (`nRESET`). A máquina PIO do Raspberry Pi Pico (`dwire32a.pio`) é universalmente compatível com todos os gravadores equipados com esse barramento.
3. **Invariância do Superbloco MEIHDFS:**
   A assinatura mágica ASCII `"SH"` e a geometria com espelho de segurança em 28 MB são idênticas desde a série E85H até as séries digitais EH e EX.

---

## 10. Acervo de Manuais Técnicos, Esquemas e Datasheets (30 Documentos Catalogados)

Todos os manuais de serviço, esquemas e datasheets estão organizados hierarquicamente em `manuals_and_schematics/`:

### A. Manuais de Serviço e Esquemas Técnicos Panasonic DIGA

* `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_vol2_schematics.pdf` (177 págs, esquemas placa VEP79138, pinout P7501)
* `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_vol1_diagnostics.pdf` (97 págs, fluxo de autodiagnóstico e erro U99)
* `dmr-eh55/service/panasonic_dmr_eh55p9_eh55pc9_service_manual_north_america.pdf` (97 págs, chassi NTSC norte-americano)
* `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_europe.pdf` (92 págs, chassi PAL europeu)
* `dmr-eh55/service/panasonic_dmr_eh55ee_service_manual_east_europe.pdf` (91 págs, chassi SECAM leste-europeu)
* `dmr-eh80v/service/panasonic_dmr_eh80v_service_manual_europe.pdf` (12.2 MB, esquemas combo triplo VHS+DVD+HDD)
* `dmr-es10/service/panasonic_dmr_es10_service_manual_north_america.pdf` (5.3 MB, manual de serviço DVD-RAM, erro U99 na pág. 13)
* `dmr-ex98v/service/panasonic_dmr_ex98v_service_manual_vol1.pdf` (10.4 MB, manual de serviço combo digital DVB-T/HDMI Vol 1)
* `dmr-eh55/firmware_update/panasonic_dmr_eh55_firmware_installation_guide.pdf` (8 págs, guia oficial de atualização por CD-R)

### B. Manuais de Usuário e Operação Panasonic DIGA

* `dmr-eh55/user/panasonic_dmr_eh55_operating_instructions_global.pdf` (89 págs)
* `dmr-eh55/user/panasonic_dmr_eh55s_operating_instructions_silver.pdf` (89 págs)
* `dmr-eh55/user/panasonic_dmr_eh55_eh56_operating_instructions_europe.pdf` (84 págs)
* `dmr-eh55/user/panasonic_diga_dmr_eh55_user_manual.pdf` (84 págs)
* `dmr-eh55/user/panasonic_dmr_eh55_user_manual_oceania.pdf` (76 págs)
* `dmr-eh55/user/panasonic_dmr_eh55_australia_new_zealand_manual.pdf` (76 págs)
* `dmr-eh55/user/panasonic_dmr_eh55_hdmi_hdavi_control_guide.pdf` (6 págs)
* `dmr-eh55/user/panasonic_dmr_eh55_quick_setup_guide.pdf` (2 págs)
* `dmr-eh56/user/panasonic_dmr_eh56_operating_instructions.pdf` (89 págs)
* `dmr-eh50/user/panasonic_dmr_eh50_user_manual.pdf` (4.1 MB)
* `dmr-eh65/user/panasonic_dmr_eh65_user_manual.pdf` (4.8 MB)
* `dmr-eh80v/service/panasonic_dmr_eh80v_service_manual_europe.pdf` (12.2 MB)
* `dmr-e85h/user/panasonic_dmr_e85h_user_manual.pdf` (2.7 MB)
* `dmr-e55/user/panasonic_dmr_e55_operating_instructions.pdf` (3.2 MB)
* `dmr-e500h/user/panasonic_dmr_e500h_user_manual.pdf` (4.9 MB)
* `dmr-es30v/user/panasonic_dmr_es30v_user_manual.pdf` (7.9 MB)

### C. Datasheets de Silício e Manuais de Arquitetura (`datasheets/`)

* `datasheets/matsushita_mn10300_series_cross_assembler_users_manual.pdf` (Conjunto de instruções AM34, registradores, opcodes)
* `datasheets/matsushita_mn10300_series_c_debugger_users_manual.pdf` (Protocolo de depuração On-Chip Debug OCD em linguagem C)
* `datasheets/rp2040_datasheet_raspberrypi.pdf` (5.3 MB, especificação técnica oficial do microcontrolador RP2040 e máquinas PIO)
* `datasheets/rp2350_datasheet_raspberrypi.pdf` (11.1 MB, especificação técnica oficial do novo microcontrolador RP2350 - Pico 2)
* `datasheets/spansion_am29lv640d_nor_flash_datasheet.pdf` (1.0 MB, Flash NOR Paralela 64 Mbit TSOP-48, comandos JEDEC)
* `datasheets/spansion_s29gl128n_nor_flash_datasheet.pdf` (4.2 MB, Flash NOR MirrorBit 128 Mbit TSOP-56 para a série DMR-EX)

---

## 11. Links Diretos para o Acervo de Dossiês Técnicos Especializados (`docs/`)

O acervo oficial de 13 grandes dossiês técnicos bilíngues está disponível em `docs/pt-BR/` (com espelho em inglês em `docs/en-US/` e portal em `docs/INDEX.md`):

* 🏛️ **[Arquitetura de Sistemas e Blueprint de Engenharia](docs/pt-BR/ARQUITETURA_E_BLUEPRINT_DE_ENGENHARIA.md)** ([English Mirror](docs/en-US/SYSTEM_ARCHITECTURE_AND_ENGINEERING_BLUEPRINT.md))*Especificação mestre do ecossistema: decomposição de monólitos, Pure FP domain, MVVM rigoroso, anel GDB e invariantes de silício.*
* 📱 **[Arquitetura Mobile Android e Bluetooth](docs/pt-BR/ARQUITETURA_MOBILE_ANDROID_E_BLUETOOTH.md)** ([English Mirror](docs/en-US/ANDROID_APK_AND_BLUETOOTH_ARCHITECTURE.md))*Construção de APK Android (`aarch64-linux-android`), HAL nativa em Rust (BLE NUS / USB-C OTG) e isolamento da porta P7501.*
* 🧪 **[Arquitetura de Testes e Matriz de Conformidade](docs/pt-BR/ARQUITETURA_DE_TESTES_E_CONFORMIDADE.md)** ([English Mirror](docs/en-US/TESTING_ARCHITECTURE_AND_COMPLIANCE.md))*Pirâmide de testes em 5 camadas (Pure FP, GDB RSP, MVVM ADTs, CLI SSOT, E2E), rastreabilidade e status CI/CD.*
* ⚡ **[Blueprint do Esquema Elétrico de Bancada](docs/pt-BR/BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md)** ([English Mirror](docs/en-US/BLUEPRINT_BENCH_WIRING_AND_SCHEMATIC.md))*Chicote de conexão, pinagem do conector P7501, RP2040, disclaimer de resistores de protótipo vs produto final.*
* 💾 **[Arquitetura de Armazenamento MEIHDFS e Partições](docs/pt-BR/ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md)** ([English Mirror](docs/en-US/STORAGE_ARCHITECTURE_MEIHDFS_PARTITIONS.md))*MEIHFS Panasonic, MBR 'SH' em LBA 0, espelho a 28 MB, journal transacional de 10 blocos e partições de firmware.*
* 🔐 **[Dossiê de Criptografia e Contêiner PANA_DVD.FRM](docs/pt-BR/DOSSIE_CRIPTOGRAFIA_E_FIRMWARE_PANA_DVD.md)** ([English Mirror](docs/en-US/DOSSIER_CRYPTOGRAPHY_AND_FIRMWARE_PANA_DVD.md))*Contêiner óptico PANA_DVD.FRM, cifras CPRM/C2, codebook ECB de 64 bits e análise matemática de entropia.*
* 🔬 **[Estratégia de Integração Ghidra e Servidor GDB](docs/pt-BR/ESTRATEGIA_INTEGRACAO_GHIDRA_GDB.md)** ([English Mirror](docs/en-US/GHIDRA_GDB_INTEGRATION_STRATEGY.md))*Processador Matsushita AM34/MN103S, módulo SLED Ghidra e servidor GDB Remote Serial Protocol (:3333).*
* 🛠️ **[Guia Ilustrado de Montagem da Sonda de Bancada](docs/pt-BR/GUIA_MONTAGEM_E_FOTOS_PROBE_BANCADA.md)** ([English Mirror](docs/en-US/BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md))*Montagem da sonda passo a passo, chicote JST ZR 1.5mm e teste mandatória de continuidade do terra (Pino 4 GND).*
* 📋 **[Inventário Técnico e Guia de Operações](docs/pt-BR/INVENTARIO_TECNICO_E_GUIA_DE_OPERACAO.md)** ([English Mirror](docs/en-US/TECHNICAL_INVENTORY_AND_OPERATIONS_MANUAL.md))*Inventário de arquivos, hashes SHA-256 e roteiro passo a passo de desbrick do erro U99.*
* 📊 **[Relatório de Auditoria e Modularização Final](docs/pt-BR/RELATORIO_AUDITORIA_E_MODULARIZACAO_FINAL.md)** ([English Mirror](docs/en-US/FINAL_AUDIT_AND_MODULARIZATION_REPORT.md))*Relatório de modularização, integração cross-stack (Rust/C/Python/TypeScript) e homologação dos 9 Quality Gates.*
* 📡 **[Subsistema de Logging e Telemetria Unificada](docs/pt-BR/SUBSISTEMA_LOGGING_E_TELEMETRIA.md)** ([English Mirror](docs/en-US/LOGGING_AND_TELEMETRY_SUBSYSTEM.md))*Logging estruturado em JSON, sinks thread-safe e telemetria serial em tempo real na interface gráfica.*
* 💡 **[Teoria e Prática: Resolução do U99 e Semicondutores](docs/pt-BR/TEORIA_E_PRATICA_UNBRICK_U99_E_CRIPTOGRAFIA_MATSUSHITA.md)** ([English Mirror](docs/en-US/THEORY_AND_PRACTICE_U99_UNBRICK_MATSUSHITA_CRYPTO.md))*Física de semicondutores da Flash NOR Spansion, motor circular EEMUL e sanitização dos 64 KB de NVRAM para 0xFF.*
* 🛡️ **[Motor RAG Zero-Error e Segurança Física de Bancada](docs/pt-BR/ZERO_ERROR_RAG_ENGINE_E_SEGURANCA_FISICA.md)** ([English Mirror](docs/en-US/ZERO_ERROR_RAG_ENGINE_AND_PHYSICAL_SAFETY.md))
  *Sistema de defesa em 6 camadas determinísticas (QueryGuard, CRAG, GroundedSynthesis, FactCritic, InvariantShield).*

---

## 12. Equipamentos e Consumíveis Recomendados de Laboratório

* **Sonda Microcontrolada:** Raspberry Pi Pico (RP2040) ou Raspberry Pi Pico 2 (RP2350).
* **Conector Fêmea de Acoplamento:** Conector JST ZR fêmea de 6 vias, passo 1.5mm (compatível com `B6B-ZR-3R4`).
* **Componentes Passivos de Proteção:** Quatro resistores de $100\,\Omega$ $0.25	ext{W}$, um resistor de pull-up de $4.7	ext{ k}\Omega$, um resistor de isolamento ADC de $1.0	ext{ k}\Omega$.
* **Instrumentação:** Multímetro Digital (teste de continuidade sonoro), Osciloscópio de 2 canais 100 MHz ou Analisador Lógico USB de 24 MHz.
* **Forense de Armazenamento:** Adaptador de bancada PATA/IDE para USB 3.0 com fonte Molex externa independente de 12V/5V.

---

## 13. Qualidade Total, Pirâmide de Testes e Proteção Anti-Regressão (9 Quality Gates)

O ecossistema Panadiga impõe uma rigorosa pipeline anti-regressão composta por **9 Quality Gates**:

```text
       Camada 5: Testes de Navegador End-to-End (Playwright Chromium 8/8)
       Camada 4: Testes Unitários de UI Desktop (Vitest + React Testing Library 33/33)
       Camada 3: Testes de Host C11 do Firmware Pico (Harness de Teste GCC 7/7)
       Camada 2: CLI Typer & Verificação Pytest RAG em Python (Pytest 21/21, Mypy Strict)
       Camada 1: Testes de Integração & Core Nativo Rust (Cargo Test 41/41)
```

### 13.1. Os 9 Quality Gates Integrados e Ativos

1. **Rust Cargo Check & Test Gate:** Formatação (`cargo fmt`), tipagem e 41 testes unitários/integração.
2. **TypeScript Strict Type-Check Gate:** `tsc --noEmit` valida tipagem estrita com zero escapes `any`.
3. **Frontend Vitest Unit Suite:** 33 testes de ViewModels e transições de estado algébricas.
4. **Copy/Paste Detector (`jscpd`):** Limite estrito de 0.00% de duplicação de código.
5. **UI/UX Completeness Linter (`uxlint`):** Cobertura total de estados loading, empty e error em todas as views (100/100).
6. **Playwright E2E Suite:** 8 testes funcionais completos em navegador Chromium headless.
7. **Ghidra Script Generator Gate:** Exportação automática de mapas de memória e símbolos do SSOT em Rust.
8. **C11 Firmware Host Test Gate:** Compilação e execução com `-Wall -Wextra -Werror` do firmware do Pico.
9. **Python Typer & Pytest Gate:** Tipagem estrita (`mypy --strict`) e 21 testes automatizados.

### 13.2. Execução da Validação Completa Local

```bash
# No Windows PowerShell:
.alidate_all.ps1

# No Windows CMD:
.alidate_all.cmd

# No Linux / macOS / WSL:
./run.sh --validate
```

---

## 14. Governança de Inteligência Artificial, Servidor MCP e Logging Estruturado

### 14.1. Matriz de Subagentes Especialistas (`.agents/agents/`)

O repositório define 5 subagentes de engenharia especializados para suporte autônomo:

* **`panasonic-hardware-engineer`:** Esquemáticos, topografia de PCB, tensões e integridade de sinal DWire32A.
* **`panasonic-crypto-analyst`:** Criptoanálise de blocos, codebooks ECB e proteção CPRM/DivX.
* **`panasonic-storage-architect`:** Geometria MEIHDFS, reparo de MBR e análise forense de sistemas de arquivos.
* **`panasonic-desktop-architect`:** Interface desktop Tauri 2.0, React 19 e bindings IPC tipados.
* **`panasonic-rag-evaluator`:** Motor RAG Zero-Error, ancoragem factual e guardrails físicos.

### 14.2. Servidor Nativo Model Context Protocol (`tools/panadiga_mcp_server.py`)

Um servidor MCP padronizado em JSON-RPC 2.0 expõe ferramentas de laboratório para agentes de IA:

* `panadiga_rag_query`: Consulta a base local do RAG com fundamentação factual estrita.
* `panadiga_manuals_search`: Busca nos 30 manuais técnicos e datasheets com citações de página.
* `panadiga_inspect_hdd`: Inspeciona fronteiras de partição e validação do superbloco MEIHDFS.
* `panadiga_analyze_frm`: Analisa contêineres de atualização óptica e extrai a TOC de módulos.

### 14.3. Subsistema de Logging e Telemetria Unificada (`src/logger.rs`)

Todos os componentes operam através de sinks estruturados:

* **Zero console.log/println:** É terminantemente proibido o uso de logs não-estruturados soltos em código de produção.
* **Níveis de Log:** `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` com carimbo de tempo em microssegundos e tags de subsistema.
* **Streaming na Interface Desktop:** Eventos de log são transmitidos em tempo real para o painel de terminal da UI.

---

## 15. Conformidade e Integridade de Engenharia

Distribuído sob a **Licença MIT**. Consulte [`LICENSE`](LICENSE) para termos completos.

Toda a engenharia reversa, análise criptográfica e documentação de hardware neste projeto foram conduzidas estritamente para fins de interoperabilidade, preservação histórica e reparo de equipamentos eletrônicos descontinuados, em estrita conformidade com exceções legais de engenharia reversa para interoperabilidade e reparabilidade.
