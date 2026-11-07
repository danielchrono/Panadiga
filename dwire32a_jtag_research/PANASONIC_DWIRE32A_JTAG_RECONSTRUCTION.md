# Projeto de Engenharia Reversa: Reconstrução da Ferramenta DWire32A / OCD Panasonic

Documento de especificação técnica e blueprint definitivo para reconstrução da interface proprietária de gravação e diagnóstico in-circuit do processador **Panasonic MN2DS0016 (UniPhier / AM34)** através do conector de 6 pinos (`CN55001`) na placa digital (`VEP79138`).

---

## 1. Identificação Real da Interface: Panasonic DWire32A

Diferente de uma UART assíncrona comum, o conector de 6 pinos na placa digital é a interface física do protocolo síncrono proprietário **Matsushita/Panasonic DWire32A** (*On-Chip Debug Serial Interface*).

* **Núcleo do Processador:** Matsushita **AM34 / MN103S** (arquitetura RISC de 32 bits, variante avançada da família `MN10300`).
* **Ecossistema Oficial de Fábrica:**
  * **Emulador In-Circuit:** Panasonic `PanaX NEO` / `PX-ICE` (posteriormente Nuvoton `Nu-EX`).
  * **Adaptador de Conexão:** Panasonic `PX-ADPDWA` (*DWire32A Connection Adapter*).
  * **Cabo de Comunicação:** `PX-CBLD14` / chicote de 6 pinos.
  * **Ambiente de Software:** *DebugFactory Builder*, *EX Commander*, *fl103* (Flash Loader), *db103* (Debugger).

```text
       CONECTOR DE DIAGNÓSTICO DWIRE32A / P7501 [B6B-ZR-3R4] (1.5mm Pitch)
       (Orientação Física na Placa VEP79138: Contagem do Flat 10V para Flat 8V)
       ┌───────────┐
Pin 1  │ ● EXTRG   │ External Trigger (Gatilho de Hardware / Breakpoint Trace)
Pin 2  │ ● SCLK    │ Serial Clock (Clock Síncrono gerado pelo Host RP2040 @ 1.0 MHz)
Pin 3  │ ● GND     │ Ground Comum (0.0Ω para o Chassi Metálico!)
Pin 4  │ ● SDATA   │ Serial Data (Dados Bidirecionais Half-Duplex, Pull-up 4.7kΩ)
Pin 5  │ ● nRESET  │ Hardware Reset (Linha ativa em nível baixo / XRST / Open-Drain)
Pin 6  │ ● VDD     │ Target Power Reference (Sense de +3.3V da placa)
       └───────────┘
```

---

## 2. Projeto Elétrico da Interface de Hardware (RP2040 $\leftrightarrow$ Placa Digital `VEP79138`)

Para conectar o **Raspberry Pi Pico (RP2040)** ao conector de 6 pinos da placa verde sem risco de colisão de barramento (*bus contention*) durante a troca de direção da linha bidirecional `SDATA`, a interface física deve incluir resistores de amortecimento em série e controle em dreno aberto (*Open-Drain*) para a linha de reset.

| Pino DWire32A (Físico) | Pino Esquemático P7501 | Sinal na Placa | GPIO RP2040 | Componente em Série / Proteção | Função Elétrica |
| :---: | :---: | :--- | :--- | :--- | :--- |
| **Pino 1** | Pino 6 | `EXTRG` | `GP2` (Pino 4) | Resistor $100\,\Omega$ em série | Gatilho externo / Request de parada OCD |
| **Pino 2** | Pino 5 | `SCLK` | `GP3` (Pino 5) | Resistor $47\,\Omega$ a $100\,\Omega$ em série | Clock síncrono gerado pelo PIO Master (1.0 MHz) |
| **Pino 3** | **Pino 4** | **`GND`** | **`GND` (Pino 38)** | Ligação direta de baixa impedância | **Referência comum de 0V ($0.0\,\Omega$ ao chassi)** |
| **Pino 4** | Pino 3 | `SDATA` | `GP4` (Pino 6) | Resistor $100\,\Omega$ + Pull-up $4.7\text{ k}\Omega$ p/ Target VDD | Linha de dados Half-Duplex bidirecional |
| **Pino 5** | Pino 2 | `nRESET` | `GP5` (Pino 7) | Resistor $100\,\Omega$ (**Modo Open-Drain**) | Controle da malha `XRST` / `D_RESET`. Nunca push-pull! |
| **Pino 6** | Pino 1 | `VDD` | `GP26 / ADC0` | Leitura via resistor $1\text{ k}\Omega$ | Monitoramento da presença de 3.3V na placa |

### Regras Críticas de Montagem Elétrica

1. **Prevenção de Curto em `nRESET` (Emulação Open-Drain):**
   * O GPIO `GP5` do RP2040 **jamais** deve dirigir nível lógico alto (`3.3V` push-pull) contra a linha `nRESET` da placa, pois o CI supervisor de reset da Panasonic puxa a linha para `GND` simultaneamente durante o ciclo de inicialização.
   * No código do RP2040, `nRESET` opera exclusivamente alternando a direção do pino:
     * **Ativar Reset (`0V`):** Configurar `GP5` como **saída** com nível lógico `0`.
     * **Liberar Reset (`High-Z`):** Configurar `GP5` como **entrada** (deixando o resistor de pull-up original da placa verde elevar a linha naturalmente para `3.3V`).

2. **Proteção contra Colisão em `SDATA`:**
   * O resistor de $100\,\Omega$ em série no pino `GP4` limita a corrente de pico a no máximo $33\text{ mA}$ caso o processador `MN2DS0016` e o RP2040 tentem transmitir simultaneamente durante a fase de descoberta de opcodes.

3. **Desbloqueio Físico do Pino 14 (`WP#/ACC`) da Spansion para Gravação In-Circuit:**
   * Na arquitetura **Model 00** da Spansion `S29JL064H70TFI00` (`IC55007`), o pino 14 (`WP#`) protege por hardware os dois primeiros micro-setores de boot (`SA0` e `SA1`) e também **os dois últimos micro-setores do topo (`SA140` e `SA141`, faixa `0x7FC000` a `0x7FFFFF`)** caso esteja aterrado (`0V`).
   * Antes de executar qualquer operação de *Erase* in-circuit, meça com o multímetro a tensão na **ilha quadrada estanhada (*bottom-right*)** adjacente ao Pino 14:
     * Se ela medir **`3.3V`** com o aparelho alimentado, todos os setores já estão liberados por hardware de fábrica.
     * Se medir **`0V`**, basta injetar `3.3V` nessa ilha através de um resistor de $1\text{ k}\Omega$ durante o processo para destravar a gravação dos setores finais da `EEMUL`.

---

## 3. Máquina de Estados PIO (RP2040) para o Barramento DWire32A

O subsistema **PIO (Programmable I/O)** do RP2040 permite implementar o transceptor síncrono Half-Duplex em hardware puro, trocando a direção do pino `SDATA` na borda exata do `SCLK` com ciclo de *turnaround* (Hi-Z) determinístico sem jitter de interrupção.

### Código PIO Assembly (`dwire32a.pio`)

```pio
.program dwire32a
.side_set 1 opt

; Pin mapping:
; OUT / IN / SET / PINDIRS base: GP4 (SDATA)
; SIDE-SET base: GP3 (SCLK)
;
; Formato do Header na FIFO TX (Shift Right):
; Bits 7..0 : (n_bits - 1) -> carregado em X
; Bit 8     : Direcao (1 = TX Escrita, 0 = RX Leitura) -> carregado em Y

.wrap_target
public start:
    pull block          side 0  ; Puxa o Header de 32 bits da FIFO TX para o OSR
    out x, 8            side 0  ; Carrega (n_bits - 1) nos bits 7..0 para X
    out y, 1            side 0  ; Carrega a flag de direcao (Bit 8) para Y
    jmp !y, rx_mode     side 0  ; Se Y == 0, salta para o modo de recepcao

tx_mode:
    pull block          side 0  ; Descarta o resto do Header e puxa a palavra DATA
    set pindirs, 1      side 0  ; Configura SDATA como SAIDA
tx_loop:
    out pins, 1         side 0 [1] ; Coloca 1 bit em SDATA com SCLK=0 (Setup)
    nop                 side 1 [1] ; Sobe SCLK=1 (Hold / CPU amostra na borda de subida)
    jmp x--, tx_loop    side 0  ; Repete para todos os bits
    set pindirs, 0      side 0  ; Fim do TX: devolve SDATA para High-Z (Turnaround)
    jmp start           side 0

rx_mode:
    set pindirs, 0      side 0 [1] ; Garante SDATA como ENTRADA + Turnaround delay
rx_loop:
    nop                 side 1 [1] ; Sobe SCLK=1 (CPU coloca o bit na linha SDATA)
    in pins, 1          side 0 [1] ; Desce SCLK=0 e amostra SDATA no ISR
    jmp x--, rx_loop    side 0  ; Repete para todos os bits
    push block          side 0  ; Envia a palavra lida no ISR para a FIFO RX do ARM
.wrap
```

---

## 4. Sequência de Entrada no Modo OCD (*Hardware Break-In*) e Mapeamento de Comandos

Na família **Matsushita MN103S / AM34**, o bloco **OCD (*On-Chip Debugger*)** precisa interceptar o núcleo **antes** que ele comece a executar o código corrompido da NOR Flash que culmina no travamento em `U99`.

### 1. Sequência Temporal de Captura no Reset (*Cold Break-In*)

O RP2040 executa a seguinte rotina precisa para congelar o processador `MN2DS0016` em *Debug Halt*:

1. **Estado Inicial:** Puxar `nRESET` para `0V` (nível baixo por $50\text{ ms}$), mantendo `SCLK` em `0V`, `EXTRG` em `0V` (ou ativo conforme polaridade de gatilho) e `SDATA` em nível alto via pull-up.
2. **Ativação do Clock de Debug durante o Reset:** Ainda com `nRESET = 0V`, o RP2040 gera **32 pulsos de clock em `SCLK`** a $1\text{ MHz}$. Isso energiza e acorda o domínio de clock assíncrono da unidade OCD integrada no SoC.
3. **Liberação do `nRESET` com `EXTRG` Ativo:** O RP2040 libera o pino `nRESET` (transição para entrada High-Z, subindo para `3.3V`) mantendo `EXTRG` ativo (ou transmitindo imediatamente a palavra de parada de debug em `SDATA`).
4. **Verificação Física de Sucesso (Prova de Parada da CPU):**
   * Se a CPU entrar em **OCD Halt**, ela **não executa o bootloader** e não estabelece comunicação com o microcontrolador de painel (`IC7001` na placa marrom): o display frontal permanece estático no estado inicial ou não avança para `U99` no tempo habitual, e o pino `SDATA` passa a modular níveis lógicos em resposta aos pulsos de leitura do RP2040.

### 2. Scanner Automatizado no RP2040 para Descobrir os Opcodes DWire32A

O firmware do RP2040 conta com uma rotina de varredura automatizada (`dwire_probe_opcodes`) para testar combinações de comando e polaridade:

```c
// Estrutura de varredura para descobrir o opcode de leitura de ID / Status do OCD
void dwire_probe_opcodes(void) {
    for (uint8_t mode = 0; mode < 4; mode++) { // Testa CPOL/CPHA e estado de EXTRG
        for (uint16_t cmd = 0x00; cmd <= 0xFF; cmd++) {
            target_enter_reset_halt(mode);
            
            // Envia 8 bits de comando (ex: Read OCD ID / DCSR)
            pio_dwire_write_bits(cmd, 8);
            
            // Le 32 bits de resposta em SDATA
            uint32_t resp = pio_dwire_read_bits(32);
            
            // Se a resposta nao for 0xFFFFFFFF (pull-up vazio) nem 0x00000000:
            if (resp != 0xFFFFFFFF && resp != 0x00000000) {
                printf("[HIT!] Mode=%d | CMD=0x%02X -> RESP=0x%08lX\n", mode, cmd, resp);
            }
        }
    }
}
```

Assim que o processador responder com qualquer sequência diferente de `0xFFFFFFFF` e `0x00000000`, confirma-se a captura do barramento e o registrador de controle (`CHIP_ID` / `DCSR`).

---

## 5. Mapa de Memória Físico da Arquitetura AM34 (`MN2DS0016`)

Conforme a especificação técnica da arquitetura **Matsushita AM33v2 / AM34** (mapeada no kernel Linux em `arch/mn10300` e nas árvores GNU GCC/Binutils), o espaço de endereçamento linear de 32 bits divide-se em regiões cacheadas e acessos diretos não cacheados:

| Faixa de Endereço (Hex) | Região da Arquitetura AM34 | Dispositivo Mapeado na Placa `VEP79138` |
| :--- | :--- | :--- |
| `0x00000000` – `0x007FFFFF` | Espaço de Boot (Com Cache) | Espelho da NOR Flash Spansion `S29JL064H` (8 MB) |
| **`0x80000000` – `0x807FFFFF`** | **Espaço Físico Direto (Sem Cache)** | **Acesso Físico Direto à Spansion `IC55007` (`CS0` / `SB0`)** |
| `0x8C000000` / `0x90000000` | Espaço Físico SDRAM (Sem Cache) | Memórias SDRAM Elpida `IC53202` e `IC53203` |
| `0xC0000000` – `0xDFFFFFFF` | Periféricos Internos SoC | Controladores Serial (`SC0`–`SC2`), Timers, GPIO, Barramento ATA/IDE |
| `0xFF000000` – `0xFFFFFFFF` | Registradores de Núcleo e OCD | `DCSR` (Debug Control), `IBCR` (Breakpoints), `MEMCTR` |

> [!CRITICAL]
> **Regra de Ouro para Gravação JEDEC via CPU:**
> Todo comando JEDEC emitido pelo *RAM Loader* para apagar ou programar a memória Flash Spansion **deve obrigatoriamente utilizar a faixa de endereço físico sem cache (`0x80000000` + offset)** e jamais o espelho `0x00000000`.
> Se um comando de escrita for emitido em `0x00000000` com o cache L1 de dados ativo, a CPU armazenará os ciclos de desbloqueio `0xAA` / `0x55` nas linhas de cache internas, e os pulsos de barramento jamais alcançarão os pinos físicos da memória Flash.

---

## 6. Código do "RAM Loader" (Driver JEDEC Nativo AM34 para Spansion `S29JL064H`)

Quando a unidade OCD assume o controle da CPU, o host injeta na SRAM interna um micro-binário de posição independente (< 200 bytes) que executa nativamente o algoritmo JEDEC em barramento de 16 bits (`Word Mode`) para apagar os 8 micro-setores finais de 8 KB (`SA134` a `SA141`, faixa `0x7F0000` a `0x7FFFFF`).

### Cálculo dos Endereços de Desbloqueio em 16 Bits (`Word Mode`)

No modo de palavra de 16 bits (`BYTE# = High`), os endereços JEDEC de 16 bits (`0x555` e `0x2AA`) são transladados pelo barramento de endereços de bytes da CPU através de um deslocamento à esquerda de 1 bit:
* `Unlock Addr 1`: `0x555 << 1` = **`0xAAA`** (Dado: `0x00AA`)
* `Unlock Addr 2`: `0x2AA << 1` = **`0x554`** (Dado: `0x0055`)
* **Endereço Base Uncached:** `0x80000000`
* **Alvos de Apagamento (8 micro-setores de 8 KB da `EEMUL`):**
  * `0x807F0000` (`SA134`)
  * `0x807F2000` (`SA135`)
  * `0x807F4000` (`SA136`)
  * `0x807F6000` (`SA137`)
  * `0x807F8000` (`SA138`)
  * `0x807FA000` (`SA139`)
  * `0x807FC000` (`SA140`)
  * `0x807FE000` (`SA141`)

### Código C Nativo Independente de Posição (`ram_loader.c`)

Compilado via `mn10300-elf-gcc -mam33-2 -Os -fPIC -nostdlib -c ram_loader.c`:

```c
/**
 * @file ram_loader.c
 * @brief Autonomous AM34 / MN103S position-independent SRAM payload.
 *
 * Injected into internal SRAM via the Panasonic DWire32A On-Chip Debugger (OCD)
 * to erase the corrupted NVRAM emulation partition (EEMUL) on the Spansion
 * S29JL064H70TFI00 NOR Flash (IC55007), clearing the DMR-EH55 U99 fatal state.
 *
 * Target Processor: Panasonic MN2DS0016 (UniPhier / AM34 / MN103S)
 * Target Hardware:  Panasonic DMR-EH55 Digital Main Board (VEP79138)
 *
 * Compilation:
 *   mn10300-elf-gcc -mam33-2 -std=c11 -Os -fPIC -nostdlib -c ram_loader.c -o ram_loader.o
 *   mn10300-elf-objcopy -O binary -j .text.entry -j .text ram_loader.o ram_loader.bin
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include <stdint.h>
#include <stdbool.h>

/* =========================================================================
 * Physical Memory Bus Architecture
 * ========================================================================= */

/**
 * @brief Direct Physical Uncached Region Base Address.
 *
 * CRITICAL ARCHITECTURAL REQUIREMENT:
 * The AM34 architecture caches accesses to the mirror at 0x00000000.
 * In order for JEDEC command cycles to be driven out to external memory
 * pins (CS0/SB0), all flash register accesses MUST target the physical
 * uncached aperture starting at 0x80000000.
 */
#define FLASH_UNCACHED_BASE         0x80000000UL

/**
 * @brief JEDEC Word-Mode Byte-Bus Address Offsets.
 *
 * The Spansion S29JL064H operates in 16-bit word mode (BYTE# = High).
 * In 16-bit mode, standard JEDEC address cycles (0x555 and 0x2AA) are shifted
 * left by 1 bit on the CPU byte-addressed address bus:
 *   Addr1: 0x555 << 1 = 0x0AAA
 *   Addr2: 0x2AA << 1 = 0x0554
 */
#define JEDEC_UNLOCK_ADDR1_OFFSET   0x0AAAUL
#define JEDEC_UNLOCK_ADDR2_OFFSET   0x0554UL

/* =========================================================================
 * JEDEC Standard Flash Command Definitions (16-bit Word Mode)
 * ========================================================================= */

#define JEDEC_CMD_AA                0x00AAU /**< Unlock cycle 1 and 4 command */
#define JEDEC_CMD_55                0x0055U /**< Unlock cycle 2 and 5 command */
#define JEDEC_CMD_ERASE_SETUP       0x0080U /**< Erase setup command */
#define JEDEC_CMD_SECTOR_ERASE      0x0030U /**< Sector erase initiate command */
#define JEDEC_CMD_READ_RESET        0x00F0U /**< Reset array to Read Mode */

/* =========================================================================
 * Hardware Status Polling Bitmasks (Data# Polling DQ7 / Exceeded Timing DQ5)
 * ========================================================================= */

#define JEDEC_POLL_DQ7_MASK         0x0080U /**< Data# Polling: Inverted data during erase; 1 when done */
#define JEDEC_ERROR_DQ5_MASK        0x0020U /**< Exceeded Timing Limit flag (timeout error) */

/* =========================================================================
 * Spansion S29JL064H Top-Boot Partition Boundaries (EEMUL Region)
 * ========================================================================= */

/** Base address of the 8 x 8 KB parameter micro-sectors (SA134 to SA141). */
#define SPANSION_NVRAM_BASE_ADDR    0x007F0000UL

/** Upper limit boundary of the 64 Mbit (8 MB) flash memory space. */
#define SPANSION_NVRAM_END_ADDR     0x00800000UL

/** Micro-sector size stride (8 KB per sector). */
#define SPANSION_MICROSECTOR_STRIDE 0x00002000UL

/* =========================================================================
 * Hardware Access Macros
 * ========================================================================= */

#define FLASH_REG16(addr) (*(volatile uint16_t *)((uintptr_t)(addr)))
#define JEDEC_REG_UNLOCK1 FLASH_REG16(FLASH_UNCACHED_BASE + JEDEC_UNLOCK_ADDR1_OFFSET)
#define JEDEC_REG_UNLOCK2 FLASH_REG16(FLASH_UNCACHED_BASE + JEDEC_UNLOCK_ADDR2_OFFSET)

/* =========================================================================
 * JEDEC Sector Erase Algorithm
 * ========================================================================= */

/**
 * @brief Erase a specific sector on Spansion S29JL064H and poll DQ7 for completion.
 *
 * Implements the standard 6-cycle JEDEC sector erase sequence followed by
 * DQ7 Data# Polling with DQ5 timeout monitoring.
 *
 * @param sector_offset Relative byte offset of sector within flash (e.g. 0x007F0000).
 * @return true if sector erased successfully, false on timeout/hardware fault.
 */
static bool spansion_erase_sector(uint32_t sector_offset) {
    volatile uint16_t *sector_ptr =
        (volatile uint16_t *)((uintptr_t)(FLASH_UNCACHED_BASE + sector_offset));

    /* 6-cycle JEDEC Sector Erase command sequence in 16-bit word mode */
    JEDEC_REG_UNLOCK1 = JEDEC_CMD_AA;
    JEDEC_REG_UNLOCK2 = JEDEC_CMD_55;
    JEDEC_REG_UNLOCK1 = JEDEC_CMD_ERASE_SETUP;
    JEDEC_REG_UNLOCK1 = JEDEC_CMD_AA;
    JEDEC_REG_UNLOCK2 = JEDEC_CMD_55;
    *sector_ptr       = JEDEC_CMD_SECTOR_ERASE;

    /*
     * Poll DQ7 Data# Polling:
     * While erase is in progress, DQ7 outputs 0.
     * When erase completes, the sector is 0xFFFF and DQ7 outputs 1.
     */
    while ((*sector_ptr & JEDEC_POLL_DQ7_MASK) == 0U) {
        /* Check if DQ5 error flag is asserted (internal erase timeout) */
        if ((*sector_ptr & JEDEC_ERROR_DQ5_MASK) != 0U) {
            /* Re-check DQ7 to avoid false error on boundary completion */
            if ((*sector_ptr & JEDEC_POLL_DQ7_MASK) == 0U) {
                *sector_ptr = JEDEC_CMD_READ_RESET;
                return false; /* Hardware erase error */
            }
            break;
        }
    }

    /* Restore flash to normal array read mode */
    *sector_ptr = JEDEC_CMD_READ_RESET;
    return true;
}

/* =========================================================================
 * RAM Loader Entry Point
 * ========================================================================= */

/**
 * @brief Execution entry point invoked from SRAM by the DWire32A host.
 *
 * Sequentially erases the 8 micro-sectors of the EEMUL partition (0x7F0000 - 0x7FFFFF)
 * and triggers a Program Interrupt (pi) breakpoint instruction to return control.
 */
void __attribute__((section(".text.entry"), noreturn)) _start(void) {
    /* Erase all 8 corrupted NVRAM micro-sectors (SA134 to SA141) */
    for (uint32_t offset = SPANSION_NVRAM_BASE_ADDR;
         offset < SPANSION_NVRAM_END_ADDR;
         offset += SPANSION_MICROSECTOR_STRIDE) {
        (void)spansion_erase_sector(offset);
    }

    /*
     * Signal completion to the DWire32A host debugger.
     * "pi" is the native AM33/AM34 Program Interrupt / Software Breakpoint instruction.
     */
    __asm__ volatile ("pi");

    /* Lock core in infinite halt loop if resumed */
    while (1) {
        __asm__ volatile ("nop");
    }
}

```

---

## 7. Guia de Descompilação e Análise Estática do Firmware (8 MB)

Assim que o chip `IC55007` for lido na bancada (via gravador externo SVOD 4 ou interface DWire32A), o arquivo binário intacto de 8 MB (`8.388.608` bytes) revelará as estruturas internas do sistema operacional proprietário da Panasonic.

### Passo 1: Validação de Endianness e Verificação de *Byte-Swap*

Dependendo da configuração do soquete TSOP-48 e do gravador em palavras de 16 bits, o arquivo pode ser salvo em formato *Big-Endian* ou *Little-Endian*.

* Abra a imagem no editor hexadecimal (**HxD**) e pesquise por strings ASCII conhecidas: `PANASONIC`, `MATSUSHITA`, `DMR`, `MEIHFS`.
* Se o texto aparecer invertido a cada 2 bytes (ex.: `APANSNOIC` ou `AMSTSUHITA`), os pares de bytes de 16 bits estão trocados.
* Para inverter e normalizar a imagem para análise em descompiladores, execute o script auxiliar:

```python
with open('DMR_EH55_RAW.bin', 'rb') as f:
    data = f.read()
# Inverte cada palavra de 16 bits
swapped = bytearray(len(data))
swapped[0::2] = data[1::2]
swapped[1::2] = data[0::2]
with open('DMR_EH55_SWAPPED.bin', 'wb') as f:
    f.write(swapped)
```
*(Observação: a versão gravada fisicamente de volta na Flash deve respeitar a ordem nativa exigida pelo gravador do técnico).*

### Passo 2: Desassemblagem com `binutils` GNU Oficial (`mn10300-elf-objdump`)

A família MN10300 / AM34 é suportada nativamente nas árvores GNU:

```bash
mn10300-elf-objdump -D -b binary -m am33-2 --adjust-vma=0x80000000 DMR_EH55_ORIGINAL.bin > dmr_eh55_asm.txt
```

### Passo 3: Roteiro de Inspeção Imediata em `dmr_eh55_asm.txt`

1. **Vetor de Boot em `0x80000000`:**
   * Contém as primeiras instruções executadas pelo SoC.
   * Inicialização do registrador de barramento `SBBCR` / `SB0BASE` e configuração do Stack Pointer (`SP`), definindo a localização exata da **SRAM interna** para posicionamento do RAM Loader.
2. **Cálculo de Checksum da `EEMUL` em `0x807F0000`:**
   * A busca por referências ao endereço `0x807F0000` no código assembly isola a rotina exata de validação de integridade (CRC16/CRC32 ou soma de verificação) que a Panasonic aplica sobre os blocos de parâmetros de 8 KB.
3. **Mapeamento de Periféricos Seriais (`0xD4000000`):**
   * Configuração dos registradores `SC0CTR`, `SC1CTR` e `SC2CTR` (taxas de baud rate, paridade e buffers seriais de serviço utilizados pelo conector de 6 pinos).

---

## 8. Comparativo de Métodos de Recuperação na Bancada

| Critério | Método 1: Gravador Externo (SVOD 4 + TSOP-48) | Método 2: Injeção DWire32A (RP2040 In-Circuit) |
| :--- | :--- | :--- |
| **Maturidade Técnica** | **100% comprovado, pronto e imediato** | Engenharia reversa em desenvolvimento |
| **Intervenção Física** | Dessolda do chip `IC55007` com ar quente | Soldagem de 5 ou 6 fios finos no conector `CN55001` |
| **Velocidade de Execução** | ~3 a 5 minutos na bancada | Imediato após calibração do protocolo |
| **Risco de Hardware** | Mínimo com técnico experiente em SMD | Quase nulo (protegido por resistores em série) |
| **Ação no Firmware** | Script `clean_spansion_nvram.py` zera `0x7F0000`–`0x7FFFFF` | RAM Loader executa `spansion_erase_sector` nos 8 micro-setores |

Ambas as rotas levam ao mesmo resultado técnico exato: o restabelecimento do estado virgem da partição `EEMUL`, forçando o firmware Matsushita a reformatar seus parâmetros operacionais e eliminando em definitivo o travamento em `U99`.
