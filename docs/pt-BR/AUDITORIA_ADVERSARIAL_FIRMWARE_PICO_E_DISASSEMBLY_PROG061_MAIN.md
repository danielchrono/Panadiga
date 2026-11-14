# Auditoria Adversarial do Firmware Pico (`P7501`), Correção de LED e Disassembly `PROG061` / `MAIN_3410PQ`

> **Escopo & Veracidade Empírica:** Este dossiê consolida: (1) a refatoração modular (`SRP / SoC / SSOT / DRY / FP`) dos módulos C11 restantes em [`firmware/pico-p7501/`](../../firmware/pico-p7501/), (2) a auditoria adversarial do firmware e do parser desktop/Rust contra medições reais de bancada na porta **`COM10` (`Panasonic DMR-EH55` via `P7501`)**, (3) a causa raiz e solução definitiva do controle do **LED onboard** (`RP2040`/`RP2350`/`CYW43`), e (4) as descobertas mais recentes de disassembly MN10300/AM34 em **`eh55_frm_PROG061_BOOT.bin`** e **`EH55_MAIN_3410PQ.bin`** obtidas via PyGhidra (`Panadiga_MN103S`).

---

## 1. Decomposição Modular em C11 Estrito (`SRP / SoC / SSOT / DRY`)

Os monólitos remanescentes em [`firmware/pico-p7501/`](../../firmware/pico-p7501/) foram decompostos em módulos coesos, com **todos os arquivos de produção e cabeçalhos com $\le 257$ linhas** (máximos atuais: `257` linhas em `cli/p7501_cli.c` e `246` linhas em `uart/p7501_uart_pio_engine.c`), compilados em ISO C11 com `-Wall -Wextra -Werror` em ARM/RISC-V e `/std:c11 /W4 /WX` no host Windows:

| Subsistema | Arquivos Criados / Refatorados | Responsabilidade Única (SRP / SoC) |
| :--- | :--- | :--- |
| **`common/`** | [`p7501_hex.h`](../../firmware/pico-p7501/common/p7501_hex.h), [`p7501_hex.c`](../../firmware/pico-p7501/common/p7501_hex.c) | **SSOT DRY** de decodificação hex (`p7501_decode_hex_bytes`, suportando espaços, `:` e `,`), comparação case-insensitive (`p7501_ascii_ieq`), cópia em maiúsculas (`p7501_ascii_upper_copy`) e tokenização (`p7501_next_token`). Eliminou parsers duplicados em `cli/`, `uart/p7501_fmu_probe.c` e `gdb/`. |
| **`hal/`** | [`p7501_led_logic.h`](../../firmware/pico-p7501/hal/p7501_led_logic.h), [`p7501_led_logic.c`](../../firmware/pico-p7501/hal/p7501_led_logic.c), [`p7501_led.h`](../../firmware/pico-p7501/hal/p7501_led.h), [`p7501_led.c`](../../firmware/pico-p7501/hal/p7501_led.c) | Separação **Pure FP Logic (`p7501_led_logic.c`, testável no host)** ↔ **HAL IO/IRQ (`p7501_led.c`)** com suporte unificado a `GP25` (`RP2040`/`RP2350`) e `CYW43_WL_GPIO_LED_PIN` (`Pico W`/`Pico 2 W`). |
| **`diag/`** | [`p7501_diag_electrical.c`](../../firmware/pico-p7501/diag/p7501_diag_electrical.c), [`p7501_diag_busdiag.c`](../../firmware/pico-p7501/diag/p7501_diag_busdiag.c), [`p7501_diag_pwrtrace.c`](../../firmware/pico-p7501/diag/p7501_diag_pwrtrace.c) | Matriz elétrica 6 pinos (`PINCHECK`), inspeção de barramento (`BUSDIAG`) e logger temporal ADC0 + bordas (`PWRTRACE`). |
| **`flash/`** | [`p7501_flash_cli.h`](../../firmware/pico-p7501/flash/p7501_flash_cli.h), [`p7501_flash_cli.c`](../../firmware/pico-p7501/flash/p7501_flash_cli.c), [`p7501_ram_loader.c`](../../firmware/pico-p7501/flash/p7501_ram_loader.c) | Comandos CLI Flash/EEMUL (`AUTODETECT`, `CFI`, `FLASH_SIZE`, `INJECT_LOADER`, `ERASE_NVRAM`) desacoplados de `p7501_cli.c`. |
| **`uart/`** | [`p7501_uart_pio_engine.c`](../../firmware/pico-p7501/uart/p7501_uart_pio_engine.c), [`p7501_uart_baud.c`](../../firmware/pico-p7501/uart/p7501_uart_baud.c), [`p7501_uart_stream.c`](../../firmware/pico-p7501/uart/p7501_uart_stream.c), [`p7501_uart_tx.c`](../../firmware/pico-p7501/uart/p7501_uart_tx.c), [`p7501_fmu_batch.c`](../../firmware/pico-p7501/uart/p7501_fmu_batch.c) | Motor `PIO1` (`SM0` RX / `SM1` TX em `p7501_uart_duplex.pio`), autobaud (`BAUD`/`BAUDSCAN`), streaming sem perdas (`UARTSTREAM`/`HEXCAP`), gatilho temporizado (`WAITTX`), ponte transparente (`BRIDGE`) e varredura FMU (`FMUBATCH`). |
| **`cli/` & `tests/`** | [`p7501_cli.c`](../../firmware/pico-p7501/cli/p7501_cli.c), [`p7501_cli_help.c`](../../firmware/pico-p7501/cli/p7501_cli_help.c), [`test_uart_codec.c`](../../firmware/pico-p7501/tests/test_uart_codec.c) | Roteador fino de comandos (`257` linhas), banner/help e testes unitários em host para codec, args `JIG:<ms>`, hex e FSM de LED. |

---

## 2. Causa Raiz do Bug no LED & Solução Determinística (`p7501_led.c`)

Na revisão anterior, havia a impressão de que *"a própria placa definia suas sequências de LED"*. A auditoria do código e o teste em hardware revelaram **três defeitos combinados**:

1. **Sobrescrita silenciosa pelo polling de telemetria (`STATUS`):** Comandos rotineiros de UI/host (ou o laço principal) alteravam `p7501_led_set_mode(...)` em eventos internos, destruindo o modo selecionado pelo usuário (`LED GDB`, `LED FAULT`, etc.).
2. **Congelamento durante capturas síncronas:** Os loops de captura UART (`115200 bps` / `3000–8000 ms`) monopolizavam o núcleo único executando `p7501_led_tick()` apenas fora das capturas.
3. **Ausência de aplicação imediata no pino e de controle manual persistente:** Em mudanças de estado, o pino físico só refletia o novo padrão no tick seguinte, e não havia `LED OFF` nem `LED AUTO`.

### Solução Implementada e Validada em `COM10`

* **FSM Pura + `MANUAL_LOCK` (`p7501_led_logic.c`):**
  * `p7501_led_state_set_manual(&s_led_state, mode)` trava o estado em `lock = MANUAL`, impedindo que chamadas internas `p7501_led_set_auto_mode(...)` alterem o LED até que o operador execute `LED AUTO` (`p7501_led_unlock_auto`).
  * Padrões de `10 Hz` diferenciados: `OFF`, `HEARTBEAT_IDLE`, `BUS_ACTIVE`, `GDB_RSP` e `FAULT`.
* **Temporizador Hardware IRQ-Safe (`p7501_led.c`):**
  * Em placas padrão (`RP2040`/`RP2350`, `GP25`), `p7501_led_init()` registra `add_repeating_timer_ms(-100, led_timer_callback, NULL, &s_led_timer)` protegido por `save_and_disable_interrupts()` / `restore_interrupts()`. O LED continua piscando regularmente mesmo dentro de capturas `UARTSTREAM`/`HEXCAP`/`WAITTX`.
  * Em placas com Wi-Fi/BLE (`PANADIGA_BLE_ENABLED=1`), evita chamar API SPI do `CYW43` dentro de contexto de IRQ, usando `p7501_led_tick()` cooperativo e `p7501_led_sleep_ms()`.

---

## 3. Auditoria Adversarial Firmware ↔ Bancada (`COM10`) & Parser Desktop/Rust

### 3.1 Achados Críticos Corrigidos Durante a Auditoria

| Severidade | Origem | Defeito Encontrado em Teste Real / Revisão | Correção Implementada & Evidência |
| :---: | :--- | :--- | :--- |
| **CRÍTICO** | `uart/p7501_uart_stream.c` (`HEXCAP`) | `HEXCAP 3000 NORMAL` descartava bytes no `COM10` (`"Chip se"`, `"uncoress"`, `overflow=YES`) porque `flush_hexcap_row()` chamava `fflush(stdout)` (USB CDC bloqueante) a cada 16 bytes, estourando a FIFO de 8 palavras da `PIO1` a `115200 bps` (`86.8 µs/byte`). | Amostras `(byte, gap_us)` agora são bufferizadas em RAM estática (`HEXCAP_RAM_BUF_CAP = 2048`) durante toda a janela RX e impressas somente após `p7501_pio_end_session()`. Captura ao vivo subsequente: **`348 bytes, framing_err=0, overflow=NO`** (`+88 µs`/byte contínuos). |
| **CRÍTICO** | `flash/p7501_flash_cli.c` & `gdb/gdb_rsp_cpu.c` | Risco de operações aparentemente bem-sucedidas com alvo ausente / sem monitor síncrono (`DCSR=0xFFFFFFFF`). | Enforce **fail-loud**: `AUTODETECT`, `INJECT_LOADER`, `ERASE_NVRAM` e comandos `GDB` (`G`, `c`, `s`, `?`) detectam barramento aberto `P7501_BUS_RESP_OPEN_HIGH` e abortam explícitamente (`[AUTODETECT] ERROR: No synchronous monitor online (DCSR=0xFFFFFFFF).`). |
| **ALTO** | `diag/p7501_diag_pwrtrace.c` | Uso de macros obsoletas (`P7501_PIN_SW_UART`, `P7501_ADC_MAX_COUNT`) impedia compilação no SDK ARM após a modularização. | Migrado para as constantes SSOT de `p7501_types.h`: `P7501_PIN_SW_UART_ADC`, `P7501_ADC_VREF`, `P7501_ADC_RESOLUTION`, `P7501_ADC_CH_VDD`. |
| **ALTO** | `flash/p7501_ram_loader.c` | Redefinição de `SPANSION_MICROSECTOR_SIZE_BYTES` em conflito com `ram_loader_payload.h`. | Removida definição local duplicada (SSOT único em `ram_loader_payload.h`). |
| **ALTO** | `crates/panadiga-core`, `diagnostics.rs`, `diagnosticParser.ts` | Logs legítimos da Bootrom Panasonic (`BGO driver init failed[1].`, `SD init failed[-90002].`, `ERROR -91006`) faziam o parser classificar comandos bem-sucedidos (`WAITTX: OK`) como `failed`. | O classificador agora verifica erros fatais do próprio Pico (`<cmd>: error`, `[<cmd>] error:`, `[safety interlock]`, `[handshake] failed`, `unknown command:`) e dá precedência à linha final `<CMD>: OK`; timeouts escalam até `125_000 ms` e `spec.rs` / `protocol.py` / mocks expõem todos os novos comandos `PIO1`. |
| **MÉDIO** | Circuito `Q7514` (`SW_UART` vs `TXD`) | Segurar `GP26` em `LOW` (`XFWE`) continuamente bloqueia `UARTM2P_T` (`GP4` via `Q7514`) durante `wait 1`. | Argumento `JIG:<ms>` (ex.: **`WAITTX 8000 JIG:350 "wait 1" 0D 4000`**): mantém `SW_UART=LOW` por `350 ms` para entrar em `serial boot mode` e solta `GP26` para `ADC0 Hi-Z` antes de disparar o payload em `wait 1`. |

### 3.2 Verificação Empírica em Hardware Vivo (`COM10`, RP2040 Unified UF2)

Firmware universal gravado em `D:\` via `BOOTSEL` (`SHA-256: 4ed6e74c781402c6aa400e360ecc38f351c5aa6af6306a4a52109ceba87a7793`), conectado ao **DMR-EH55** via chicote `P7501`:

* **`STATUS`:** `VDD: 3.25V (OK)`.
* **`LED GDB` -> 3x `STATUS` (espaçados de `0.75 s`) -> `LED STATUS` -> `LED OFF` -> `LED AUTO`:**
  * Manteve `mode=GDB_RSP | lock=MANUAL` imune aos polls `STATUS`, com alternância real `pin=1` → `pin=0` pelo timer IRQ.
* **`AUTODETECT`:** `[AUTODETECT] ERROR: No synchronous monitor online (DCSR=0xFFFFFFFF). Flash geometry unknown.` (fail-loud confirmado).
* **`PINCHECK`:** `GP26=3.24V (Hi-Z) / 2.70V (PD) / 3.26V (PU)`, `XRESET Release Rise Time (Hi-Z): 10 us`, `Post-Reset 250ms Transitions: GP2=872, GP3=0, GP4=0, GP5=0`, **`PINCHECK: OK`**.
* **`BAUDSCAN 3500 NORMAL`:** `edges=2124 min_low_us=8 max_low_us=54 raw_bps=125000 snapped_baud=115200`, **`BAUDSCAN: OK`**.
* **`PWRTRACE 3000 JIG:350`:** `GP26_SWUART: min=3.21V max=3.30V | Edges: RXD=2124 NSW5V=0 TXD=0 (final NSW5V=1)`, **`PWRTRACE: OK`**.
* **`UARTSTREAM 3500 NORMAL` & `HEXCAP 3000 NORMAL`:** `348 bytes, framing_err=0, overflow=NO`, capturando `DDR CHECK: OK`, `uncompress dst=90202000 len=160000 src=84080040 len=acc01`, `HEADER:5627DD54, DECOMP:5627DD54`, `Loader ver0.61 [06/03/06]`, `BGO driver init failed[1].`, `Main hung-up time changed 255[s].` (`+744207 us` para descompressão BGO, `+88 us/byte` no burst UART).
* **`WAITTX 8000 JIG:350 "wait 1" 0D 4000` e `1B0D`:** ambos dispararam no prompt `wait 1` após `!!! start fmu to read & exec fvu. !!!`, prosseguindo para `*** timer correspond initialize ***`, `startup reason = [70]`, `SD init start...`, `[SDCD]SetBusWidth:SD Status ERROR -91006` (rejeição de SDHC no leitor legado, exigindo `SDSC <= 2 GB` ou `CD-R` / `DIGADIAG.FRM`) e `*** CD diag boot mode ***` (**`WAITTX: OK (triggered=YES, rx=715 bytes)`**).
* **`ISOLATE`:** `[BUS] P7501 pins placed in High-Impedance Isolation (NSW5V/XRESET Hi-Z, GP26 ADC0).`

---

## 4. Comparativo Técnico: C11 + PIO vs. C++ vs. MicroPython

A opção do projeto por **ISO C11 estrito + PIO (`p7501_uart_duplex.pio`)** é validada diretamente pelas restrições elétricas e temporais medidas na bancada:

1. **Margem Temporal Crítica (`~8 µs` – `88 µs`):** A `115200 bps` cada bit dura `8.68 µs` (`min_low_us=8` observado no `BAUDSCAN`) e cada byte chega a cada `88 µs`. Conforme provado pelo bug do `HEXCAP`, até um `fflush()` USB CDC em C foi suficiente para estourar a FIFO de 8 palavras da `PIO1`. Um runtime interpretado com coletor de lixo (MicroPython) sofre pausas de milissegundos em alocações/strings que inviabilizariam gatilhos sub-milissegundo (`WAITTX`) e capturas sem perda.
2. **Determinismo de Segurança Elétrica (`NSW5V` / `XRESET` / `Q7514`):** `GP3` (`NSW5V`) nunca pode sair de `Hi-Z Input` e `GP5` (`XRESET`) deve operar estritamente em dreno aberto (`0V` / `Hi-Z`, subida medida em `10 µs`). Em C11 bare-metal não há exceções, RTTI ou construtores dinâmicos interferindo na sequência de reset.
3. **C++ vs. C11:** C++20 poderia oferecer `constexpr` e namespaces sem custo se compilado com `-fno-exceptions -fno-rtti`, mas sem ganho de vazão sobre C11 + PIO; C11 puro já garante FFI limpa caso blocos sejam compartilhados no host (`test_uart_codec.c` / `test_gdb_rsp_host.c`) e mantém o binário enxuto.

---

## 5. Novas Descobertas de Disassembly MN10300 (`PROG061` & `MAIN_3410PQ`)

Análise conduzida via scripts PyGhidra sobre o projeto `Panadiga_MN103S` (`C:\ghidra\projects\Panadiga_MN103S.gpr`):

### 5.1 `/eh55_frm_PROG061_BOOT.bin` (`0x50400000..0x5043FFFF`)

| Endereço / Símbolo | Tamanho | Função Identificada & Comportamento Decompilado |
| :--- | :---: | :--- |
| **`FUN_50404ec0`** (`0x50404EC0`) | `148 B` | Editor de linha UART (`gets`): usa `func_0x50404ca5` (`getc`) e `func_0x50404d15` (`putc`); trata `\r` enviando `\r\n` + terminador NUL; trata `\b` (`0x08`) apagando visualmente com `0x08 0x20 0x08`. |
| **`FUN_50404f12`** (`0x50404F12`) & **`FUN_50404f3a`** (`0x50404F3A`) | `32 B` / `88 B` | Rotinas de pausa interativa do monitor FVU: imprimem **`"---- Hit Any Key ----"`** (`0x50432DD0`) e **`"---- Hit 'Q' to Escape / Any Other Key to Continue ----"`** (`0x50432E20`). `FUN_50404f3a` lê um caractere, converte minúscula para maiúscula via tabela de atributos em `0x50438461` (`& 2` → `- 0x20`) e retorna `ch != 'Q'` (`0x51`). |
| **`FUN_5040ea30`** (`0x5040EA30`) | `75 B` | Validador `list_sum` do contêiner FMU: soma byte a byte os primeiros `0x2E` (`46`) bytes do descritor e compara contra `*(short *)(buf + 0x2E)`, emitindo `"[fmu] error! list_sum is invalid.\n"` (`0x504349F6`) se divergir. |
| **`FUN_5040df40`** (`0x5040DF40`) | `1058 B` | **Máquina de Atualização de Flash do Timer-MCU (`IC7501`):** envia comando de queda de drive (`0x84 .. 0xFF`, `func_0x5040ba7e(0xf)`), aguarda `3000 ms`, manipula **`P5OUT`** (`&= 0xF7`) e **`0xDB000500`** (`&= 0xFF3F` / `\|= 0x40`), envia **`0x8B`**, ajusta divisor de baud serial para Timer em **`0xDB000100`** (`(v & 0xFFF0) \| 10`), envia `16` bytes `0x00` de sincronismo (`30 ms` entre bytes), executa handshake **`0xB0`** (espera eco `-0x50`) e **`0xB3`** (espera eco `-0x4D`), dispara apagamento/reset com **`0xA7`** + **`0xD0`**, realiza polling do byte de status **`0x70`** (bits `0x80`, `0x20`, `0x02` em 2 bytes retornados) em até 2 tentativas e chama gravação (`func_0x5040e41b`) + verificação (`func_0x5040e6dc`). |
| **`FUN_504068bd`** (`0x504068B0`..`0x504069BF`) | — | Laço de inicialização de drive/bandeja no boot FVU com ramo de giro infinito (`BRA 0x504069bf`) caso `0x504F00F0 != 0` após checagens em `0x50480288`/`0x50480284`/`0x5040c779` e `0x504F00F4`. |

### 5.2 `/EH55_MAIN_3410PQ.bin` (`0x57000000..0x57C6692B`) — Descoberta e Cadeia de Referências do Stub Serial `+$OK#9A`

1. **Descoberta Principal — Stub Serial de Depuração Remota (`FUN_574f0775` @ `0x574F0775`, `413 B`):**
   * O firmware principal contém um **interpretador de pacotes RSP/serial completo** associado às strings em **`0x57A7243E..0x57A72485`**:
     `_dev_SERIAL`, `-$E01#A6` (`0x57A7244A`), `-$E00#A5` (`0x57A72453`), `+$OK#9A` (`0x57A7245C`), `-$E02#A7` (`0x57A72464`), `,` (`0x57A7246D`), `-$E03#A8` (`0x57A7246F`), `#`, `:`, `-$E04#A9` (`0x57A7247C`).
   * **Fluxo exato do handler `FUN_574f0775` e sub-rotinas decompiladas (`0x574F06E2..0x574F1323`):**
     * **`FUN_574f06e2` (`0x574F06E2`) / `FUN_574f0735` (`0x574F0735`):** Entrada do laço RSP; configura o buffer de recepção em `0x5177FE42` e o buffer de transmissão em `0x51780282` (com `0x15` NAK ou prefixo `+$`).
     * **`FUN_574f0913` (`0x574F0913`, `290 B`):** Receptor de quadros RSP por polling direto da UART (`*puRam515b2e48 & 0x10`, leitura em `*pcRam515b2e4c`); exige marcador inicial `'$'` (erro envia `0x15` + `-$E01#A6`). No comando binário `'X'`, interpreta o comprimento entre `','` (`0x2C`) e `':'` (`0x3A`) para que bytes `0x23` (`'#'`) dentro do payload binário não encerrem prematuramente o pacote.
     * **`FUN_574f0775` (`0x574F0775`, `413 B`):** Calcula soma de 8 bits módulo 256 sobre os bytes internos do frame e valida contra os 2 dígitos hex finais após `'#'` (divergência envia `-$E00#A5`); despacha comandos pelo caractere `cmd = buf[1]`:
       * `'m'` / `'x'` / `'c'` → **`FUN_574f0a33` (`0x574F0A33`, `560 B`):** leitura de memória em hex (`'m'`, usando tabela `0x57A72417` em blocos de `0x400` bytes), binário bruto (`'x'`, blocos de `0x440` bytes) ou comprimida (`'c'`, via `func_0x574f0fef` / `func_0x574f102b` / `func_0x574f11bd` com cabeçalho `0x8000 | len`), anexando `'#'` + checksum hex de 2 dígitos; envia `-$E03#A8` (`0x57A7246F`) em falha de parse.
       * `'M'` / `'X'` → **`FUN_574f0c63` (`0x574F0C63`, `320 B`):** escrita de memória em hex (`'M'`, validando caracteres via `*(byte *)(ch + 0x57B96BED) & 0x44` e tabela `0x57A723F7`, respondendo `-$E04#A9` se inválido) ou binário bruto (`'X'`); responde **`+$OK#9A`** (`0x57A7245C`).
       * `'g'` → **`FUN_574f0da3` (`0x574F0DA3`, `223 B`):** serializa `0x70` (`112`) bytes do contexto de registradores da CPU apontado por `iRam5316ae8c` (224 dígitos hex + `#checksum`).
       * `'Z'` (`'Z0'` / `'Z1'`) → responde **`+$OK#9A`**; no comando `'Z0'`, limpa flag de laço ativo (`in_R4 = 0`), libera recursos em `uRam577e5dc4` (`func_0x5770d53e` / `func_0x5770d5bc`) e seta **`0xC0001002 |= 0x80`** antes de retornar.
       * Outro opcode → responde `-$E02#A7`.
   * **Cadeia Completa de Registro e Tabelas de Driver (`_dev_SERIAL`):**
     * **Registro dinâmico (`FUN_574f0538` @ `0x574F0538`, `73 B`):** registra `"_dev_SERIAL"` (`0x57A7243E`) via `func_0x5770897c(param_1, 0x5177f640)` e instala callbacks padrão **`0x574f12f7`** (leitura de 1 byte em `*puRam515b2e50`) e **`0x574f1323`** (`func_0x5770d625(uRam577e5dc8)`).
     * **Tabela de Dispositivos do Kernel (`0x577E4E80..0x577E4EC0`):** registros de 12 bytes `(u32 dev_id, u32 drv_idx, char *name)` contendo `[0] (1, 1, "_mn864600ieee1394")`, `[1] (2, 0x101, "_mn864600ake")`, `[2] (3, 2, "_fp_path")`, `[3] (4, 3, "_key_path")` e **`[4] (5, 4, 0x577E4ED0 "_dev_SERIAL")` em `0x577E4EB0..0x577E4EBC`**.
     * **Vtable de Operações do Driver `_dev_SERIAL` (`0x577E4D94..0x577E4DB4`):** `dev_id = 5` (`0x577E4D94`), `open = 0x574F0E82`, `close = 0x574F0F19`, `read = 0x574F0F15` (chama `FUN_574f0f3e`, que consome o ring-buffer de `2048` bytes `0x5177F642[uRam515b2e54 & 0x7FF]`, decrementa `iRam515b2e58` e sinaliza `uRam577e5dc4`), `write = 0x574F0F17`, `ioctl_a = 0x574F0F1D`, `ioctl_b = 0x574F0F1B`, `poll = 0x574F0F9D`.
     * **Vetor de Interrupção Serial e Inicialização:** `0x577E4E50` mapeia `0x00030015` → **`0x574F0F9F`** (ISR da `SC0` para vetor `0x15`); `0x57C59F54` aponta para **`0x574F0590`** (rotina principal de RX/RSP em `serial.a`); e o suposto `CALL16` em `0x574F828B` → `0x574F0735` mostrou-se falso positivo em offset `+1` de `0x574F828A` (`[fc cd aa 84 4f 57] MOV 0x574f84aa,D1`). Chamadas reais de 32 bits em MN103S usam o opcode de 7 bytes `0xDD <rel32_le> <regs_u8> <imm8>` (detalhamento na Seção 5.4).
2. **Tabela de Strings de Bibliotecas e Dispositivos (`0x57C15400..0x57C1555F` & `0x57A52D98`):**
   * Confirmada a compilação em `EH55_MAIN_3410PQ.bin`: `DVR_SYSTEM_LIB:rtscdd.a K0.58 (2006/07/05):RTSD` (`0x57C15403`), `rtsdauth.a Ver0.15-MV3 (2005/12/14)` (`0x57C15433`), **`DVR_SYSTEM_LIB: serial.a Ver0.33 (2005/06/28):Debug Serial`** (`0x57C15470`/`0x57C15480`), `sys_boot.a Ver6.0.076.03 (2006/03/14): sys_boot(M6)` (`0x57C154AB`), `sys_cd.a Ver 7.06 (2005/11/02): CD Media App` (`0x57C154EF`), `sys_cntl.a Ver6.0.143.19 (2006/05/12)` (`0x57C1552A`).
   * Descritor `_dev_SERIAL` em `0x577E4ED0` (e tabela vizinha `_mcp_graph`, `_key_path`, `_fp_path`, `_mn864600ake`, `_mn864600ieee1394`) além dos nomes de registradores `SC0STR` (`0x57A52D98`), `SC0RXB`, `SC0TXB`, `SC0ICR`, `SC0CTR`. *(Nota: offsets do binário `main3060np_unpacked.bin` usado em sondas preliminares ficam deslocados por ~`+0x838C0` em relação a `EH55_MAIN_3410PQ.bin`, cuja RAM vai de `0x57000000` até `0x57C6692B`.)*
3. **Correção de Atribuição (`FUN_574f8354` / `FUN_574f862b` / `FUN_574f79f2`):**
   * A inspeção decompilada mostrou que `FUN_574f8354` (`48 B`) e `FUN_574f862b` (`142 B`) lêem identificadores de hardware na **EEPROM/NVRAM** (endereços `0xE4`, `0xD0`, `0xD3`, retornando códigos `'H'`/`'I'`/`'J'` / default `'L'`), alimentando `src/sys_cd_apl_main.c` (`0x57AE2F53`, `"CD APL Start!! \n"` em `0x57AE2F69`), e **não** formam parte do protocolo serial.
4. **Menu de Debug `ENC_MENU` (`0x57074B4F`..`0x57074CAA`):**
   * Loop de comando serial encerrado por `0x1A` (`Ctrl-Z`), emitindo `" Not supported!! \n"` (`0x577F926F`) quando o índice passa do limite e despachando 16 handlers pela tabela de ponteiros em `0x577F9284`.

### 5.3 Comparativo dos Carregadores SD (`BOOT.BIN` vs `eh55_frm_PROG061_BOOT.bin`) e Arquivos Prontos para Cartão SDSC

O cruzamento entre `/BOOT.BIN` (`ver0.63`) e `/eh55_frm_PROG061_BOOT.bin` (`ver0.61`) esclareceu exatamente como cada estágio lê `DIGADIAG.FRM` e `PANA__SD.FRM` do cartão **SDSC ($\le 2\text{ GB}$, FAT16)**:

| Estágio / Binário | Funções Decompiladas | Comportamento para `DIGADIAG.FRM` | Comportamento para `PANA__SD.FRM` |
| :--- | :--- | :--- | :--- |
| **1º Estágio (`BOOT.BIN` / Flash `0x44000000`)** | `FUN_44006cd9` → `FUN_44007ba5` → `FUN_4400a97c` (`SDDIAG`) e `FUN_44006fe7` → `FUN_4400a8c7` / `FUN_4400aa5f` / `FUN_44007a12` (`FVU`) | Quando `SW_UART = LOW` (`serial boot mode`, `param_2 == 2`), exibe `"SDDIAG"`, inicializa o cartão (`FUN_4400a247`), abre `"DIGADIAG.FRM"`, lê **`0x20000` (`131.072`) bytes** para `0x93FC0000`, chama `uncompress(&DAT_93fc0000)` (`EXTRHEAD` zlib em texto claro → `0x50400000`), imprime `"*** start diag ***"` e salta para `0x50400000`. | Abre `"PANA__SD.FRM"`, lê **`0x2000` (`8.192`) bytes** de cabeçalho TOC em texto claro (**128 entradas de 64 bytes**, `list_sum` em `+0x3E` validado por `FUN_4400a6d0`) e lê cada módulo sem cifra Feistel externa (`FUN_44007a12`). |
| **2º Estágio (`PROG061` / RAM `0x50400000`)** | `FUN_5040ebfe`, `FUN_5040ed7c`, `FUN_5040ea7d`, `FUN_5040ea1c`, `FUN_5040722b` e `FUN_5040eca9` | `FUN_5040eca9` abre `"DIGADIAG.FRM"` e lê **`0x40000` (`262.144`) bytes** brutos diretamente para `0x90400000`. | Quando `iRam50480284 == 5` (modo SD), `FUN_5040ebfe` abre `"PANA__SD.FRM"`, `FUN_5040ed7c` lê **`0x1800` (`6.144`) bytes** para `0x50481B5C` **sem Feistel** (diferente de `FUN_5040eb78` do CD-R que decifra via `FUN_5040e9ae`), `FUN_5040ea7d` percorre **entradas de 48 bytes (`0x30`)** com `list_sum` em `+0x2E` (`FUN_5040ea1c`), e `FUN_5040722b` carrega os módulos sem Feistel externa validando `fvu_checksum_u32` (`FUN_5040eb35`). |

Todos os arquivos `.FRM` foram gerados e verificados via `panadiga frm pack` (incluindo suporte ao novo `--mode sd48` e metadados por módulo `NAME:PATH:VER:FLAG`) para transferência imediata à máquina com leitor SD:
* **`DIGADIAG.FRM` (raiz e `OPCAO_2_DIAGNOSTICO_FVU_SD\`):** `131.072` bytes (`0x20000`, `SHA-256: 78779d805eab049c65c3eec5e2b0646927c18168c4c97ae8eb24488a1caf339f`) — contêiner `EXTRHEAD` (`tag='FVU '`, `build='PROG061'`, `dst=0x50400000`) para boot direto via `SDDIAG`.
* **`PANA__SD.FRM` (raiz e `OPCAO_1_ATUALIZACAO_COMPLETA_SD\`):** `7.533.792` bytes (`SHA-256: edbe9ea8ec8a4905c5c891d37484205e2897c12a50f73d272a268ebe91bb7a1a`) — TOC plaintext de 64B (`0x2000`) + `DRV S211`, `PROG061 ` e `MAIN3060NP`.
* **`OPCAO_3_FORMATO_VER061_RAW\`:** variante para o 2º estágio `PROG061` (`PANA__SD.FRM` com TOC plaintext de 48B `0x1800` = `7.531.744` bytes e `DIGADIAG.FRM` bruto `0x40000` = `262.144` bytes).

### 5.4 Investigação do Spindle Não Girar (`CDDIAG` / `DRV S211` / `BOOT.BIN`), Tabela CLI Completa e Mecanismo de Entrada RSP no `MAIN_3410PQ`

1. **Causa Raiz do Não Acionamento do Motor Spindle (`spinner`) no Boot / `CDDIAG` (`DRV` + `BOOT.BIN`):**
   * **Bloqueio por falha prévia de laser (`LD_OFSCAN` antes do giro do spindle):** Na rotina de startup de disco **`FUN_400c3d1d`** (`/DRV_decomp.bin`, equivalente a `0x0be162` em `drv_s211_unpacked.bin`), o firmware do drive executa obrigatoriamente **`FUN_40080241`** (`LD_OFSCAN`, comando DSP `0x2c`, `0x07d2b3` em `s211`) **antes** de chamar **`FUN_400c8eda(1, 25000)`** (comando DSP `0x0f`, giro do spindle, `0x0c313e` em `s211`).
   * **Caminho de retentativa (`FUN_400c39cd`):** O caminho alternativo/retry acionado via **`FUN_40095555`** (quando `(DAT_00000e41 & 0x80) == 0`) também chama **`FUN_40080387()`** (verificação de laser / `LD_OFSCAN` marcando `DAT_00001538 |= 0x7f` + `FUN_400db4a7(7)`) antes de alcançar `FUN_400c8eda(1, 25000)`.
   * **Inicialização de registradores do diodo laser e gating (`DAT_00001538`):**
     * **`FUN_400804d3(1)`** alterna o pino `0x1c` → registrador DSP `0x55 = 0xff` (`FUN_4001053c`), chama **`FUN_40080547(1)`** (`0x07d5a3` em `s211`), que lê o byte sombra da EEPROM **`0x40160b97`** via `FUN_400d70b5(0xb97)` (ou `0xb96` para `param_1 == 0`) e escreve o registrador DSP `0x59` via `FUN_400ae1ab(0, 0x59)` se o byte `!= 0xff`, e então configura 9 registradores DSP `[0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x50, 0x5c]` (`DAT_40129d30`, em `0x12535c` na `s211`, com eventuais overrides `'#'` / `'\'` de `0x50c` / `0x50a` em `0x40160000`) via `FUN_400af605(0x500, ...)`.
     * **`FUN_40080241`** (`LD_OFSCAN`) marca o bit 6 de `DAT_00001538` (`|= 0x40`). **`FUN_40080578`** exige `(DAT_00001538 & 0x40) != 0` (caso contrário registra `_DAT_00000de8 = 0x714` e retorna `0x3d`) e leitura não nula em `FUN_4008063b(0)` (caso contrário `_DAT_00000de8 = 0x715`), antes de acionar `FUN_400db4a7(0x23)`. Já **`FUN_40083396`** carrega tabelas de calibração/OPC (`DAT_40119778`, `DAT_401196dc`, `DAT_401197c8`) e aciona `FUN_400db4a7(0x46)` / `0x2b` / `0x7c`.
     * **Aborto com código DSP `0x03A7` (`0x83A7`):** Quando `LD_OFSCAN` falha retornando status DSP `0x03A7`, `FUN_400db60c` (`0x0d5378` em `s211`) grava **`_DAT_00000de8 = (ushort)uVar4 | 0x8000` (`0x83A7`)**, encerrando `FUN_400c3d1d` e fazendo **`FUN_40094f10`** (`0x0902e4` em `s211`) transicionar `DAT_00000dd2` para `0x40` / `0x20` sem jamais energizar o motor spindle. (Adicionalmente, **`FUN_400d6f39`** valida checksum e contadores em `_DAT_40161000` / `_DAT_4016102e` / `_DAT_40161030` da sombra EEPROM `0x40160000`, acusando `_DAT_00000de8 = 0x7104` / `0x7105` / `0x7106` em falhas de EEPROM.)
   * **Implicação no `BOOT.BIN` (`CDDIAG`) e status do `PROG061`:**
     * Em `/BOOT.BIN`, `cd_diag_boot_mode` chama **`FUN_44009bd7`**, que só envia ATAPI **`0x1e`** + **`0x1b` (`cdb[4]=2` EJECT)** após sucesso de `startup_drive(1)`; falhando `startup_drive(1)`, entra em **`FUN_440079a4()`** → `FUN_440056b1(0x80, 0)`, `FUN_440056b1(0x81, 1)`, `FUN_44005896()`, e trava em `while (true)`. Além de `EXTRHEAD` algoritmo `0` (zlib/deflate em `FUN_44028393`), **`FUN_4400b6c6`** também suporta algoritmos `2` (`FUN_4402ba08`) e `3` (`FUN_4402c22c`, LZMA) para imagens em `0x84070000` → `0x90202000` / `0x90362000`.
     * Confirmado também que **`/eh55_frm_PROG061_BOOT.bin` é idêntico a `prog061_dec.bin`** (`0x50400000..0x5043FFFF`) e implementa o atualizador **`***** Fvu ver%s [%s] *****`**, não um menu interativo de teste manual de spindle/laser.

2. **Interfaces de Diagnóstico Existentes no Firmware `DRV` (`/DRV_decomp.bin` ↔ `drv_s211_unpacked.bin`):**
   * **Mapeamento Cruzado Validado (`DRV_decomp` → `s211`):** `FUN_4001455d`→`0x0145aa`, `FUN_4001591c`→`0x015970`, `FUN_4001b718`→`0x01b676`, `FUN_4001b871`→`0x01b7cf`, `FUN_4002025e` (`FE`)→`0x020300`, `FUN_400277de`→`0x02791a`, `FUN_40027fcd`→`0x028109`, `FUN_4002803a`→`0x028176`, `FUN_400280ca`→`0x028206`, `FUN_4002a1ce`→`0x02a2ee`, `FUN_40080241`→`0x07d2b3`, `FUN_400802a8`→`0x07d31a`, `FUN_40080547`→`0x07d5a3`, `FUN_40094f10`→`0x0902e4`, `FUN_400c3d1d`→`0x0be162`, `FUN_400c8eda`→`0x0c313e`, `FUN_400d65e1`→`0x0d07e3`, `FUN_400db4a7`→`0x0d5213`, `FUN_400db60c`→`0x0d5378`, `FUN_400db7e8`→`0x0d5554`, `pin[0]`→`0x0d7540`, `DAT_40129d30`→`0x12535c`, `SLOGON`→`0x0d92a4`, `LD_OFSCAN`→`0x0dae02`, `0.5mW`→`0x0de266`, `MATSHITA`→`0x008ae6`, e mapa de opcodes CDB `0x400dea64` → `0x0d8760` (`s211` com tabela de ponteiros em `0x0d86a0..`).
   * **Desbloqueio de Serviço e Console Serial do Drive (`0x71` = `113` comandos):**
     * O receptor serial **`FUN_400273bd`** lê `DAT_0000b002` (quando `(DAT_0000b003 & 0x47) == 0`) para o buffer `+0xce6` até `\r` (`0x0d`) e dispara `FUN_400dd690(0, 0x2000)`.
     * O despachante **`drv_serial_cli_dispatch`** percorre **`0x71` (`113`)** registros de `20` bytes em `0x400df5c0` (`0x0d92a4` em `s211`), exigindo **`(DAT_00000cd8 & 1) != 0`** ou flag **`0x80`** no byte `+0x0c` da entrada (caso do comando **`PASS`**, em `0x4013420a` / **`FUN_400278a3`** que seta `DAT_00000cd8 |= 3` e configura `0xffff896c`, além do desbloqueio via byte EEPROM `0x80` em **`FUN_400c69df`** quando bits `0x80|0x40` e sentinela `0xa7 == 0xa5` conferem). O laço principal **`FUN_400138cc`** só processa a linha CLI (`FUN_40028489(0x100)` / `FUN_400274ab()`) quando `DAT_00000cd8 & 1` está ativo.
     * Principais grupos entre os `113` comandos CLI observados na tabela: sessão/memória/EEPROM (`SLOGON`, `SLOGOFF`, `PASS`, `VER`, `RESET`, `LOGCUT`, `MMODE`, `SMODE`, `PRMPT`, `RB`, `RW`, `WB`, `WW`, `EEP`, `D`), logs (`MLOG`, `DSCLOG`, `CLOG`, `ACSLOG`, `TMPLOG`, `LFMLOG`), servo/laser/ODC (`PON`/`HON`, `POFF`/`HOFF`, `PCONT`, `PDOWN`, `PTIME`, `DSCC`, `DSC`, `FEP`, `DSCREG`, `LDD`, `CIRC`, `DRC`, `ODCMEM`, `ODCCTL`, `ODCSTS`, `ZTBL`, `ADR`), controle direto de bandeja/motor (`START`, `IDLE`, `STANDBY`, `STOP`, `SPEED`, `WRSPEED`, `RDSPEED`, `EJECT`, `LOAD`, `SEEK`, `RPM`, `JMON`, **`FUN_4002a1ce`** montando `CDB 0x1B` via `FUN_4001b71d`), mídia (`DVDROM`..`DVDRWDL`) e testes de leitura/potência (`RFCHK`, `RFCHK2`, `BLER`, `WTADJ`, `MEASR`, `TMP`, `BER`, `BER2`, `RDPCA`, `TBETA`, `SVER`, `SCRIN`, `SCREXE`, `BLCHK`, `RDLNG`, `FORMAT`, `JIT`, `SUBQ`, `DATAID`).
     * No barramento ATAPI, além do fluxo `0x1e` + `0x1b` (`FUN_44009bd7` no `BOOT.BIN`; `FUN_4002a1ce` → `FUN_4001b71d` `CDB 0x1B` no `DRV`) e de **`FUN_4002025e` (`FE` → `0x020300`)**, **`FUN_400201b5`** implementa o comando que ativa bit 7 (`|= 0x80`) ou bits `0..6` (`|= 0x7f`) de **`DAT_00000e41` (`MMODE`)** após validar `_DAT_00000246 & 0xff20 == 0`, `(DAT_0000023d & 0x1f) == 0`, `(DAT_00000246 & 0x1f) == 3` e comparar 8 bytes do CDB (`0x23e..0x245`) com `&DAT_400df438`.

3. **Esclarecimento Definitivo do Stub RSP (`+$OK#9A`) em `EH55_MAIN_3410PQ.bin`:**
   * **Gating normal vs. exceção (`iRam515b270c == 0`):** A chamada `DD-CALL` em `0x574B5ACA` invoca **`FUN_574aebdc`**, que **seta `uRam515b270c = 1` apenas enquanto `FUN_574ae930()` executa** (dump/exception handler) e zera `uRam515b270c = 0` ao retornar. Portanto, dentro de **`FUN_574f0590`**, o ramo **`iRam515b270c == 0`** é o **estado normal fora de exceção**.
   * **Gatilho RSP por `0x15` (`NAK`) e sequência mágica `0x0E 0x04 0x02`:** No ramo `iRam515b270c == 0`, `FUN_574f0590` sincroniza `uRam577e5dc4` (constante ROM `0x0f` em `0x577e5dc4`) e lê 1 byte via `FUN_574f08b6` (`*puRam515b2e48 & 0x10`, dado em `*puRam515b2e4c`):
     * Se o byte recebido for **`0x15` (`NAK`)**, limpa bit 7 de `UNK_c0001002` (`&= 0x7f`) e entra diretamente no laço RSP (`FUN_574f0913` + handlers `'m'`/`'x'`/`'c'` → `FUN_574f0a33`, `'M'`/`'X'` → `FUN_574f0c63`, `'g'` → `FUN_574f0da3` escrevendo via `FUN_574f0876` em `*puRam515b2e50`, `'Z0'`/`'Z1'` → `+$OK#9A`, `'Z0'` sai do laço e restaura `UNK_c0001002 |= 0x80`).
     * Para bytes `!= 0x15`, armazena no ring buffer `0x5177f642` (`0x800` bytes) e reconhece a sequência de 3 bytes **`0x0E -> 0x04 -> 0x02`** (`bRam515b2e60: 0 -> 3 -> 0xc3 -> iRam515b2e64 = 1`), que suprime o register dump de `FUN_574ae930` no próximo evento (`if (iRam515b2e64 == 0) ... else iRam515b2e64 = 0`).
   * **Inicialização UART/Timer (`FUN_574efb97` & `FUN_574f0e82`) e correção de opcode:** **`FUN_574f0e82`** (`open` de `_dev_SERIAL` em `0x577e4d98`) configura `TMPSCNT = 0x80`, `TM2BR = 7`, `TM2MD = 0x41/1/0x81`, `SC0ICR`, `Ramd4002000 = 0xc085` (usando constante ROM `uRam577e5dc8 = 0x10`). Já **`FUN_574efb97`** registra os callbacks `0x574f0498`/`0x574f0464` via `FUN_574f052f` (`"_dev_SERIAL"`, `0x5177f640`), chama `func_0x574ef7d2()` e, se `(bRam515b2e39 & 2) != 0`, configura `TM2BR = 0x2b`, `Ramd4002000 = 0xc085` e os ring buffers em `0x5316a260`/`0x5316aa70` (consumidos por `FUN_574efad8`). Quanto às chamadas reais de 32 bits em MN103S, foram confirmados `DD-CALL` legítimos para `FUN_574f8242` (`0x574F7B71`) e `FUN_574f8278` (`0x574F7CF4` e `0x574FDFEB`), descartando em definitivo o falso positivo anterior em `0x574F828B`.

---

## 6. Roadmap Técnico: Melhorias no Pico & Próximos Passos no Disassembly

1. **Próxima Evolução no Firmware `pico-p7501`:**
   * **Cliente / Sonda `PANA_RSP` (`+$...#XX`):** aproveitar `PIO1` + parser de checksum 8-bit já validado no `HEXCAP`/`WAITTX` para sondar o stub **`FUN_574f0590`** enviando **`0x15` (`NAK`)** seguido de **`$g#67`**, **`$Z0#8A`** / **`$Z1#8B`** ou **`0x0E 0x04 0x02`** no contexto em que `_dev_SERIAL` estiver inicializado no MAIN (`+$OK#9A` / `-$E01#A6`).
   * **Amostrador PIO de alta resolução:** captura lógica simultânea em `GP2`/`GP3`/`GP4` sem polling por CPU para medir tempos de resposta do Timer-MCU (`IC7501`) vs `MN103S` durante janelas de atualização FMU (`0x84`/`0xB0`/`0xB3`).
   * **Exposição de `BRIDGE` e `WAITTX` interativo no Frontend Tauri:** integrar o passthrough USB-CDC ↔ `SC0` (`~~~` para sair) e controle serial do drive (`PASS` + `START`/`LDD`/`DSCREG`) diretamente no painel de Bancada Desktop.
2. **Continuação do Disassembly & Bancada Real (`DMR-EH55`):**
   * **Isolar o motivo físico/EEPROM da falha pré-spindle (`0x83A7` em `LD_OFSCAN`):** mapear os campos lidos de `0x40160000` (`0xb97`, `0x50c`, `0x50a` e endereços referenciados por `DAT_40119778`..`DAT_401197c8`) e verificar na bancada a resposta do conjunto óptico (`0.5mW`, fotodiodo/FEP/driver de laser) via **`PASS`** + comandos `LDD`/`FEP`/`START` na serial do drive (ou via ponte no `MAIN`).
   * **Rastrear os chamadores reais de `FUN_574efb97` e `func_0x574ef7d2` em `EH55_MAIN_3410PQ.bin`:** identificar o ponto do boot normal ou modo de fábrica em que `(bRam515b2e39 & 2)` / `FUN_574f0e82` habilitam a `SC0` (`puRam515b2e48`/`4c`/`50`) para o stub `0x15` (`FUN_574f0590`).
   * Testar na bancada o cartão **SDSC ($\le 2\text{ GB}$, FAT16)** preparado em `sd_card_eh55\` com `DIGADIAG.FRM` + `PANA__SD.FRM` para ultrapassar o erro `-91006` e capturar a execução de `*** start diag ***` (`0x50400000`) na `COM10`.
