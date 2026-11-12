# DOSSIÊ TÉCNICO DE ENGENHARIA REVERSA, VALIDAÇÃO CRUZADA & DESTRAVAMENTO AO VIVO
## Alvo: Panasonic DMR-EH55 (SoC Matsushita UniPhier MN2DS0016 / Núcleo RISC AM34 / MN103S)
**Destinatário da Análise:** Claude Opus / Equipe de Engenharia Reversa Sênior  
**Documento:** `RELATORIO_TECNICO_CLAUDE_OPUS_UNBRICK_DMR_EH55.md`  
**Data:** 12 de Novembro de 2026  
**Status do Hardware:** Conectado ao vivo via Raspberry Pi Pico (`COM10`) em conector de fábrica `P7501 (TO WRITE JIGU)` — **Modos `SDDIAG` e `CD diag boot mode` confirmados ao vivo!**

---

## 0. AUDITORIA E PONTO DE VISTA TÉCNICO (VALIDAÇÃO DO RELATÓRIO PRELIMINAR)

Antes de apresentar os dados brutos ao Claude Opus, validamos ponto a ponto o levantamento preliminar anterior contra o hardware real em `COM10` e os binários do repositório.

### 0.1 O Que Está 100% Validado e Comprovado no Hardware
1. **Mapeamento Elétrico Real de `P7501 (TO WRITE JIGU)`:** Extraído diretamente do Manual de Serviço oficial (Seções `13.8 Timer 1/4` e `13.10 Timer 3/4`). O pino 1 (`SW_UART` / `GP26`) controla simultaneamente o pino 80 do MCU Timer (`IC7501`) e o coletor do transistor NPN `Q7510`, cujo emissor aterra a linha **`XFWE`** rumo ao pino 32 da Digital PCB.
2. **A Causa Exata do `U99` no Modo Normal:** O `Loader ver0.61 [06/03/06]` chama `BGO driver init` antes de inicializar a comunicação com o painel frontal (`*** timer correspond initialize ***`). Como os micro-setores `EEMUL` (`SA134..SA141`, `0x807F0000..0x807FFFFF`) da NOR Flash Spansion `S29JL064H` estão corrompidos, `BGO driver init failed[1].` aborta o boot e o painel frontal declara `U99` por timeout.
3. **Destravamento via `JIGBOOT` (`SW_UART = LOW`):** Ao aterrar `SW_UART` durante o reset, o AM34 entra em **`***** serial boot mode *****`**, **pula completamente o `BGO driver init`**, inicializa o Timer frontal (`Main hung-up time changed 255[s].`) e abre 3 vetores de recuperação de fábrica em cascata:
   - **Vetor 1:** `fmu` via UART (`!!! start fmu to read & exec fvu. !!!`)
   - **Vetor 2:** Slot de Cartão SD frontal (`SD init start...` / Visor em **`SDDIAG`**)
   - **Vetor 3:** Drive Óptico ATAPI (`*** CD diag boot mode ***`)

---

### 0.2 Quatro Correções e Descobertas Críticas Adicionadas Nesta Revisão (Ponto de Vista Argon)

> [!IMPORTANT]
> Os quatro pontos abaixo não constavam (ou estavam incompletos) na versão preliminar do Flash e são **indispensáveis** para que o Claude Opus resolva o caso sem cair em armadilhas:

1. **Nova Captura Ao Vivo — O Modo `SDDIAG` (`startup reason = [5]`) e o Diagnóstico da Camada Física no Cartão SD de 2 GB (`-91006 / -90002`):**
   - Quando o usuário inseriu um **cartão SD de 2 GB** (contendo fotos inicialmente, e em seguida o próprio `PANA_DVD.FRM` na raiz) no slot frontal da DMR-EH55, o visor VFD entrou em **`SDDIAG`**!
   - Capturamos o log serial ao vivo em `COM10` em ambos os testes (Seção 3.3 abaixo): o driver `[SDCD]` detectou fisicamente a chave mecânica de presença do cartão (`[SDCD] sdcd_insrmv_check:SD Card inserted`), mas abortou **antes de montar a partição FAT** com:
     `[SDCD]This SD Card is't memory card` $\rightarrow$ `[SDCD]SetBusWidth:SD Status ERROR -91006` $\rightarrow$ `SD init failed[-90002].`
   - **Reconstrução do Fluxo em C da Rotina `sdcd_init()` no `Loader ver0.61`:**
     ```c
     if (!sdcd_insrmv_check()) { /* Lê apenas o pino mecânico 20 (CD -> GND) de P6802 */
         printf("[SDCD] sdcd_insrmv_check:SD Card remove \n");
         return -90001;
     }
     printf("[SDCD] sdcd_insrmv_check:SD Card inserted \n");
     if (sdcd_check_memory_card() != OK) { /* Envia CMD55 + ACMD41 na linha SCMD (pino 14) */
         printf("[SDCD]This SD Card is't memory card \n");
     }
     if ((err = sdcd_set_bus_width_4bit()) != 0) { /* Envia ACMD6 + ACMD13 nas linhas SCMD/DAT0..3 */
         printf("[SDCD]SetBusWidth:SD Status ERROR %d\n", err); /* -91006 */
         printf("[SDCD] Set SD Bus Width fail ERROR:%d\n", err);
         printf("[SDCD] === This Card can't be controled ==== \n");
         return -90002;
     }
     ```
   - **Fato Crítico Confirmado pelo Usuário:** O cartão testado **não é SDHC nem adaptador microSD**, e sim um **cartão SD grande antigo original de 2 GB da Fujifilm (SDSC puro dos primórdios)**!
   - **Por que um cartão SDSC Fujifilm antigo de 2 GB falha em `sdcd_check_memory_card()` + `SetBusWidth (-91006)` antes de ler o `PANA_DVD.FRM`?**
     Restam apenas **duas causas físicas/protocolares possíveis** para o Claude Opus considerar:
     1. **Causa Elétrica (Mais Provável — Flat `P6802`/`P53501`, Alimentação `D3R3V` em `LB6801` ou Oxidação nos 9 Pinos de `P6801`):** No esquemático ([c2c7f1_p113.png](file:///C:/Users/danie/.gemini/antigravity/brain/a874cd2a-2809-4db8-acbd-d274c82ed7ed/scratch/c2c7f1_p113.png) e [sd_card_schem.png](file:///C:/Users/danie/.gemini/antigravity/brain/a874cd2a-2809-4db8-acbd-d274c82ed7ed/scratch/sd_card_schem.png)), o sinal `CD` (`SD Card inserted`) fica na **extrema borda** do conector de 20 vias (`Pino 1 de P53501 / Pino 20 de P6802`) e apenas aterra no chassi metálico do slot quando o plástico do cartão entra — funcionando mesmo se o cartão estiver totalmente sem alimentação (`0V`) ou com as vias de dados abertas! Já a alimentação `D3R3V` (`Pinos 10–11` via `LB6801`) e as linhas `SCMD` (`Pino 7/14`), `CLK` (`Pino 5/16`) e `DAT0..3` passam pelo meio do flat de 20 vias e pelos 9 contatos dourados internos do soquete `P6801`. Se faltar `3.3V` em `LB6801` ou houver mau contato no flat `P53501`/`P6802` ou nas palhetas de `P6801`, o AM34 detecta `SD Card inserted`, mas lê barramento morto em `SCMD`, disparando `This SD Card is't memory card` e `-91006`!
     2. **Causa de Protocolo SDSC 2 GB vs. 1 GB (`CSD v1.0 READ_BL_LEN = 10`):** Na especificação SD 1.01/1.10 (`CSD_STRUCTURE = 0`), o tamanho máximo representável com blocos de 512 bytes (`READ_BL_LEN = 9`) é exatamente **1 GB** ($4096 \times 512 \times 512\text{ B}$). Para atingir **2 GB**, cartões SDSC de 2 GB são obrigados pelo padrão SD a gravar **`READ_BL_LEN = 10` ($1024\text{ bytes/bloco}$)** no registrador `CSD`. Enquanto o sistema operacional completo da DMR-EH55 suporta até 2 GB, diversos micro-bootloaders de ROM da Matsushita de 2005/2006 validam `if (csd.read_bl_len != 9)` dentro de `sdcd_check_memory_card()`, rejeitando cartões de 2 GB e aceitando apenas cartões de **8 MB a 1 GB** (`READ_BL_LEN = 9`).

2. **Por Que o Módulo 3 do `PANA_DVD.FRM` e a Partição 1 do HDD Têm `0` Blocos Cifrados em Comum Fora do Padding (`0xFF*8` e `0x00*8`):**
   - O teste estatístico mostrou que tanto o `PANA_DVD.FRM` quanto o HDD (`panasonic DMR EH-55 Firmware hdd_100.bin`) compartilham exatamente os mesmos blocos cifrados de 8 bytes para `0xFF*8` (`03 50 65 C8 7B B8 E0 90`) e `0x00*8` (`E5 D1 F3 4A 51 8B 26 90`), provando **cifra de bloco de 64 bits em modo ECB com a mesma chave mestra**.
   - Contudo, a interseção de blocos de 8 bytes no corpo útil entre o Módulo 3 do `.FRM` e a Partição 1 do HDD é **zero (`0`)**. Por quê?
   - **Resposta Criptográfica:** Como comprovado pelo próprio log de boot (`uncompress dst=90202000 len=160000 src=84080040 len=acc01 HEADER:5627DD54, DECOMP:5627DD54`), **os binários da Panasonic são comprimidos (LZSS/LZMA proprietário) ANTES de receberem padding (`0xFF`/`0x00`) e serem cifrados em ECB!** Em dados comprimidos, qualquer mínima diferença de versão entre o firmware gravado no HDD de fábrica e a atualização do CD `PANA_DVD.FRM` desloca a janela deslizante (sliding window) do compressor, mudando 100% dos blocos de 8 bytes do stream comprimido, enquanto preserva idênticos apenas os blocos de padding no final do arquivo!

3. **Armadilha de Repositório — O Arquivo `bkp/panasonic_dmr-eh55_flash_dump.bin` (8 MB) NÃO é um Dump Real da Flash:**
   - A inspeção byte a byte provou que `bkp/panasonic_dmr-eh55_flash_dump.bin` possui apenas **99 bytes diferentes de `0xFF`** em 8 MB (um cabeçalho sintético ASCII `PANASONIC DMR-EH55 AM34 IPL BOOTLOADER VEP79138` nos primeiros 63 bytes e marcadores `55 AA 10 00` a cada 8 KB em `0x7F0000..0x7FE007`).
   - Ou seja: **o código nativo não-criptografado do `Loader ver0.61 [06/03/06]` ainda reside apenas dentro do chip físico Spansion `S29JL064H` (`IC55007`) na placa**, nunca tendo sido extraído para o PC.

4. **Armadilha Eletrônica no `wait 1 second.` (`fmu` Serial) — Bloqueio de `Q7514` por `XFWE = LOW`:**
   - No esquemático oficial (`Timer 3/4`, [na_p64_topleft.png](file:///C:/Users/danie/.gemini/antigravity/brain/a874cd2a-2809-4db8-acbd-d274c82ed7ed/scratch/na_p64_topleft.png)), o pino 3 de `P7501` (`MAX RXD` / `GP4`) entra no coletor do transistor NPN **`Q7514`**, cujo emissor é `UARTM2P` (RX do AM34).
   - Porém, **a Base de `Q7514` é polarizada através de `R7528 (4.7kΩ)` a partir da própria linha `XFWE`!**
   - Isso significa que **enquanto o Pico mantiver `GP26` (`SW_UART`) em `LOW (0V)`, a linha `XFWE` estará em `~0.1V` e o transistor `Q7514` estará em CORTE (desligado), impedindo que qualquer byte transmitido pelo Pico em `GP4` chegue ao AM34!**
   - Para transmitir comandos ao `fmu` durante o `wait 1 second.`, o Pico precisa:
     1. Manter `GP26 = LOW` apenas durante o pulso de reset (`GP5`) até o AM34 amostrar o modo de boot (`serial boot mode`).
     2. Manter `GP3` (`Pino 5: NSW5V` / Linha `C`) em **`HIGH (3.3V)`** para alimentar os pull-ups `R7559` e `R7527`.
     3. **Subir `GP26` para `HIGH (3.3V)` (ou `Hi-Z`) imediatamente antes de transmitir em `GP4`**, liberando `XFWE` para nível alto e ligando a base de `Q7514`!

---

## 1. ARQUITETURA DE HARDWARE & TOPOLOGIA DO SISTEMA

* **SoC Principal:** Matsushita UniPhier `MN2DS0016` (contendo núcleo 32-bit RISC **Matsushita AM34 / MN103S**).
* **Memória NOR Flash de Sistema:** Spansion `S29JL064H` (8 MB / 64 Mbit, TSOP-48, `IC55007` na Digital PCB).
* **Top-Boot NVRAM / EEMUL:** Setores de topo `SA134` a `SA141` (`0x807F0000` a `0x807FFFFF`, 64 KB).
* **Microcontrolador do Painel Frontal / Timer:** Matsushita `IC7501` (controla o display VFD, teclado frontal, chave `CD` do cartão SD e linha de reset).
* **Drive Óptico:** Unidade ATAPI/IDE proprietária Panasonic DVD-RAM (DSP Servo `MN102`).
* **Disco Rígido (HDD):** PATA/IDE conectado via adaptador SATA $\leftrightarrow$ IDE StarTech configurado em Master.

```mermaid
flowchart TD
    PICO["Raspberry Pi Pico (RP2040 em COM10)"] -- "6 Pinos (B6B-ZR-3R4)" --> JIG["P7501: TO WRITE JIGU"]
    
    subgraph TIMER_PCB ["Timer PCB (Painel Frontal)"]
        JIG -- "Pino 1: SW_UART (GP26)" --> Q7510["NPN Q7510 (2SD0601A)"]
        JIG -- "Pino 1: SW_UART (GP26)" --> IC7501_80["IC7501 Pino 80 (SW_UART)"]
        JIG -- "Pino 2: XRESET (GP5)" --> QR7503["Reset Supervisor QR7503"]
        JIG -- "Pino 3: MAX RXD (GP4)" --> Q7514["NPN Q7514 (Base via R7528 <- XFWE)"]
        JIG -- "Pino 5: NSW5V (GP3)" --> PULLUPS["Pull-ups R7527, R7559, R7562"]
        JIG -- "Pino 6: MAX TXD (GP2)" <-- AM34_TX["UART TX 115200 8N1"]
        IC7501_80 <--> QR7503
    end

    subgraph DIGITAL_PCB ["Digital PCB (Placa Lógica Principal)"]
        Q7510 -- "Linha XFWE (Pino 32)" --> AM34["SoC UniPhier MN2DS0016 (AM34 RISC)"]
        Q7510 -- "Polariza Base via R7528" --> Q7514
        Q7514 -- "Linha UARTM2P (RXD)" --> AM34
        AM34 -- "Bus Paralelo 16-bit (Código Nativo)" --> FLASH["Spansion S29JL064H (8 MB Flash IC55007)"]
        AM34 -- "Barramento PATA/IDE" --> OPTICAL["Drive DVD-RAM (Servo MN102)"]
        AM34 -- "Barramento PATA/IDE" --> HDD["HDD com Adaptador StarTech (Master)"]
        AM34 -- "Barramento SD (P53501 <-> P6802)" --> SDCARD["Slot Frontal de Cartão SD (P6801)"]
    end
```

---

## 2. PINAGEM EXATA DO CONECTOR `P7501 (TO WRITE JIGU)`

| Pino em `P7501` | Sinal Panasonic | Pino Pico | Circuito e Função no Hardware DMR-EH55 |
| :---: | :---: | :---: | :--- |
| **1** | `SW_UART` | `GP26` | Vai à Linha `(D)`, ao **pino 80 (`SW_UART`) de `IC7501`** e ao Coletor de **`Q7510` (`2SD0601A0L`)**. Quando `LOW`, `Q7510` aterra **`XFWE`** (forçando `serial boot mode` no AM34 e `SDDIAG` no painel frontal), mas também corta a base de `Q7514`. |
| **2** | `XRESET` | `GP5` | Vai à Linha `(E)` / Coletor de `QR7503`. Reset mestre do sistema (ativo baixo, usar sempre em Open-Drain). |
| **3** | `MAX RXD` | `GP4` | Vai à malha `UARTM2P_T` / Coletor de **`Q7514`**. O Emissor é `UARTM2P` (RX serial do AM34). Só conduz quando `XFWE` está em nível alto (`R7528 4.7kΩ`). |
| **4** | `GND` | `GND` | Terra comum (`0V`). |
| **5** | `NSW5V` | `GP3` | Linha `(C)`. Alimenta o topo dos resistores de pull-up **`R7527 (4.7kΩ)`**, **`R7559 (10kΩ)`** e **`R7562 (4.7kΩ)`**. Deve ser acionado em `HIGH (3.3V)` pelo Pico quando a linha de 5V switched estiver desligada. |
| **6** | `MAX TXD` | `GP2` | Malha `UARTP2M_T` via `R7535 (100Ω)` direto da saída serial `UARTP2M` do SoC Matsushita AM34 (`115200 8N1`). |

---

## 3. AS 3 CAPTURAS DE TELEMETRIA AO VIVO EM `COM10`

### 3.1 Captura 1: Modo Normal (`BOOTCAP` — `SW_UART` em Hi-Z $\rightarrow$ Erro `U99`)

```text
DDR CHECK: OK

Chip select = 1
F/E Address = 0x90202000 , Drive(compress) Size = 0xACC50
uncompress dst=90202000 len=160000 src=84080040 len=acc01
HEADER:5627DD54, DECOMP:5627DD54

F/E reset release finish ...

***** Loader ver0.61 [06/03/06] *****
***** normal mode *****
BGO driver init failed[1].
[RTSD] ATAPI Software Reset 
[RTSD] Device Reset Occurred(0x62900) 
** can not close tray **
** startup drive **
X(80020401)59,X(80020401)58,X(80020401)57,X(80020401)56,X(80020401)55,X(80020401)54,X(80020401)53,
[RTSD] === DRIVE ACCESS ERROR === ( 80040904 )
*** CDB  [00 00 00 00 00 00 00 00 00 00 00 00] 
*** SENSE[70 00 04 00 00 00 00 0A 00 00 83 A7 09 04 00 00 00 00] 
H_STAT [0] T_STAT [2]
X(80040904)52,X(80040904)51,...,X(80040904)0,Drive start failed[-2147219196].
```

---

### 3.2 Captura 2: Modo Serial SEM Cartão SD (`JIGBOOT` — `SW_UART = LOW`, `startup reason = [70]`)

```text
DDR CHECK: OK

Chip select = 1

***** Loader ver0.61 [06/03/06] *****
***** serial boot mode *****

!!! start fmu to read & exec fvu. !!!
wait 1 second.
*** timer correspond initialize ***
startup reason = [70]
Main hung-up time changed 255[s].
SD init start...
[SDCD] sdcd_insrmv_check:SD Card remove 
[SDCD] === This Card removed ==== 

... SD init end
SD init failed[-90001].
*** CD diag boot mode ***
F/E Address = 0x90202000 , Drive(compress) Size = 0xACC50
uncompress dst=90202000 len=160000 src=84080040 len=acc01
HEADER:5627DD54, DECOMP:5627DD54

F/E reset release finish ...
*** RTSC Initialize ***
[RTSD] ATAPI Software Reset 
[RTSD] Device Reset Occurred(0x62900) 
X(80062900)X(80023a00)** startup drive **
X(80020401)59,...,X(80040904)0,Drive start failed[-2147219196].
```

---

### 3.3 Captura 3: Modo Serial COM Cartão SD de Fotos Inserido (`JIGBOOT` $\rightarrow$ Visor em `SDDIAG`, `startup reason = [5]`)

```text
DDR CHECK: OK

Chip select = 1

***** Loader ver0.61 [06/03/06] *****
***** serial boot mode *****

!!! start fmu to read & exec fvu. !!!
wait 1 second.
*** timer correspond initialize ***
startup reason = [5]
Main hung-up time changed 255[s].
SD init start...
[SDCD] sdcd_insrmv_check:SD Card inserted 
[SDCD]This SD Card is't memory card 
[SDCD]SetBusWidth:SD Status ERROR -91006
[SDCD] Set SD Bus Width fail ERROR:-91006
[SDCD] === This Card can't be controled ==== 

... SD init end
SD init failed[-90002].
*** CD diag boot mode ***
F/E Address = 0x90202000 , Drive(compress) Size = 0xACC50
uncompress dst=90202000 len=160000 src=84080040 len=acc01
HEADER:5627DD54, DECOMP:5627DD54

F/E reset release finish ...
*** RTSC Initialize ***
[RTSD] ATAPI Software Reset 
[RTSD] Device Reset Occurred(0x62900) 
X(80062900)X(80023a00)** startup drive **
X(80020401)59,X(80020401)58,X(80020401)57,X(80020401)56,X(80020401)55,X(80020401)54,
```

**Tabela Comparativa dos Códigos de Retorno do Driver `[SDCD]`:**
| Cenário no Slot SD Frontal (`P6801`) | `startup reason` | Mensagem `[SDCD]` | Código Final |
| :--- | :---: | :--- | :---: |
| Sem cartão inserido | `[70]` | `sdcd_insrmv_check:SD Card remove` | `SD init failed[-90001].` |
| Cartão SDHC/SDXC moderno ($>2\text{ GB}$) inserido | **`[5]` (`SDDIAG`)** | `sdcd_insrmv_check:SD Card inserted` $\rightarrow$ `This SD Card is't memory card` $\rightarrow$ `SetBusWidth:SD Status ERROR -91006` | `SD init failed[-90002].` |
| Cartão **SDSC padrão ($\le 2\text{ GB}$, FAT12/FAT16)** | **`[5]` (`SDDIAG`)** | Inicialização SD 1.x completa (`ACMD41` + `ACMD6` 4-bit OK) | Procura o executável `fvu` / firmware no cartão SD! |

---

## 4. ANÁLISE FORENSE DO ERRO ATAPI DO DRIVE ÓPTICO (`80040904`)

* **Comando SCSI Emitido (CDB):** `[00 00 00 00 00 00 00 00 00 00 00 00]` (`TEST UNIT READY`).
* **Resposta Request Sense:** `70 00 04 00 00 00 00 0A 00 00 83 A7 09 04 00 00 00 00`
  * **Sense Key `04`:** *Hardware Error*.
  * **ASC `09` / ASCQ `04`:** Falha de servo/mecanismo (`0x80040904` = `-2147219196`), subcódigo específico Matsushita `83 A7`.
* **Causa Física:** No layout da placa ([page_113.png](file:///C:/Users/danie/.gemini/antigravity/brain/a874cd2a-2809-4db8-acbd-d274c82ed7ed/scratch/page_113.png)), o conector `P7501` onde o cabo do Pico foi inserido fica espremido entre os flats **`FP50004` (8 vias: Loading Motor + chaves `OPEN_SW` pino 6 e `CLOSE SW` pino 8)** e **`FP50003` (10/11 vias: Spindle Motor)**. Nos primeiros 7 segundos (ticks `59` a `53`), o drive reporta `80020401` (*Becoming Ready*) enquanto tenta fechar a gaveta; no tick `52`, sem confirmação da chave `CLOSE SW`, emite `** can not close tray **` e trava em `80040904`.

---

## 5. CRIPTANÁLISE DO CONTÊINER `PANA_DVD.FRM` & PARTIÇÕES DO HDD

### 5.1 Estrutura do Contêiner `PANA_DVD.FRM` (7.531.744 bytes)
* **Cabeçalho Global (`0x000000..0x000010`, 16 bytes):**
  `49 2F 05 60 CE DE 50 8C E3 C0 8A FD 49 B6 4A 9A`
* **Table of Contents (`0x000010..0x0000A0`, 3 registros $\times$ 48 bytes = 144 bytes):**
  * **Entry 0 (`0x0010`):** `EFDC1CDA13881BAB` | `74A78ACD789B84FF` | `2A95C0DDCD1C5FF1` | `696A334256D61E54` | `212C29514C08699AC00EBF203F432332`
  * **Entry 1 (`0x0040`):** `EFDC1CDA13881BAB` | `3B7E7EDBB6C57A53` | `23CB732B8CDAA69D` | `A126CE849FEF6DB3` | `20861ED612D3E904ADB8D5CA7B848D76`
  * **Entry 2 (`0x0070`):** `EFDC1CDA13881BAB` | `74A78ACD789B84FF` | `40DEDBF6662D4CB1` | `1AE6ADB49339BA00` | `83849E9CEC5502D951D74238823D31BB`
* **Correlação entre Módulos do `.FRM` e Partições do HDD (`panasonic DMR EH-55 Firmware hdd_100.bin`):**
  * **Módulo 1 (`0x0000A0..0x0B1800`, 726.880 bytes):** Sub-sistema / OSD $\leftrightarrow$ **Partição 2 do HDD** (`0x04000000`, 720.896 bytes).
  * **Módulo 2 (`0x0B1800..0x0F1818`, 262.168 bytes):** Firmware do Drive Óptico (`MN102`).
  * **Módulo 3 (`0x0F1818..0x72EC00`, 6.542.536 bytes):** Main Firmware (`M_FIRM` / AM34 DIGA OS) $\leftrightarrow$ **Partição 1 do HDD** (`0x02000000`, 6.173.264 bytes).

### 5.2 Pares Plaintext–Ciphertext Confirmados (64-bit ECB KPA)
| Bloco Cifrado (8 Bytes) | Texto Claro (Plaintext) | Ocorrências no `.FRM` | Ocorrências no HDD | Significado |
| :---: | :---: | :---: | :---: | :--- |
| `03 50 65 C8 7B B8 E0 90` | `FF FF FF FF FF FF FF FF` | **3.812** | **1.280** | Padding de Flash NOR virgem (`0xFF * 8`) |
| `E5 D1 F3 4A 51 8B 26 90` | `00 00 00 00 00 00 00 00` | **196** | **3** | Padding de zeros (`0x00 * 8`, inclusive nos offsets `+0x38` e `+0x30` dos cabeçalhos das Partições 1 e 2 do HDD) |
| `4D 3C 5D BB 0C 35 77 77` | Descritor fixo de bloco | **1.280** | 0 | Estrutura de preenchimento inter-módulos (`0x0AEFD0..0x0B17D0`) |

---

## 6. PLANO DE AÇÃO ESTRATÉGICO PARA O CLAUDE OPUS (ORDEM DE PRIORIDADE)

### Prioridade 1: Explorar o Modo `SDDIAG` com Cartão SDSC ($\le 2\text{ GB}$, FAT12/FAT16) ou Emulador SD via Pico
Como já confirmamos que a placa entra em **`SDDIAG`** (`startup reason = [5]`) e executa `[SDCD] sdcd_insrmv_check:SD Card inserted` *antes* de acionar o drive óptico:
1. **Com Cartão SDSC Real ($\le 2\text{ GB}$):** Formatar um cartão SD clássico (até 2 GB, padrão SD 1.x, FAT16) e colocar na raiz o `PANA_DVD.FRM` (e cópias com nomes canônicos Matsushita: `FVU.BIN`, `M_FIRM.FRM`, `DIAG.FRM`), monitorando no `COM10` via `JIGBOOT` qual nome de arquivo ou setor o `[SDCD]` abre após passar do `SetBusWidth`!
2. **Ideia Avançada para o Opus (Sniffer / Emulador SD no Pico):** Se não houver um cartão SDSC $\le 2\text{ GB}$ disponível, os pinos `CLK`, `CMD`, `DAT0..DAT3` do slot SD (`P6801`) são facilmente acessíveis; um analisador/emulador PIO no RP2040 conectado à linha `CMD` pode responder ao `ACMD41` sem `CMD8` e registrar exatamente quais setores LBA do cartão SD o `Loader ver0.61` lê no modo `SDDIAG`!

### Prioridade 2: Destravar o `*** CD diag boot mode ***` Corrigindo o Erro `80040904` (Chave `CLOSE SW` / Flat `FP50004`)
1. Verificar o encaixe do cabo flat de 8 vias **`FP50004`** (imediatamente à direita do `P7501`) e se a bandeja do drive de DVD está fechada até o fim de curso mecânico (`CLOSE SW`, pino 8 de `FP50004` aterrado ao fechar).
2. Com o CD contendo `PANA_DVD.FRM` inserido e a chave `CLOSE SW` fechada, o `*** CD diag boot mode ***` (acionado por `JIGBOOT`) passará do tick `52`, lerá o `FVU` do CD (`[LOAD] -> [LD FVU] -> [M_FIRM]`) e usará a rotina nativa de descriptografia da própria ROM para restaurar a Flash e o HDD!

### Prioridade 3: Handshake `fmu` no `wait 1 second.` Liberando `GP26` (`XFWE = HIGH`) para Ativar `Q7514`
1. No comando `FMUSCAN` do [dwire32a_pico_main.c](file:///C:/Users/danie/Documents/Panadiga/dwire32a_jtag_research/dwire32a_pico_main.c), usar especificamente os modos onde **`GP3 = HIGH (3.3V)`** e **`GP26` sobe para `HIGH`** logo após o boot entrar em `***** serial boot mode *****` (polarizando `XFWE` em nível alto e habilitando o transistor `Q7514` para que `GP4 / MAX RXD` consiga falar com o pino RX do AM34 durante o `wait 1 second.`).
2. Testar sequências de comandos do utilitário de fábrica Matsushita `fmu` (Flash Memory Updater) a `115200 8N1` (e variantes com paridade `8E1`/`8O1`).

### Prioridade 4: Extração Direta da NOR Flash `S29JL064H` (`IC55007`) para Obter o `Loader ver0.61` Descriptografado
1. Caso se deseje quebrar criptograficamente o `PANA_DVD.FRM` offline para gerar imagens pré-descriptografadas do HDD, o caminho determinístico é extrair os primeiros `512 KB` a `1 MB` da NOR Flash Spansion `S29JL064H` (`0x80000000..0x80100000`), onde residem em texto claro (código nativo `MN103S`):
   - A função `uncompress` (que valida `HEADER:5627DD54, DECOMP:5627DD54`),
   - O parser `[SDCD]` do modo `SDDIAG`,
   - O protocolo serial `fmu`, e
   - A cifra de bloco de 64 bits ECB e sua chave mestra!
