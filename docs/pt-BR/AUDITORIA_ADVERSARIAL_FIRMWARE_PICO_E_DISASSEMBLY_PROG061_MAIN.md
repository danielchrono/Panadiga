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

### 5.2 `/EH55_MAIN_3410PQ.bin` (`0x57000000..0x57C6692B`) — Descoberta do Stub Serial `+$OK#9A`

1. **Descoberta Principal — Stub Serial de Depuração Remota (`FUN_574f0775` @ `0x574F0775`, `413 B`):**
   * O firmware principal contém um **interpretador de pacotes RSP/serial completo** associado às strings em **`0x57A7243E..0x57A72485`**:
     `_dev_SERIAL`, `-$E01#A6` (`0x57A7244A`), `-$E00#A5` (`0x57A72453`), `+$OK#9A` (`0x57A7245C`), `-$E02#A7` (`0x57A72464`), `,` (`0x57A7246D`), `-$E03#A8` (`0x57A7246F`), `#`, `:`, `-$E04#A9` (`0x57A7247C`).
   * **Fluxo exato do handler `FUN_574f0775`:**
     * Lê pacote via `func_0x574f0913`; exige marcador inicial `'$'` (erro envia ACK/NAK `0x15` + `-$E01#A6`).
     * Calcula soma de 8 bits módulo 256 sobre os bytes internos do frame e valida contra os 2 dígitos hex finais após `'#'` (divergência envia `-$E00#A5`).
     * Despacha comandos pelo caractere `cmd = buf[1]`:
       * `'m'` / `'x'` → `func_0x574f0a33()` (leitura de memória).
       * `'M'` / `'X'` → `func_0x574f0c63()` (escrita de memória).
       * `'c'` → `func_0x574f0a33()` (continuação).
       * `'g'` → `func_0x574f0da3()` (leitura de registradores).
       * `'Z'` (`'Z0'` / `'Z1'`) → responde **`+$OK#9A`**; no comando `'Z0'`, limpa flag de laço ativo (`in_R4 = 0`), libera recursos em `uRam577e5dc4` (`func_0x5770d53e` / `func_0x5770d5bc`) e seta **`0xC0001002 |= 0x80`** antes de retornar.
       * Outro opcode → responde `-$E02#A7`.
   * **Registro do dispositivo (`FUN_574f0538` @ `0x574F0538`, `73 B`):** registra `"_dev_SERIAL"` (`0x57A7243E`) via `func_0x5770897c(param_1, 0x5177f640)` e instala callbacks padrão **`0x574f12f7`** (leitura de 1 byte em `*puRam515b2e50`) e **`0x574f1323`** (`func_0x5770d625(uRam577e5dc8)`).
2. **Tabela de Strings de Bibliotecas e Dispositivos (`0x57C15400..0x57C1555F` & `0x57A52D98`):**
   * Confirmada a compilação em `EH55_MAIN_3410PQ.bin`: `DVR_SYSTEM_LIB:rtscdd.a K0.58 (2006/07/05):RTSD` (`0x57C15403`), `rtsdauth.a Ver0.15-MV3 (2005/12/14)` (`0x57C15433`), **`DVR_SYSTEM_LIB: serial.a Ver0.33 (2005/06/28):Debug Serial`** (`0x57C15470`/`0x57C15480`), `sys_boot.a Ver6.0.076.03 (2006/03/14): sys_boot(M6)` (`0x57C154AB`), `sys_cd.a Ver 7.06 (2005/11/02): CD Media App` (`0x57C154EF`), `sys_cntl.a Ver6.0.143.19 (2006/05/12)` (`0x57C1552A`).
   * Descritor `_dev_SERIAL` em `0x577E4ED0` (e tabela vizinha `_mcp_graph`, `_key_path`, `_fp_path`, `_mn864600ake`, `_mn864600ieee1394`) além dos nomes de registradores `SC0STR` (`0x57A52D98`), `SC0RXB`, `SC0TXB`, `SC0ICR`, `SC0CTR`. *(Nota: offsets do binário `main3060np_unpacked.bin` usado em sondas preliminares ficam deslocados por ~`+0x838C0` em relação a `EH55_MAIN_3410PQ.bin`, cuja RAM vai de `0x57000000` até `0x57C6692B`.)*
3. **Correção de Atribuição (`FUN_574f8354` / `FUN_574f862b` / `FUN_574f79f2`):**
   * A inspeção decompilada mostrou que `FUN_574f8354` (`48 B`) e `FUN_574f862b` (`142 B`) lêem identificadores de hardware na **EEPROM/NVRAM** (endereços `0xE4`, `0xD0`, `0xD3`, retornando códigos `'H'`/`'I'`/`'J'` / default `'L'`), alimentando `src/sys_cd_apl_main.c` (`0x57AE2F53`, `"CD APL Start!! \n"` em `0x57AE2F69`), e **não** formam parte do protocolo serial.
4. **Menu de Debug `ENC_MENU` (`0x57074B4F`..`0x57074CAA`):**
   * Loop de comando serial encerrado por `0x1A` (`Ctrl-Z`), emitindo `" Not supported!! \n"` (`0x577F926F`) quando o índice passa do limite e despachando 16 handlers pela tabela de ponteiros em `0x577F9284`.

---

## 6. Roadmap Técnico: Melhorias no Pico & Próximos Passos no Disassembly

1. **Próxima Evolução no Firmware `pico-p7501`:**
   * **Cliente / Sonda `PANA_RSP` (`+$...#XX`):** aproveitar `PIO1` + parser de checksum 8-bit já validado no `HEXCAP`/`WAITTX` para sondar o stub `FUN_574f0775` (`'g'`, `'m'`, `'Z0'` esperando `+$OK#9A` / `-$E01#A6`) assim que o MAIN destravar `SC0` ou o stub estiver ativo.
   * **Amostrador PIO de alta resolução:** captura lógica simultânea em `GP2`/`GP3`/`GP4` sem polling por CPU para medir tempos de resposta do Timer-MCU (`IC7501`) vs `MN103S` durante janelas de atualização FMU (`0x84`/`0xB0`/`0xB3`).
   * **Exposição de `BRIDGE` e `WAITTX` interativo no Frontend Tauri:** integrar o passthrough USB-CDC ↔ `SC0` (`~~~` para sair) diretamente no painel de Bancada Desktop.
2. **Continuação do Disassembly & Bancada Real (`DMR-EH55`):**
   * Rastrear todas as referências (incluindo chamadas indiretas/ponteiros em RAM) até `FUN_574f0538` e `FUN_574f0775` em `EH55_MAIN_3410PQ.bin` para identificar qual tarefa RTOS, evento de teclado frontal/IR, ou byte de NVRAM habilita a entrada do stub `+$OK#9A`.
   * Em paralelo na bancada, testar mídia **SDSC ($\le 2\text{ GB}$, FAT16/32)** com `PANA__SD.FRM` ou `DIGADIAG.FRM` empacotado por `panadiga frm pack` para contornar o erro `-91006` (SDHC incompatível com `Loader ver0.61` medido em `COM10`).
