# Especificação Técnica: Interface `P7501` Async UART (`115200 8N1`) & Arquitetura AM34 / MN103S

Documento de especificação técnica para interfaceamento físico, captura de boot, sondagem diagnóstica e suporte à arquitetura de memória do gravador **Panasonic DMR-EH55** (SoC **Matsushita `AM34` / `MN103S`** + Syscon **`IC7501`**, placa **`VEP79138`**, conector **`P7501`** `[B6B-ZR-3R4]` `"TO JIGU"`).

> [!WARNING]
> **Errata Empírica Crítica — Panasonic DMR-EH55 (`P7501` vs. Legado `CN55001` / `DWire32A`):**
> A verificação exaustiva dos 3 manuais de serviço oficiais do **DMR-EH55** (`CHM0605028CE`, `DSD0603003CE` Vol.1/Vol.2) combinada com testes elétricos e capturas lógicas ao vivo via RP2040 (`COM10`: `PINCHECK`, `BUSDIAG`, `UARTPROBE`, `JIGBOOT`, `UARTMON`) **comprovou definitivamente que a porta `P7501` (`B6B-ZR-3R4`, `TO JIGU`) do DMR-EH55 NÃO é `DWire32A`, e sim uma interface dual-target UART Assíncrona `115200 8N1` + Controle de Boot/Reset**:
> - **Zero ocorrências nos manuais do EH55:** `MN2DS`, `MN2WS`, `CN55001`, `P55001`, `DWire` e `IC55007` não existem nos esquemáticos do DMR-EH55 (pertencem a outro chassi DIGA). No EH55, `VEP79138K` é a PCB Digital, `P59001` interliga Main PCB $\leftrightarrow$ Digital PCB (`UARTM2P`, `UARTP2M`, `XFWE`) e `P7501` fica na Main PCB conectado a `IC7501` / SoC Principal.
> - **Mapeamento Físico Verificado em `P7501` ($\leftrightarrow$ Pico RP2040):** Pino 1 **`SW_UART`** (`GP26/ADC0`, Hi-Z `3.24–3.25V`, Pull-Down `2.68–2.69V`), Pino 2 **`XRESET`** (`GP5`, subida `20 µs`), Pino 3 **`UARTM2P_T` / MAX RXD** (`GP4`, TX da sonda, bloqueado por `Q7514` se `SW_UART=LOW`), Pino 4 **`GND`**, Pino 5 **`NSW5V` (`X_SW_5R2V`, ~5,15V nominal — manter em `Hi-Z` em `GP3`!)**, Pino 6 **`UARTP2M_T` / MAX TXD** (`GP2`, RX da sonda, único pino com transições UART `115200 8N1`).
> - Consulte a **Seção 10** de [DOSSIE_ENGENHARIA_REVERSA_BOOTLOADER_E_DRIVE_MN103S.md](DOSSIE_ENGENHARIA_REVERSA_BOOTLOADER_E_DRIVE_MN103S.md) para a análise completa dos circuitos discretos (`IC1506`, `D7510`, `B7501`, `Q7510`, `Q7514`, `QR7503`, `C7519`, `K7503`).

---

## 1. Identificação Real da Interface: `P7501` Async UART (`115200 8N1`)

Ao contrário da hipótese inicial de literatura baseada em outros chassis Panasonic (`PanaX NEO` / `PX-ADPDWA` / `DWire32A`), o conector de serviço de 6 pinos **`P7501` (`JST B6B-ZR-3R4`, `TO JIGU`)** na Main PCB `VEP79138` do DMR-EH55 é um barramento **UART assíncrono a `115200 8N1`** acoplado a um estágio transistorizado discreto (`Q7510`, `Q7514`, `QR7503`) e ao circuito de alimentação comutada **`NSW5V` (`IC1506` via `D7510`)**:

* **Núcleo do Processador:** Matsushita **AM34 / MN103S** (arquitetura RISC de 32 bits, família `MN10300`), acoplado via cabo/interconexão **`P59001`** à PCB Digital `VEP79138K` e ao Syscon **`IC7501`**.
* **Sinais do Barramento Interplaca (`P59001`):** `UARTM2P`, `UARTP2M`, `XFWE`, `SCLK`, `SBPTM`, `SBMTP`, `XMPREQ`, `XINTM`, `XINTP`.

```text
       CONECTOR CANÔNICO DE SERVIÇO P7501 [B6B-ZR-3R4] ("TO JIGU" — 1.5mm Pitch)
       ┌───────────────────────────────────────────────────────────────────────┐
Pin 1  │ ● SW_UART   │ Boot / Serial Enable (Hi-Z ~3.25V; LOW -> XFWE / corte Q7514)
Pin 2  │ ● XRESET    │ Hardware Reset (Open-Drain ativo LOW via QR7503, ~20µs subida)
Pin 3  │ ● UARTM2P_T │ MAX RXD (TX Pico -> AM34 RX via Q7514; R7570 10kΩ onboard)
Pin 4  │ ● GND       │ Ground Comum (0.0Ω direto para o Chassi Metálico!)
Pin 5  │ ● NSW5V     │ X_SW_5R2V (IC1506 ~5.15V / D7510, ~3.29V repouso — MANTER Hi-Z!)
Pin 6  │ ● UARTP2M_T │ MAX TXD (RX Pico <- AM34 TX 115200 8N1; Pad Quadrado PCB)
       └───────────────────────────────────────────────────────────────────────┘
```

---

## 2. Projeto Elétrico da Interface de Hardware (RP2040 / RP2350 $\leftrightarrow$ `P7501`)

A interconexão entre o **Raspberry Pi Pico (RP2040 / RP2350)** e o conector `P7501` exige respeito estrito às características discretas de `VEP79138` (`R7570`/`R7572` `10 kΩ`, `R7562` `4.7 kΩ`, `R7559` `10 kΩ`, `R7528` `4.7 kΩ`):

| Pino Canônico `P7501` | Sinal no Esquemático | Papel no Circuito (`VEP79138`) | GPIO RP2040 / RP2350 | Condicionamento / Proteção | Função Elétrica e Comportamento Medido |
| :---: | :--- | :--- | :---: | :--- | :--- |
| **Pino 1** | `SW_UART` `(D)` | Coletor `Q7510`, Base `QR7503`, `IC7501` Pin 80 | **`GP26 / ADC0`** (Pino 31) | Opcional `1.0 kΩ` em série | `3.24–3.25V` em Hi-Z; `2.68–2.69V` sob pull-down. Nível `LOW` drena `XFWE` e corta `Q7514`. |
| **Pino 2** | `XRESET` | Linha de reset `QR7503` | **`GP5`** (Pino 7) | **Modo Open-Drain** (op. `100 Ω` série) | Ativo em `0V`; libera para `Hi-Z` com subida em `~20 µs`. Nunca dirigir `+3.3V` push-pull! |
| **Pino 3** | `UARTM2P_T` (`MAX RXD`) | Via `Q7514` p/ `UARTM2P`, `R7570 10 kΩ` | **`GP4`** (Pino 6, UART TX) | Sem pull-up externo (op. `100 Ω` série) | TX da sonda (`115200 8N1`) para RX da placa. Exige `SW_UART = Hi-Z` para `Q7514` conduzir. |
| **Pino 4** | **`GND`** | `GND_DIGITAL` | **`GND`** (Pino 38) | Ligação direta de baixa impedância | **Referência comum de `0V` ($0.0\,\Omega$ direto ao chassi metálico).** |
| **Pino 5** | **`NSW5V` (`X_SW_5R2V`)** | Alimentação `IC1506` (`~5.15V`) via `D7510` | **`GP3`** (Pino 5) | **ESTRITAMENTE `Hi-Z Input` sem pulls** | Barramento estático de `~3.29V` em repouso (`~5.15V` upstream). **NUNCA dirigir push-pull!** |
| **Pino 6** | `UARTP2M_T` (`MAX TXD`) | Saída via `R7535` / `R7572 10 kΩ` | **`GP2`** (Pino 4, UART RX) | Opcional `100 Ω` série | Único pino UART TX ativo da placa (`<00>DDR CHECK: OK`, `CD diag boot mode`). Pad quadrado PCB. |

### Regras Críticas de Montagem Elétrica

1. **Guarda Absoluta em `NSW5V` (`GP3` / Pino 5):**
   * `IC1506` (`SI-3011ZFE`) gera `~5.15V` que chegam via `D7510` a `R7570`/`R7572` e ao `VDD` de `IC7501`. Por isso, `GP3` do RP2040/RP2350 deve permanecer bloqueado em `GPIO_IN` com `gpio_disable_pulls(3)` durante todo o ciclo de execução. **Nunca** alimente o Pico por `P7501` e **nunca** aplique nível lógico alto ou baixo em `GP3`.
2. **Prevenção de Curto em `XRESET` (`GP5` Open-Drain):**
   * O GPIO `GP5` opera exclusivamente alternando a direção do pino: `0V` (saída baixa) para forçar reset e `GPIO_IN` sem pulls (`Hi-Z`) para soltar o barramento.
3. **Acoplamento `SW_UART` $\leftrightarrow$ `Q7514` (`UARTM2P_T`):**
   * Quando `SW_UART = LOW` (`0V`), `Q7510` drena `XFWE` para nível baixo (forçando modo serial/JIG), mas simultaneamente coloca `Q7514` em corte (via `R7528 4.7 kΩ`), bloqueando fisicamente a passagem do pino 3 (`UARTM2P_T`) até o `UARTM2P` do SoC. Para que o sinal de TX em `GP4` alcance a placa, `SW_UART` (`GP26`) precisa estar liberado em **`Hi-Z`**.
4. **Sem Pull-Up Externo de `4.7 kΩ` em `UARTM2P_T`:**
   * A placa `VEP79138` já incorpora os resistores `R7570` (`10 kΩ`) e `R7572` (`10 kΩ`). Não solde pull-up adicional de `4.7 kΩ` entre `P7501` e `3V3_OUT` do Pico.
5. **Desbloqueio Físico do Pino 14 (`WP#/ACC`) da Spansion (`IC5001`) para Gravação In-Circuit:**
   * Na Spansion `S29JL064H70TFI00` (`IC5001`, Top Boot), se o pino 14 (`WP#/ACC`) estiver em `0V`, os dois micro-setores finais (`SA140` e `SA141`, faixa `0x7FC000`–`0x7FFFFF`) ficam protegidos por hardware. Verifique a tensão com multímetro antes de qualquer apagamento in-circuit em Flash (`3.3V` = liberado).

---

## 3. Máquina de Estados PIO (RP2040 / RP2350) para RX Assíncrono `115200 8N1`

O subsistema **PIO (`PIO0`)** realiza amostragem determinística dos bytes enviados por `UARTP2M_T` (`GP2`, `P7501` Pino 6) a **`115200 8N1`** (oversampling 8x a `921.600 Hz`) sem carregar interrupções da CPU ARM e sem jamais tocar no pino `GP3` (`NSW5V`).

### Código PIO Assembly Verificado ([`firmware/pico-p7501/p7501_uart.pio`](../../firmware/pico-p7501/p7501_uart.pio))

```pio
.program p7501_uart_rx

; =========================================================================
; Panasonic P7501 Asynchronous 115200 8N1 UART Receiver (Passive Input Only)
; Oversampling: 8 SM cycles per bit (921,600 Hz @ 115200 baud)
;
; Pin mapping:
;   IN / JMP pin: GP2 (P7501 Pin 6 UARTP2M_T, MAX TXD -> Pico RX)
;   GP3 (P7501 Pin 5 NSW5V) is NEVER touched by PIO and stays High-Z Input.
; =========================================================================

.wrap_target
start:
    wait 0 pin 0            ; Wait for falling edge of Start bit (GP2 == 0)
    set x, 7        [10]    ; 8 data bits; wait 1.5 bit periods (12 cycles total to bit 0 center)
bitloop:
    in pins, 1              ; Shift LSB-first data bit into ISR
    jmp x--, bitloop [6]    ; Loop every 8 cycles (1 + 7 cycles)
    jmp pin, good_stop      ; Verify Stop bit is HIGH
    irq 4 rel               ; Framing error signaled on IRQ 4
    wait 1 pin 0            ; Wait for idle HIGH recovery
    jmp start
good_stop:
    push block              ; Deliver MSB-aligned byte in upper 8 bits of RX FIFO word
.wrap
```

---

## 4. Temporização Discreta de Boot (`SW_UART` + `XRESET`) & Primitivas `P7501`

Ao executar `JIGBOOT` ou perfis discretos de inicialização via [`p7501_uart_bitbang.h`](../../firmware/pico-p7501/uart/p7501_uart_bitbang.h):

1. **Pulso Open-Drain em `XRESET` (`GP5`):** `p7501_hw_reset_assert()` força `0V` por `15 ms`, e `p7501_hw_reset_release()` devolve `GP5` a `Hi-Z` (~`20 µs` de subida via `QR7503`).
2. **Amostragem de Boot & Captura UART (`115200 8N1`):** Em `JIGBOOT` (`SW_UART = LOW` durante a subida do reset), o bootloader imprime `<00>DDR CHECK: OK` seguido de `CD diag boot mode` em `GP2` (`UARTP2M_T`), exibindo a telemetria de sense do mecanismo óptico (`X(80020401)59...` $\rightarrow$ `X(80040904)` $\rightarrow$ `Drive start failed[-2147219196]`).
3. **Transmissão Interativa / Sondagem de Bootrom (`UARTSEND` / `FMUSCAN`):** Para enviar caracteres a `115200 8N1` pela linha `GP4` (`UARTM2P_T`), execute `ENGAGE` com `SW_UART` em `Hi-Z` de modo que `Q7514` permaneça polarizado e encaminhe os bits até o SoC.
4. **Isolamento Seguro (`ISOLATE` / `HIZ`):** Desacopla transmissores ao término da operação, restaurando `GP26` (`ADC0`) via `p7501_hw_restore_sw_uart_adc0()` e mantendo `GP3` (`NSW5V`) sempre `Hi-Z Input`.

---

## 5. Mapa de Memória Físico da Arquitetura AM34 (`MN103S`)

Conforme a especificação técnica da arquitetura **Matsushita AM33v2 / AM34** (e o mapeamento Linux `arch/mn10300` / GNU Binutils), o espaço de endereçamento linear de 32 bits divide-se em regiões cacheadas e acessos diretos não cacheados:

| Faixa de Endereço (Hex) | Região da Arquitetura AM34 | Dispositivo Mapeado na Placa `VEP79138` |
| :--- | :--- | :--- |
| `0x00000000` – `0x007FFFFF` | Espaço de Boot (Com Cache) | Espelho da NOR Flash Spansion `S29JL064H` (`IC5001`, 8 MB) |
| **`0x80000000` – `0x807FFFFF`** | **Espaço Físico Direto (Sem Cache)** | **Acesso Físico Direto à Spansion `IC5001` (`CS0` / `SB0`)** |
| `0x8C000000` / `0x90000000` | Espaço Físico SDRAM (Sem Cache) | Memórias SDRAM principais (buffers de sistema e runtime) |
| `0xC0000000` – `0xDFFFFFFF` | Periféricos Internos SoC | Controladores Seriais (`SC0`–`SC2` em `0xD4002000`), Timers, GPIO, ATA/IDE |
| `0xFF000000` – `0xFFFFFFFF` | Registradores de Núcleo | Registradores de controle do núcleo e interrupções |

> [!CRITICAL]
> **Regra de Ouro para Gravação JEDEC via CPU:**
> Todo comando JEDEC emitido pelo *RAM Loader* para apagar ou programar a memória Flash Spansion **deve obrigatoriamente utilizar a faixa de endereço físico sem cache (`0x80000000` + offset)** e jamais o espelho `0x00000000`.
> Se um comando de escrita for emitido em `0x00000000` com o cache L1 de dados ativo, a CPU armazenará os ciclos de desbloqueio `0xAA` / `0x55` nas linhas de cache internas, e os pulsos de barramento jamais alcançarão os pinos físicos da memória Flash.

---

## 6. Código do "RAM Loader" (Driver JEDEC Nativo AM34 para Spansion `S29JL064H`)

Para cenários onde há execução de payload em SRAM interna (`0x00004000`), o firmware embarca o micro-binário independente de posição (< 200 bytes) que executa o algoritmo JEDEC em barramento de 16 bits (`Word Mode`) para apagar os 8 micro-setores finais de 8 KB (`SA134` a `SA141`, faixa `0x7F0000` a `0x7FFFFF`).

### Cálculo dos Endereços de Desbloqueio em 16 Bits (`Word Mode`)

No modo de palavra de 16 bits (`BYTE# = High`), os endereços JEDEC de 16 bits (`0x555` e `0x2AA`) são transladados pelo barramento de endereços de bytes da CPU através de um deslocamento à esquerda de 1 bit:
* `Unlock Addr 1`: `0x555 << 1` = **`0xAAA`** (Dado: `0x00AA`)
* `Unlock Addr 2`: `0x2AA << 1` = **`0x554`** (Dado: `0x0055`)
* **Endereço Base Uncached:** `0x80000000`
* **Alvos de Apagamento (8 micro-setores de 8 KB da `EEMUL`):**
  * `0x807F0000` (`SA134`) a `0x807FE000` (`SA141`)

### Código C Nativo Independente de Posição ([`firmware/pico-p7501/flash/ram_loader_am34_target.c`](../../firmware/pico-p7501/flash/ram_loader_am34_target.c))

```c
/**
 * @file ram_loader_am34_target.c
 * @brief Autonomous AM34 / MN103S position-independent SRAM payload.
 *
 * Injected into internal SRAM via P7501 / AM34 monitor transport
 * to erase the corrupted NVRAM emulation partition (EEMUL) on the Spansion
 * S29JL064H70TFI00 NOR Flash, clearing the DMR-EH55 U99 fatal state.
 *
 * Target Processor: Matsushita UniPhier / AM34 / MN103S
 * Target Hardware:  Panasonic DMR-EH55 Digital Board (VEP79138K)
 *
 * Compilation:
 *   mn10300-elf-gcc -mam33-2 -std=c11 -Os -fPIC -nostdlib -c ram_loader.c -o ram_loader.o
 *   mn10300-elf-objcopy -O binary -j .text.entry -j .text ram_loader.o ram_loader.bin
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include <stdint.h>
#include <stdbool.h>

#define FLASH_UNCACHED_BASE         0x80000000UL
#define JEDEC_UNLOCK_ADDR1_OFFSET   0x0AAAUL
#define JEDEC_UNLOCK_ADDR2_OFFSET   0x0554UL

#define JEDEC_CMD_AA                0x00AAU
#define JEDEC_CMD_55                0x0055U
#define JEDEC_CMD_ERASE_SETUP       0x0080U
#define JEDEC_CMD_SECTOR_ERASE      0x0030U
#define JEDEC_CMD_READ_RESET        0x00F0U

#define JEDEC_POLL_DQ7_MASK         0x0080U
#define JEDEC_ERROR_DQ5_MASK        0x0020U

#define SPANSION_NVRAM_BASE_ADDR    0x007F0000UL
#define SPANSION_NVRAM_END_ADDR     0x00800000UL
#define SPANSION_MICROSECTOR_STRIDE 0x00002000UL

#define FLASH_REG16(addr) (*(volatile uint16_t *)((uintptr_t)(addr)))
#define JEDEC_REG_UNLOCK1 FLASH_REG16(FLASH_UNCACHED_BASE + JEDEC_UNLOCK_ADDR1_OFFSET)
#define JEDEC_REG_UNLOCK2 FLASH_REG16(FLASH_UNCACHED_BASE + JEDEC_UNLOCK_ADDR2_OFFSET)

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

    while ((*sector_ptr & JEDEC_POLL_DQ7_MASK) == 0U) {
        if ((*sector_ptr & JEDEC_ERROR_DQ5_MASK) != 0U) {
            if ((*sector_ptr & JEDEC_POLL_DQ7_MASK) == 0U) {
                *sector_ptr = JEDEC_CMD_READ_RESET;
                return false;
            }
            break;
        }
    }

    *sector_ptr = JEDEC_CMD_READ_RESET;
    return true;
}

void __attribute__((section(".text.entry"), noreturn)) _start(void) {
    for (uint32_t offset = SPANSION_NVRAM_BASE_ADDR;
         offset < SPANSION_NVRAM_END_ADDR;
         offset += SPANSION_MICROSECTOR_STRIDE) {
        (void)spansion_erase_sector(offset);
    }

    __asm__ volatile ("pi");

    while (1) {
        __asm__ volatile ("nop");
    }
}
```

---

## 7. Guia de Descompilação e Análise Estática do Firmware (8 MB)

Quando a memória Spansion `S29JL064H` (**`IC5001`**) é lida na bancada (ex.: via programador externo SVOD 4 / XGecu T48), o arquivo binário intacto de 8 MB (`8.388.608` bytes) expõe as estruturas internas do sistema operacional proprietário da Panasonic.

### Passo 1: Validação de Endianness e Verificação de *Byte-Swap*

Dependendo da configuração do soquete TSOP-48 e do gravador em palavras de 16 bits, o arquivo pode ser salvo em formato *Big-Endian* ou *Little-Endian*.

* Abra a imagem no editor hexadecimal (**HxD**) e pesquise por strings ASCII conhecidas: `PANASONIC`, `MATSUSHITA`, `DMR`, `MEIHFS`.
* Se o texto aparecer invertido a cada 2 bytes (ex.: `APANSNOIC` ou `AMSTSUHITA`), os pares de bytes de 16 bits estão trocados.
* Para inverter e normalizar a imagem para análise em descompiladores:

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
*(Observação: a versão gravada fisicamente de volta na Flash deve respeitar a ordem nativa exigida pelo gravador).*

### Passo 2: Desassemblagem com `binutils` GNU Oficial (`mn10300-elf-objdump`)

```bash
mn10300-elf-objdump -D -b binary -m am33-2 --adjust-vma=0x80000000 DMR_EH55_ORIGINAL.bin > dmr_eh55_asm.txt
```

### Passo 3: Roteiro de Inspeção Imediata em `dmr_eh55_asm.txt`

1. **Vetor de Boot em `0x80000000`:** Contém as primeiras instruções executadas pelo SoC e configuração dos registradores de barramento e `SP`.
2. **Cálculo de Checksum da `EEMUL` em `0x807F0000`:** Referências ao intervalo `0x807F0000..0x807FFFFF` isolam a verificação de integridade dos micro-setores NVRAM.
3. **Mapeamento de Periféricos Seriais (`0xD4002000`):** Rotinas de UART de boot e diagnóstico acopladas à porta **`P7501`**.

---

## 8. Comparativo de Métodos de Recuperação e Diagnóstico na Bancada

| Critério | Método 1: Gravador Externo (SVOD 4 / XGecu T48 na Flash `IC5001`) | Método 2: Sonda `P7501` Async UART (RP2040 / RP2350) |
| :--- | :--- | :--- |
| **Maturidade Técnica** | **100% comprovado, imediato e determinístico para unbrick U99** | **Validado ao vivo em bancada (`COM10`, `115200 8N1`) para telemetria/diagnóstico** |
| **Intervenção Física** | Dessolda do chip `IC5001` (TSOP-48) com ar quente | Conexão não-destrutiva do conector 6 vias **`P7501`** (`B6B-ZR-3R4`) |
| **Escopo Operacional** | Dump bruto integral de 8 MB e gravação limpa da partição `EEMUL` | Captura de boot (`<00>DDR CHECK: OK`, `CD diag`), diagnósticos FMU/Sense e ponte GDB RSP |
| **Risco de Hardware** | Mínimo com técnico experiente em retrabalho SMD | Mínimo quando mantido **`NSW5V` (`GP3`) em `Hi-Z Input`** e `XRESET` (`GP5`) Open-Drain |
| **Ferramenta de Software** | CLI nativa `panadiga nvram clean` zera `0x7F0000`–`0x7FFFFF` | Firmware `panasonic_p7501_uart` + `panadiga` CLI / Tauri UI |

Para reparo físico definitivo de `U99` em unidades onde o monitor serial não expõe acesso de escrita à Flash em `P7501`, a leitura externa do CI **`IC5001`** combinada com **`panadiga nvram clean`** restabelece os 8 micro-setores virgens (`0xFF`), eliminando a falha em definitivo.
