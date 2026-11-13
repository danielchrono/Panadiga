# ARQUITETURA DE STORAGE, FILESYSTEM MEIHDFS E PARTICIONAMENTO
## Engenharia Reversa do Disco Rígido e Firmware do Gravador Panasonic DMR-EH55
### Plataforma Matsushita UniPhier / AM34 — Chassi DIGA (PCB: VEP79138)

---

## 1. Identificação Geral do Sistema e Escopo de Engenharia de Armazenamento

O gravador digital de DVD/HDD **Panasonic DMR-EH55** (pertencente à conceituada família DIGA da Matsushita Electric Industrial Co., Ltd.) utiliza uma arquitetura de armazenamento híbrida não-convencional. Diferente de PCs ou gravadores baseados em sistemas operacionais genéricos (como Linux embarcado), o DMR-EH55 baseia-se no processador proprietário **Matsushita AM34** (núcleo RISC de 32 bits da família **MN103S**) e delega o armazenamento persistente a dois componentes complementares:

1. **Memória Flash NOR On-Board (Spansion S29JL064H — 8 MBytes TSOP-48):**
   Contém o bootloader primário (*IPL - Initial Program Loader*), rotinas mínimas de inicialização de hardware, vetores de interrupção e os blocos de parâmetros de calibração emulados em Flash (`EEMUL` / NVRAM).
2. **Disco Rígido PATA/IDE Interno (80 GB a 500 GB):**
   Não armazena apenas os vídeos do usuário. O disco rígido é um elemento ativo do subsistema de boot: **ele armazena as partições com o firmware operacional principal (AM34 executive), drivers dos CODECs de áudio/vídeo, bibliotecas de interface gráfica (OSD), microcódigos de periféricos e a infraestrutura do sistema de arquivos MEIHDFS**.

Se um disco rígido virgem (sem inicialização Panasonic) for instalado na máquina com a unidade de DVD inoperante, o aparelho trava na inicialização em autoteste com as mensagens de erro de comunicação/hardware **`HDDERR`** ou **`DRV NG`**, recusando-se a carregar o sistema operacional ou a formatar o disco pelos menus comuns.

```
                  ARQUITETURA DE INICIALIZAÇÃO HÍBRIDA DO PANASONIC DMR-EH55
 ┌────────────────────────────────────────────────────────────────────────────────────────┐
 │                              PLACA PRINCIPAL (VEP79138)                                │
 │                                                                                        │
 │   ┌──────────────────────────┐                  ┌──────────────────────────────────┐   │
 │   │  Matsushita AM34/MN103S  │   Boot Inicial   │      Spansion S29JL064H          │   │
 │   │  (UniPhier / AM34 Core)  │◄─────────────────│  (8 MB Flash NOR TSOP-48)        │   │
 │   │   108 MHz / 162 MHz      │   (Fase 1 IPL)   │  - Initial Program Loader (IPL)  │   │
 │   └────────────┬─────────────┘                  │  - Bootloader Mínimo             │   │
 │                │                                │  - NVRAM / EEMUL (Calibrações)   │   │
 │                │ Barramento IDE / ATA           └──────────────────────────────────┘   │
 │                ▼                                                                       │
 │   ┌────────────────────────────────────────────────────────────────────────────────┐   │
 │   │                        DISCO RÍGIDO (PATA / IDE / SSD)                         │   │
 │   │                                                                                │   │
 │   │   [ LBA 0 ]                Panasonic MBR Proprietário ("SH", Disk GUID)        │   │
 │   │   [ LBA 24..31 ]           Tabela de Metadados e Alocação de Baixo Nível       │   │
 │   │   [ LBA 16.384..19.199 ]   Journal Transacional (10 Blocos de 128 KB)          │   │
 │   │   [ LBA 57.344 (28 MB) ]   Tabela Espelho / Backup Idêntico ao LBA 0           │   │
 │   │   [ LBA 65.536 (32 MB) ]   Partição Firmware 1 (0x5E3250 = ~5.89 MB) (Fase 2)  │   │
 │   │   [ LBA 131.072 (64 MB) ]  Partição Firmware 2 (0xB0000 = ~704 KB) (OSD/Micro) │   │
 │   │   [ LBA 204.800+ ]         Volume MEIHDFS (VR_MOVIE.VRO / VR_MANGR.IFO)        │   │
 │   └────────────────────────────────────────────────────────────────────────────────┘   │
 └────────────────────────────────────────────────────────────────────────────────────────┘
```

### Arquivo da Imagem Analisado
* **Nome do Arquivo:** `firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin`
* **Tamanho Exato:** `104.857.600` bytes (exatamente 100,00 MB)
* **Total de Setores LBA:** `204.800` setores de 512 bytes
* **Hash MD5 Global:** `ef4453fc8651903b0a4b0c1873d950d4`
* **Hash SHA-256 Global:** `9d292d5b46f190775f359e30af11d074021fb321f040bf701d0ecf0f2056db00`

---

## 2. Mapa Geral de Memória e Estrutura de Disco (LBAs e Offsets)

A análise exaustiva setor por setor da imagem de 100 MB revelou a exata topografia de armazenamento adotada pelos engenheiros da Matsushita. O layout divide o disco em zonas de proteção, metadados transacionais, redundância em espelho e partições de código executável:

| Faixa de Setores (LBA) | Faixa de Offsets (Hexadecimal) | Tamanho (Bytes) | Tamanho (MB) | Função Primária / Descrição Técnica |
| :--- | :--- | :--- | :--- | :--- |
| **LBA 0** | `0x00000000` – `0x000001FF` | 512 B | 0,0005 MB | **Master Boot Record Panasonic:** Assinatura `"SH"`, versão 1.0 e GUID do disco. |
| **LBA 1 .. 23** | `0x00000200` – `0x00002FFF` | 11.776 B | 0,0112 MB | **Zona de Guarda 1:** Setores nulos (`0x00`) de proteção de pré-metadados. |
| **LBA 24 .. 31** | `0x00003000` – `0x00003FFF` | 4.096 B | 0,0039 MB | **Tabela de Descritores:** 8 setores com registros de 16 bytes de mapeamento físico. |
| **LBA 32 .. 16.383** | `0x00004000` – `0x007FFFFF` | 8.372.224 B | 7,9844 MB | **Zona de Guarda Intermediária:** Espaço preenchido com zeros até 8 MB. |
| **LBA 16.384** | `0x00800000` – `0x008001FF` | 512 B | 0,0005 MB | **Catálogo do Journal:** Header com contagem de blocos (`0x0A` = 10 blocos). |
| **LBA 16.385 .. 16.639** | `0x00800200` – `0x0081FFFF` | 130.560 B | 0,1245 MB | **Guarda de Pré-Alocação:** Alinhamento de limite de bloco. |
| **LBA 16.640 .. 19.199** | `0x00820000` – `0x0095FFFF` | 1.310.720 B | 1,2500 MB | **Área de Journaling Transacional:** 10 blocos de 128 KB (256 setores cada). |
| **LBA 19.200 .. 49.151** | `0x00960000` – `0x017FFFFF` | 15.335.424 B | 14,625 MB | **Zona de Guarda Pós-Journal:** Espaço preenchido com zeros até 24 MB. |
| **LBA 49.152** | `0x01800000` – `0x018001FF` | 512 B | 0,0005 MB | **Marcador de Transição de 24 MB:** Estado de formatação e limites de volume. |
| **LBA 49.153 .. 57.343** | `0x01800200` – `0x01BFFFFF` | 4.193.792 B | 3,9995 MB | **Zona de Guarda de Pré-Espelho:** Espaço preenchido com zeros até 28 MB. |
| **LBA 57.344** | `0x01C00000` – `0x01C001FF` | 512 B | 0,0005 MB | **Tabela Espelho (Mirror MBR):** Cópia idêntica bit-a-bit do LBA 0 (a 28,00 MB). |
| **LBA 57.368 .. 57.375** | `0x01C03000` – `0x01C03FFF` | 4.096 B | 0,0039 MB | **Espelho dos Descritores:** Cópia idêntica bit-a-bit dos setores 24 a 31. |
| **LBA 57.376 .. 65.535** | `0x01C04000` – `0x01FFFFFF` | 4.177.920 B | 3,9844 MB | **Zona de Guarda de Pré-Firmware:** Espaço preenchido com zeros até 32 MB. |
| **LBA 65.536 .. 77.593** | `0x02000000` – `0x025E33FF` | 6.173.696 B | 5,8877 MB | **Partição de Firmware 1:** Código principal AM34, codecs e sistema DIGA (0x5E3250). |
| **LBA 77.594 .. 131.071** | `0x025E3400` – `0x03FFFFFF` | 27.382.784 B | 26,114 MB | **Espaço Reservado Inter-Partições:** Espaço preenchido com zeros até 64 MB. |
| **LBA 131.072 .. 132.479** | `0x04000000` – `0x040AFFFF` | 720.896 B | 0,6875 MB | **Partição de Firmware 2:** Microcódigo, fontes de tela OSD e serviços (0xB0000). |
| **LBA 132.480 .. 204.799** | `0x040B0000` – `0x063FFFFF` | 37.052.416 B | 35,336 MB | **Área de Expansão dos 100 MB:** Transição final até o espaço de dados do usuário. |
| **LBA 204.800 em diante** | `0x06400000` até o fim | Restante HD | Restante | **Área do Filesystem MEIHDFS:** Extents contínuos para streaming de áudio/vídeo. |

---

## 3. Engenharia Reversa Detalhada do MBR e Setor 0 (LBA 0 & LBA 57344)

### Análise Hexadecimal do Setor 0 (Offset 0x00000000)
A leitura dos 512 bytes do LBA 0 apresenta a seguinte disposição em memória:

```text
Offset    00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F  ASCII
00000000: 00 00 00 00 01 00 53 48 62 8C 6E F5 1B 2C 36 66  ......SHb.n..,6f
00000010: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
...
000001F0: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
```

### Decodificação Estrutural de Campos

```
                      ESTRUTURA DO HEADER PROPRIETÁRIO PANASONIC (16 BYTES)
   0x00        0x03 0x04  0x05 0x06  0x07 0x08                                0x0F
  ┌────────────────┬──────────┬──────────┬────────────────────────────────────────┐
  │  00 00 00 00   │  01 00   │  53 48   │  62 8C 6E F5 1B 2C 36 66               │
  └────────────────┴──────────┴──────────┴────────────────────────────────────────┘
     Status/Flags    Versão     Tag Magic               Volume Disk GUID
      (Reservado)    Rev 1.0     ASCII "SH"         (Sessão de Formatação Única)
```

1. **Bytes `0x00 .. 0x03` (`00 00 00 00`):**
   Campo de flags de status e controle de inicialização do volume. Um valor zerado indica volume normal montável e sincronizado.
2. **Bytes `0x04 .. 0x05` (`01 00`):**
   Versão da tabela de particionamento da Panasonic em formato Little-Endian (`0x0001` = Revisão 1.0).
3. **Bytes `0x06 .. 0x07` (`53 48`):**
   Identificador mágico em caracteres ASCII: `"SH"`. Significa **Storage Header** ou **System Harddisk**. É o primeiro marcador testado pelo bootloader IPL da Spansion após emitir o comando ATA `IDENTIFY DEVICE`.
4. **Bytes `0x08 .. 0x0F` (`62 8C 6E F5 1B 2C 36 66`):**
   Identificador global único de 64 bits (*Volume Disk GUID / Formatting Session Hash*). Este valor é gerado de forma pseudo-aleatória pela CPU AM34 no momento da formatação interna pelo menu de serviço. O firmware valida se este GUID é idêntico no LBA 0 e na tabela espelho no LBA 57.344.
5. **Bytes `0x10 .. 0x1FF` (502 bytes preenchidos com `0x00`):**
   Todos os demais bytes do setor 0 são rigorosamente zerados.

### Por que o Windows e Linux não reconhecem o disco?
* **Ausência do Magic 0x55AA:** O MBR padrão de computadores IBM-PC exige os bytes `55 AA` nos offsets `0x1FE .. 0x1FF`. No Panasonic, esses bytes são `00 00`.
* **Ausência de Entradas de Partição no Offset 0x1BE:** A tabela de partição padrão MBR situa-se entre os bytes 446 e 510 (`0x01BE .. 0x01FD`), consistindo em 4 entradas de 16 bytes descrevendo CHS/LBA inicial e contagem de setores. No Panasonic, toda essa região contém zeros absolutos.
* **Diagnóstico dos Sistemas Operacionais:**
  - **Microsoft Windows:** O utilitário *Gerenciamento de Disco* (`diskmgmt.msc`) detecta a falta de assinatura `55 AA` e marca o dispositivo como **"Disco Desconhecido, Não Inicializado"**. O Windows exibe um pop-up sugerindo: *"Você deve inicializar um disco antes que o Gerenciador de Discos Lógicos possa acessá-lo (MBR ou GPT)"*. **Atenção:** Permitir a inicialização pelo Windows sobrescreve imediatamente o LBA 0, destruindo o cabeçalho `"SH"`, o que inutiliza o disco para o DMR-EH55.
  - **GNU/Linux:** O kernel emite durante o attach do dispositivo SCSI/ATA a mensagem: `sdX: unable to read partition table` ou `unknown partition table`.

### Tabela Espelho / Backup em LBA 57.344 (Offset 0x01C00000 = 28 MB)
A Panasonic implementou um mecanismo de tolerância a falhas por redundância de hardware. Exatamente no offset `0x01C00000` (29.360.128 bytes, equivalente a 28 MB ou LBA 57.344), existe um setor de backup idêntico bit-a-bit ao LBA 0.

* **Hash SHA-256 de LBA 0:** `61162c24c9240408ac807cd4307c9c5833d440715a982126c536aa10aa7530f2`
* **Hash SHA-256 de LBA 57344:** `61162c24c9240408ac807cd4307c9c5833d440715a982126c536aa10aa7530f2`
* **Status de Validação:** **Identidade estrita de 100% (512/512 bytes coincidentes)**.

Se o LBA 0 for corrompido por uma inicialização inadvertida no Windows, o firmware do DMR-EH55 é capaz de restaurar o cabeçalho original a partir de LBA 57.344 ao entrar em modo de serviço de recuperação.

---

## 4. Metadados de Alocação e Journaling Transacional (MEIHDFS Allocation Table)

### Setores 24 a 31 (0x00003000 a 0x00003FFF — 4.096 Bytes): MEIHDFS Allocation Table Encodings
The MEIHDFS allocation table and block descriptors encoded on the hard drive sectors LBA 24..31 (0x00003000..0x00003FFF) are bit-inverted using bitwise NOT / XOR 0xFF (`byte ^ 0xFF`).
Os 8 setores localizados entre LBA 24 e 31 contêm tabelas compactadas com registros de 16 bytes que mapeiam descritores de blocos de serviço do disco rígido (hard drive).

* **Arquitetura Bit-Invertida (One's Complement / XOR 0xFF):**  
  Uma característica marcante e fundamental do sistema de baixo nível da Matsushita é o armazenamento dos descritores em complemento de 1 bitwise (`byte ^ 0xFF`). A razão de engenharia para essa inversão deliberada reside no alinhamento semântico com células virgens de Flash/EPROM (onde `0xFF` representa o estado não programado ou bloco livre/não alocado).  
  Quando a tabela de 4.096 bytes é submetida ao XOR `0xFF`, emergem exatamente **256 registros estruturados de 16 bytes**, categorizados por 6 tags mestras que cobrem 100% da tabela:
  - **Tag `0x5D` (65 registros):** Extents principais, contagem de setores alocados e ponteiros LBA de blocos de áudio e vídeo.
  - **Tag `0x55` (64 registros):** Registros de sincronismo e continuação de extents fragmentados.
  - **Tag `0x57` (47 registros):** Ponteiros de LBA e vetores de endereçamento físico.
  - **Tag `0xE0` (42 registros):** Entradas de diretório e metadados de arquivos do sistema MEIHDFS.
  - **Tag `0xE1` (26 registros):** Nós virtuais (*V-Nodes*) e identificadores de inode do filesystem.
  - **Tag `0x5F` (12 registros):** Descritores de geometria de blocos e pool de mídia física.
  *(Total exato: $65 + 64 + 47 + 42 + 26 + 12 = 256\text{ registros}$ de 16 bytes).*

* **Ferramenta de Decodificação Automatizada:**  
  A suíte `panadiga` implementa a decodificação completa e instantânea dessa tabela através do comando:
  ```bash
  cargo run --release -- hdd decode-descriptors "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"
  ```

* **Espelhamento em 28 MB:** Esta tabela de 4.096 bytes é rigorosamente duplicada nos setores **LBA 57.368 a 57.375 (Offset 0x01C03000 a 0x01C03FFF)**.
  - **SHA-256 Setores 24..31:** `4b604ad2a689bd39cc8667b2479e70e304d574a78fcf72bfdeb91ca138fdce24`
  - **SHA-256 Setores 57368..57375:** `4b604ad2a689bd39cc8667b2479e70e304d574a78fcf72bfdeb91ca138fdce24`
  - **Status:** **100% idêntico bit-a-bit** com integridade confirmada.

### Catálogo e Journal de 10 Blocos (0x00800000 a 0x00960000)
A região a partir de 8 MB (`0x00800000`) atua como a área de log de transações atômicas (*Write-Ahead Journaling*) do gravador, garantindo que o sistema de arquivos sobreviva a desligamentos repentinos:

1. **LBA 16.384 (0x00800000) — Catálogo de Blocos:**
   Os primeiros 8 bytes contêm:
   - Bytes `0x00 .. 0x03`: `0A 00 00 00` (Inteiro Little-Endian de 32 bits = `10` blocos no pool de journal).
   - Bytes `0x04 .. 0x07`: `09 00 00 00` (Inteiro Little-Endian de 32 bits = índice máximo `9`, baseado em zero).
2. **Os 10 Blocos de Alocação de 128 KB (LBA 16.640 a 19.199):**
   A partir do offset `0x00820000`, sucedem-se 10 blocos consecutivos de exatamente 256 setores (128 KB = 131.072 bytes) cada. Cada bloco possui a assinatura inicial `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF`:

| Bloco | Offset Inicial | LBA Inicial | Tamanho | Header (16 Bytes Hex) | Trailer de Sequência (8 Bytes) |
| :---: | :---: | :---: | :---: | :--- | :--- |
| **0** | `0x00820000` | 16.640 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `D1 81 C8 0A 00 00 00 01` (Seq 1) |
| **1** | `0x00840000` | 16.896 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `5C FC 6E 0A 00 00 00 02` (Seq 2) |
| **2** | `0x00860000` | 17.152 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `47 06 23 0A 00 00 00 03` (Seq 3) |
| **3** | `0x00880000` | 17.408 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `71 F3 B9 0A 00 00 00 04` (Seq 4) |
| **4** | `0x008A0000` | 17.664 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `55 D5 9F 0A 00 00 00 05` (Seq 5) |
| **5** | `0x008C0000` | 17.920 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `55 54 C0 0A 00 00 00 06` (Seq 6) |
| **6** | `0x008E0000` | 18.176 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `55 57 7E 0A 00 00 00 07` (Seq 7) |
| **7** | `0x00900000` | 18.432 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `55 51 03 0A 00 00 00 08` (Seq 8) |
| **8** | `0x00920000` | 18.688 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `74 60 32 08 00 00 00 09` (Seq 9) |
| **9** | `0x00940000` | 18.944 | 128 KB | `5D FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | `17 3F 9B 09 00 00 00 0A` (Seq 10)|

Ao final do Bloco 9 (em `0x00960000`), a região é finalizada com preenchimento em zeros limpos.

---

## 5. Engenharia das Partições de Firmware Proprietárias

### A Assinatura de Partição Universal
Ao contrário dos sistemas FAT/ext/NTFS, as partições executáveis da Matsushita no DMR-EH55 não são referenciadas por uma tabela de alocação de partição com descritores CHS/LBA no MBR. Em vez disso, **cada partição inicia-se fisicamente com uma assinatura mágica de 8 bytes de sincronização de hardware**:

$$\mathbf{\text{Panasonic Partition Magic: }}\quad \mathtt{51\ 87\ 13\ 8D\ 5B\ 0A\ FB\ 22}$$

O bootloader primário na Flash Spansion salta diretamente para os endereços fixos **`0x02000000`** e **`0x04000000`** no disco rígido, lê os primeiros 8 bytes e valida se coincidem com esta assinatura antes de carregar o binário na memória SDRAM do gravador.

```
                     ESTRUTURA DAS PARTIÇÕES DE FIRMWARE NO HDD
    0x02000000 (32 MB, LBA 65.536)
   ┌────────────────────────────────────────────────────────────────────────┐
   │ 51 87 13 8D 5B 0A FB 22 ... E5 D1 F3 4A 51 8B 26 90 (+0x38 Sync)       │
   │                                                                        │
   │ PARTIÇÃO 1: FIRMWARE PRINCIPAL & CODECS DIGA                           │
   │ Payload Útil: 0x5E3250 (6.173.264 bytes = ~5,89 MB)                    │
   │ Padding Setor: 432 bytes 0x00 até 0x025E3400 (12.058 setores LBA)     │
   │ Entropia: 7,9990 bits/byte (Binário Criptografado / Comprimido)        │
   └────────────────────────────────────────────────────────────────────────┘

    0x04000000 (64 MB, LBA 131.072)
   ┌────────────────────────────────────────────────────────────────────────┐
   │ 51 87 13 8D 5B 0A FB 22 ... E5 D1 F3 4A 51 8B 26 90 (+0x30/0x38 Sync) │
   │                                                                        │
   │ PARTIÇÃO 2: SUBSISTEMA, FONTES OSD & MICROÓDIGO                        │
   │ Payload Útil: 0xAFFD0 (720.848 bytes) + 48 bytes 0xFF (Total: 0xB0000) │
   │ Tamanho: 720.896 bytes = 704 KB (1.408 setores LBA)                    │
   │ Entropia: 7,9929 bits/byte (Binário Criptografado / Comprimido)        │
   └────────────────────────────────────────────────────────────────────────┘
```

### Partição 1: Firmware do Sistema Operacional & Codecs (Offset 0x02000000)
* **Localização Física:** Offset `0x02000000` (exatamente a 32,00 MB da mídia), correspondendo ao setor **LBA 65.536**.
* **Tamanho do Payload Útil:** `6.173.264` bytes (`0x5E3250` bytes = aproximadamente **5,887 MBytes**).
* **Tamanho Alinhado ao Setor:** `6.173.696` bytes (`0x5E3400` bytes = exatamente **12.058 setores LBA**). Os últimos 432 bytes do setor 77.593 são preenchidos com zeros (`0x00`).
* **Marcadores Internos de Sincronismo:**
  - Offset `0x00 .. 0x07`: `51 87 13 8D 5B 0A FB 22` (Assinatura de Partição).
  - Offset `0x38 .. 0x3F`: `E5 D1 F3 4A 51 8B 26 90` (Marcador de verificação de chave / integridade).
* **Entropia de Shannon:** `7,9990` bits/byte. A densidade de distribuição de bytes beira o limite teórico máximo ($8,0000$), evidenciando que o código é compactado por algoritmo de alta densidade proprietário da Matsushita ou criptografado contra engenharia reversa.
* **Hashes Criptográficos Oficiais:**
  - **Payload Útil (0x5E3250):**
    * MD5: `393e52d939eee76bbf06a98f2415253f`
    * SHA-256: `6ec63de059c00b287a0e8a1a4102a8007bbccf3210a552bfb45118a03b85c898`
  - **Setor Alinhado (0x5E3400):**
    * MD5: `85af47c916ac085a0b77619e7d4fe05e`
    * SHA-256: `d77fa2af9860edc694bf9fa7250514c1bf7a2e1821d69ffdac33454940c6950b`

### Partição 2: Subsistema, Fontes OSD e Microcódigo (Offset 0x04000000)
* **Localização Física:** Offset `0x04000000` (exatamente a 64,00 MB da mídia), correspondendo ao setor **LBA 131.072**.
* **Tamanho Total da Partição:** `720.896` bytes (`0xB0000` bytes = exatamente **704 KB** = **1.408 setores LBA**).
* **Tamanho dos Dados Úteis:** `720.848` bytes (`0xAFFD0` bytes), seguidos por exatamente 48 bytes (`0x30`) de preenchimento em `0xFF`.
* **Marcadores Internos de Sincronismo:**
  - Offset `0x00 .. 0x07`: `51 87 13 8D 5B 0A FB 22` (Assinatura de Partição).
  - Offset `0x30 .. 0x37`: `E5 D1 F3 4A 51 8B 26 90`.
  - Offset `0x38 .. 0x3F`: `E5 D1 F3 4A 51 8B 26 90` (Duplicado consecutivamente).
  - Trailer Final (`0xAFFC0 .. 0xAFFCF`): Padrão de fechamento `03 50 65 C8 7B B8 E0 90` repetido 2 vezes antes dos 48 bytes de `0xFF`.
* **Entropia de Shannon:** `7,9929` bits/byte.
* **Hashes Criptográficos Oficiais (0xB0000):**
  - MD5: `ee465f12930bef1599bb4fc256346ef0`
  - SHA-256: `487d5951d4aa7bc45e5f634c4ca079f369db06d5cb62e41a173575cecd68dbae`

---

## 6. Investigação Tecnológica do Filesystem MEIHDFS & Variantes UDF

### Origem e Razão Histórica do MEIHDFS
O acrônimo **MEIHDFS** designa o **Matsushita Electric Industrial Hard Disk File System**. Ele foi concebido pela Panasonic entre 2002 e 2004 para superar as limitações críticas dos sistemas de arquivos padronizados da época quando aplicados à gravação contínua de televisão em tempo real:

1. **A Falha do FAT32:** O limite intrínseco de 4 GB por arquivo impedia a gravação contínua de filmes ou programas com mais de duas horas na taxa de dados XP (10 Mbps). Além disso, a alocação baseada em tabela FAT fragmentava o disco rapidamente, causando paradas súbitas na cabeça de gravação (*seek latency*) e perda de quadros (*dropped frames*).
2. **A Inviabilidade do NTFS / ext3:** Sistemas com journaling de uso geral consomem muitos ciclos de CPU e memória RAM para gerenciar árvores B-Tree e permissões POSIX/ACL, além de não possuírem garantias de streaming em tempo real (QoS para gravação de áudio/vídeo).
3. **A Transição do UDF:** Nos primeiros modelos de DVD-RAM (como DMR-E85H), a Panasonic gravava diretamente em UDF 1.5/2.01 no disco óptico e tentou emular o mesmo formato no HD. Entretanto, o overhead de descritores de alocação curtos e a incapacidade de gerenciar discos rígidos com dezenas de gigabytes de forma performática levaram ao projeto do **MEIHDFS-V2.0** (usado no DMR-EH55 e séries subsequentes).

### Mecanismos Operacionais do MEIHDFS-V2.0

```
                     ORGANIZAÇÃO DE INODES E EXTENTS DO MEIHDFS
  ┌─────────────────────────────────────────────────────────────────────────────┐
  │                           MEIHDFS SUPERBLOCK                                │
  │   - Número Mágico no Offset 0x0C                                            │
  │   - Ponteiro Raiz da Tabela de Inodes (`itbl`)                              │
  │   - Tamanho do Cluster Base de Extent (128 KB ou 2 MB)                     │
  └──────────────────────────────────────┬──────────────────────────────────────┘
                                         │
                                         ▼
  ┌─────────────────────────────────────────────────────────────────────────────┐
  │                     TABELA DE INODES & DIRETÓRIOS                           │
  │                                                                             │
  │   INODE 1: VR_MANGR.IFO                                                     │
  │   ├── Metadados: Tamanho, Data, Flags de Vídeo                              │
  │   └── Extent Map: LBA 204.800 (Tamanho: 64 KB)                             │
  │                                                                             │
  │   INODE 2: VR_MOVIE.VRO (Streaming MPEG-2 de Vídeo)                         │
  │   ├── Extent 0: LBA 205.056 a 409.600    (Tamanho: 100 MB contínuos)        │
  │   ├── Extent 1: LBA 800.000 a 1.200.000  (Tamanho: 200 MB contínuos)        │
  │   └── Extent N: ... (Alocação em macro-blocos sem perda de quadros)         │
  └─────────────────────────────────────────────────────────────────────────────┘
```

1. **Alocação por Extents Contínuos (*Continuous Extent Allocation*):**
   Ao iniciar a gravação de um canal de TV, o driver MEIHDFS pré-aloca blocos massivos e contíguos (macro-blocos de 128 KB, 2 MB ou 32 MB). A cabeça do disco grava o fluxo MPEG-2 de forma linear, garantindo taxa de transferência constante sem qualquer vibração ou latência mecânica.
2. **Edição Não-Linear Instantânea (NLE Sem Recópia):**
   Quando o usuário utiliza a função de edição do gravador (por exemplo, "Corte de Comerciais" / *A-B Erase* ou "Divisão de Título"):
   - O gravador **não reescreve nem move** o arquivo de vídeo de vários gigabytes.
   - O MEIHDFS apenas divide a entrada de extents no inode do arquivo, removendo o intervalo de setores LBA cortado da lista de reprodução e devolvendo esses setores ao mapa de blocos livres.
3. **Indexação de Vídeo e Áudio:**
   - **`VR_MOVIE.VRO`:** Arquivo contêiner que agrupa os fluxos de vídeo MPEG-2 (Video Elementary Stream) e áudio Dolby Digital AC-3 ou LPCM multiplexados em pacotes PES (Packetized Elementary Stream).
   - **`VR_MANGR.IFO` / `VR_MANGR.BUP`:** Tabelas de gerenciamento DVD-VR. Contêm o **Time Map (TMAP)**, que relaciona intervalos de tempo de exibição a blocos físicos chamados **VOBU (Video Object Units)**. Cada VOBU contém um pacote de navegação (*NV_PCK*) que informa o LBA do próximo quadro-chave (I-Frame), viabilizando avanço rápido e retrocesso suave.
4. **Proteção de Conteúdo e Anti-Cópia (CPRM):**
   O fato de o MEIHDFS não possuir MBR compatível com PC constitui uma barreira deliberada contra cópia pirata. O padrão CPRM (*Content Protection for Recordable Media*) exigia que transmissões protegidas com sinal *Copy-Once* fossem inacessíveis por sistemas operacionais de computadores convencionais.

### Estado da Arte da Comunidade de Engenharia Reversa
A engenharia reversa do MEIHDFS foi liderada por pesquisadores da comunidade de preservação de mídia:

* **Projeto `panasonic-rec` (`leecher1337`):**
  Disponível no GitHub, é a referência técnica aberta mais completa. O projeto desenvolveu o utilitário **`extract_meihdfs`**, capaz de abrir a imagem bruta do disco, ler o superblock do MEIHDFS-V2.0, decodificar a tabela de inodes (`itbl`) e extrair os arquivos de vídeo intactos.
* **Ferramenta `dvd-vr`:**
  Como o arquivo `VR_MOVIE.VRO` extraído muitas vezes possui descontinuidades provocadas por edições não-lineares, a ferramenta `dvd-vr` reconstrói os fluxos de áudio e vídeo e gera arquivos `.VOB` e `.MPG` padrão compatíveis com reprodutores modernos (como o VLC Media Player).
* **Soluções Periciais / Forenses:**
  Ferramentas forenses comerciais (como o **IsoBuster Pro**) incorporaram parsers proprietários para discos Panasonic MEIHDFS, permitindo a peritos judiciais e órgãos de segurança pública a extração de vídeos de vigilância registrados em aparelhos DIGA apreendidos.

---

## 7. Manual de Operação do Script `parse_panasonic_hdd_partitions.py`

O script `scripts/parse_panasonic_hdd_partitions.py` foi desenvolvido especificamente para automatizar a auditoria, verificação de integridade e extração dos componentes do disco rígido do DMR-EH55.

### Sintaxe de Comando
```bash
python scripts/parse_panasonic_hdd_partitions.py [OPÇÕES] [CAMINHO_DA_IMAGEM]
```

### Argumentos Suportados
* `image`: Caminho para a imagem de disco (padrão: `firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin`).
* `--extract`: Habilita a extração automática das partições e setores-chave para arquivos binários separados.
* `--outdir DIR`: Define o diretório de destino dos arquivos extraídos (padrão: `firmware_and_images/extracted_partitions`).
* `--json ARQUIVO`: Exporta um relatório técnico estruturado em JSON contendo todos os dados, offsets e hashes.
* `--verbose` / `-v`: Exibe detalhes avançados da análise estrutural no console.

### Exemplo de Execução Completa
```bash
python scripts/parse_panasonic_hdd_partitions.py --extract --outdir firmware_and_images/extracted_partitions --json firmware_and_images/extracted_partitions/hdd_analysis_report.json
```

### Manifesto de Arquivos Extraídos (`SHA256SUMS.txt`)
A execução do extrator gera os seguintes artefatos técnicos no diretório de destino:

```text
61162c24c9240408ac807cd4307c9c5833d440715a982126c536aa10aa7530f2  hdd_mbr_master_lba0.bin
61162c24c9240408ac807cd4307c9c5833d440715a982126c536aa10aa7530f2  hdd_mbr_mirror_lba57344.bin
4b604ad2a689bd39cc8667b2479e70e304d574a78fcf72bfdeb91ca138fdce24  hdd_allocation_descriptors_0x00003000.bin
4c28d51cde8bcbcf2d2a86d30f4351f50bf509228fe804a09601205fa06f73a2  hdd_journal_blocks_0x00820000.bin
6ec63de059c00b287a0e8a1a4102a8007bbccf3210a552bfb45118a03b85c898  hdd_partition_firmware_0x02000000.bin
d77fa2af9860edc694bf9fa7250514c1bf7a2e1821d69ffdac33454940c6950b  hdd_partition_firmware_0x02000000_sector_aligned.bin
487d5951d4aa7bc45e5f634c4ca079f369db06d5cb62e41a173575cecd68dbae  hdd_partition_firmware_0x04000000.bin
```

---

## 8. Roteiro Prático de Substituição de Disco Rígido / SSD

Quando o disco original de um Panasonic DMR-EH55 falha mecanicamente ou quando o usuário deseja substituí-lo por um SSD (utilizando adaptadores PATA-to-SATA com chipset Marvell ou JMicron), o procedimento de restauração deve seguir rigorosamente as etapas abaixo:

```
               FLUXOGRAMA DE RESTAURAÇÃO DE DISCO RÍGIDO / SSD NO DMR-EH55
  ┌────────────────────────┐
  │  Novo HD / SSD Virgem  │
  └───────────┬────────────┘
              │
              ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │ ETAPA 1: Gravação dos Primeiros 100 MB via DD no Computador            │
  │ dd if=panasonic_hdd_100.bin of=/dev/sdX bs=1M count=100 status=progress│
  │ (Cria LBA 0, Mirror a 28 MB e as Partições de Firmware a 32MB e 64MB)   │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │ ETAPA 2: Instalação no Gravador Panasonic DMR-EH55                     │
  │ Conexão ao cabo flat IDE de 40 vias e alimentação Molex                │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │ ETAPA 3: Boot Inicial Sem Erro "DRV NG"                                │
  │ O processador AM34 valida o "SH", carrega o Firmware 1 e Firmware 2    │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │ ETAPA 4: Formatação pelo Menu de Serviço Panasonic                     │
  │ Functions ➔ To Others ➔ Setup ➔ Others ➔ HDD Management ➔ Format       │
  │ (O DMR-EH55 aloca a capacidade total do novo disco para o MEIHDFS)     │
  └────────────────────────────────────────────────────────────────────────┘
```

1. **Gravação da Imagem de Bootstrap (100 MB):**
   Conecte o disco de destino (HD ou SSD) ao computador através de um adaptador IDE/SATA para USB.
   No Linux/macOS:
   ```bash
   sudo dd if="panasonic DMR EH-55 Firmware hdd_100.bin" of=/dev/sdX bs=1M count=100 status=progress
   sudo sync
   ```
   No Windows, utilize utilitários de escrita bruta em nível de setor (como `Win32DiskImager` ou `BalenaEtcher`).
2. **Instalação no Chassi DIGA:**
   Instale a mídia na gaveta de montagem do DMR-EH55, certificando-se de configurar os jumpers do disco PATA em **Master** ou **Cable Select** (conforme a fita flat original).
3. **Inicialização do Sistema:**
   Ao ligar o aparelho, o display frontal exibirá a sequência padrão `HELLO` $\rightarrow$ `PLEASE WAIT` e entrará normalmente em modo de operação ou exibirá o aviso `UNFORMAT`. O display **não exibirá mais `DRV NG` nem `HDDERR`**, pois o MBR e as partições de firmware necessárias ao AM34 estarão presentes.
4. **Formatação Oficial do Volume de Gravação:**
   Pelo controle remoto original ou teclado frontal:
   - Pressione a tecla **FUNCTIONS**.
   - Navegue até: **To Others** $\rightarrow$ **Setup** $\rightarrow$ **Others** $\rightarrow$ **HDD Management** $\rightarrow$ **Format**.
   - Confirme a formatação. O gravador preservará intactos os primeiros 100 MB com os firmwares e alocará todos os setores restantes da mídia (seja ela de 80 GB, 160 GB, 250 GB ou 500 GB) para o novo espaço de gravação MEIHDFS.

---

## 9. Conclusão Técnica

A análise de engenharia reversa do arquivo `panasonic DMR EH-55 Firmware hdd_100.bin` comprova de maneira inequívoca a arquitetura simbiótica entre a memória Flash da placa principal e o disco rígido na plataforma Panasonic DIGA. 

O disco rígido não é um mero repositório de arquivos do usuário, mas uma extensão física da memória de execução do sistema operacional AM34. A existência de cabeçalhos de redundância em espelho nos exatos 28 MB (`0x01C00000`), o catálogo transacional de 10 blocos de 128 KB em 8 MB (`0x00800000`), a partição principal de firmware de ~5,89 MB em 32 MB (`0x02000000`) e a partição secundária de 704 KB em 64 MB (`0x04000000`) constituem um sistema de armazenamento de alta sofisticação projetado para durabilidade, tolerância a quedas de energia e fidelidade absoluta na gravação audiovisual.
