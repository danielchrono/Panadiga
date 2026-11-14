# Panasonic P7501 Async UART & Boot Probe Firmware Engine (RP2040 / RP2350)

Firmware oficial do controlador de bancada baseado em **Raspberry Pi Pico (RP2040)** e **Raspberry Pi Pico 2 (RP2350)** para captura de boot, diagnóstico elétrico e comunicação UART assíncrona (**`115200 8N1`**) através do conector de serviço **`P7501` (`B6B-ZR-3R4`, legenda `TO JIGU`)** do gravador **Panasonic DMR-EH55** (processador **Matsushita `AM34` / `MN103S`** + Syscon `IC7501`, placa `VEP79138`).

---

## 1. Visão Geral do Sistema

O firmware modular em [`firmware/pico-p7501/`](../../firmware/pico-p7501/) implementa em hardware e ISO C11 estrito (com cada módulo de produção $\le 257$ linhas em `common/`, `hal/`, `uart/`, `diag/`, `flash/`, `protocol/`, `gdb/` e `cli/`):

1. **Motor UART PIO1 Full-Duplex Multi-Baud + PIO0 Legado (`p7501_uart_duplex.pio` / `uart/p7501_uart_pio_engine.c`):**
   * Recepção passiva determinística via **`PIO1` (`SM0`)** em **`GP2`** (`P7501` Pino 6 `UARTP2M_T` / `MAX TXD`) e transmissão dedicada via **`PIO1` (`SM1`)** em **`GP4`** (`P7501` Pino 3 `UARTM2P_T` / `MAX RXD` via transistor `Q7514`), com divisor fracionário em ponto fixo para taxas dinâmicas (`BAUD` e autobaud `BAUDSCAN`), captura contínua (`UARTSTREAM`), log com deltas em microssegundos em RAM (`HEXCAP`, zero-drop a `115200 8N1`), injeção por gatilho (`WAITTX`) e ponte transparente USB-CDC ↔ SC0 (`BRIDGE`).
   * Mantém ainda a máquina legada `PIO0` (`p7501_uart.pio` / `uart/p7501_uart_capture.c`) para compatibilidade com `BOOTCAP` / `UARTMON` / `JIGBOOT`.
2. **Controle Seguro de Reset em Dreno Aberto (`XRESET` / `GP5`):** `GP5` (`P7501` Pino 2) opera exclusivamente alternando entre `0V` (reset ativo) e `Hi-Z` (entrada sem pulls, tempo de subida medido `~10–20 µs` via `QR7503`), prevenindo curto-circuito contra o supervisor de reset da Panasonic.
3. **Proteção Estrita em `NSW5V` (`GP3` / `P7501` Pino 5) & Controle `SW_UART` (`GP26` / `ADC0`, Pino 1):**
   * **`GP3` (`NSW5V` / `X_SW_5R2V`):** Provém de `IC1506` (`~5.15V` upstream via `D7510`, repouso em `~3.29V` nos resistores `R7570`/`R7572`). **PERMANECE SEMPRE como `Hi-Z Input` sem pull-up/pull-down** e nunca é dirigido em push-pull.
   * **`GP26` (`SW_UART`):** Amostra a tensão via `ADC0` (`~3.24–3.25V` em Hi-Z, `~2.70V` sob pull-down, `~3.26V` sob pull-up) e força nível `LOW` apenas pelo intervalo exato de entrada em modo serial boot (`XFWE`, ex. `JIG:350`), restaurando `ADC0 Hi-Z` via `p7501_hw_restore_sw_uart_adc0()` quando TX em `GP4` precisa cruzar `Q7514`.
4. **Driver de LED Não-Bloqueante com Trava Manual (`hal/p7501_led.c` & `hal/p7501_led_logic.c`):** Máquina de estados pura (10 Hz) alimentada por `repeating_timer` IRQ-safe (`GP25` em Pico/RP2040 e Pico 2/RP2350; fallback por tick para `CYW43_WL_GPIO_LED_PIN` em Pico W / Pico 2 W), com `MANUAL_LOCK` para impedir que polls `STATUS` sobrescrevam o modo escolhido pelo operador (`LED <IDLE|ACTIVE|GDB|FAULT|OFF|AUTO>`).
5. **Console Serial USB CDC + BLE Opcional (`PANADIGA> `):** Interface interativa a `115200 bps` compatível com terminais padrão (PuTTY, TeraTerm), `panadiga` CLI e aplicativo desktop/Android Tauri (`panadiga-desktop` / APK). Placas `pico_w` e `pico2_w` habilitam também ponte Bluetooth LE Nordic UART Service (`PANADIGA_BLE_ENABLED=1`).
6. **Ponte GDB RSP & Injeção/Sanitização Fail-Loud:** Inclui servidor **GNU Remote Serial Protocol (RSP)** (`GDB`) e primitivas `AUTODETECT` / `INJECT_LOADER CONFIRM` / `ERASE_NVRAM CONFIRM` que recusam silenciosamente operar no vazio caso `DCSR=0xFFFFFFFF` (sem monitor síncrono conectado em P7501); para chips travados fora do monitor sem dessoldagem, a rota primária confirmada de gravação da Spansion `S29JL064H` (`IC5001`) é programador externo (`SVOD 4` / `XGecu T48`) + `panadiga nvram clean`.

---

## 2. Requisitos de Ferramentas de Compilação

Para compilar o firmware a partir do código-fonte:
* **CMake** $\ge$ 3.13 (disponível no `PATH`)
* **Compilador ARM GCC:** `arm-none-eabi-gcc` $\ge$ 10.3
* **Build System:** `Ninja` (recomendado) ou `MinGW Makefiles`
* **Raspberry Pi Pico SDK:** configurada via variável de ambiente `PICO_SDK_PATH` (importada por `firmware/pico-p7501/pico_sdk_import.cmake`)

---

## 3. Como Compilar o Firmware Multi-Plataforma (RP2040 e RP2350)

O subsistema em [`firmware/pico-p7501/`](../../firmware/pico-p7501/) possui suporte nativo multi-placa (`pico`, `pico_w`, `pico2`, `pico2_w`), orquestrado pelo builder multiplataforma `scripts/build.py` e pela CLI Rust nativa (compilando *out-of-tree* em `target/pico-build/`):

### Opção A: Builder Unificado do Repositório (Windows / Linux / macOS)
```bash
# Compila RP2040 + RP2350, gera panasonic_p7501_uart_unified.uf2 e sincroniza crates/panadiga-core/src/pico/firmware_bins/
npm run build:pico
# Ou diretamente via Python:
python scripts/build.py --pico
```

### Opção B: CMake Manual + Fusão UF2 Nativa via Rust CLI
```bash
cmake -S firmware/pico-p7501 -B target/pico-build/pico -G Ninja -DCMAKE_BUILD_TYPE=Release -DPICO_BOARD=pico
cmake --build target/pico-build/pico --config Release

cmake -S firmware/pico-p7501 -B target/pico-build/pico2 -G Ninja -DCMAKE_BUILD_TYPE=Release -DPICO_BOARD=pico2
cmake --build target/pico-build/pico2 --config Release

cargo run -p panadiga-cli -- pico merge-uf2 \
  --rp2040 target/pico-build/pico/panasonic_p7501_uart.uf2 \
  --rp2350 target/pico-build/pico2/panasonic_p7501_uart.uf2 \
  --out target/pico-build/out/panasonic_p7501_uart_unified.uf2
```

### Artefatos Binários Gerados
A compilação sincroniza imagens `.uf2` auto-contidas em `crates/panadiga-core/src/pico/firmware_bins/` (embutidas no binário Rust) e no empacotamento `dist/`:
* **`panasonic_p7501_uart_rp2040.uf2`** $\leftarrow$ Imagem para Raspberry Pi Pico / Pico W e placas RP2040 compatíveis (264 KB SRAM).
* **`panasonic_p7501_uart_rp2350.uf2`** $\leftarrow$ Imagem para Raspberry Pi Pico 2 / Pico 2 W (RP2350 ARM Cortex-M33 / Hazard3 RISC-V, 520 KB SRAM).
* **`panasonic_p7501_uart_unified.uf2`** $\leftarrow$ Imagem universal concatenando os blocos RP2040 + RP2350 em um único `.uf2`.
* Imagens auxiliares (`.elf`, `.bin`, `.hex`, `.map`) geradas por `pico_add_extra_outputs(panasonic_p7501_uart)` permanecem em `target/pico-build/`.

### Integração Contínua (CI/CD)
A esteira em [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) e a validação local (`python scripts/validate.py` / `npm run validate`) executam verificações em Rust, TypeScript, C11 host tests ([`test_gdb_rsp_host.c`](../../firmware/pico-p7501/tests/test_gdb_rsp_host.c) e [`test_uart_codec.c`](../../firmware/pico-p7501/tests/test_uart_codec.c)), Python (`mypy --strict`, `pytest`), `jscpd`, Vitest e Playwright.

---

## 4. Como Gravar o Arquivo `.uf2` no Raspberry Pi Pico

O microcontrolador RP2040 / RP2350 possui bootloader USB nativo em ROM (**UF2 Bootloader**). Também é possível automatizar via `panadiga pico flash --port COM10`:

```text
       GRAVAÇÃO DRAG & DROP NO RASPBERRY PI PICO
  ┌───────────────────────────────────────────────────────────┐
  │  1. Desconecte o cabo USB do Pico                         │
  │  2. Mantenha pressionado o botão branco BOOTSEL           │
  │  3. Conecte o cabo USB ao computador                      │
  │  4. Solte o botão BOOTSEL                                 │
  │  5. O volume "RPI-RP2" (ou "RP2350") aparecerá no sistema │
  │  6. Copie "panasonic_p7501_uart_unified.uf2" para o drive │
  │  7. O Pico reiniciará automaticamente no novo firmware!   │
  └───────────────────────────────────────────────────────────┘
```

*(Com o firmware já rodando em USB CDC, também é possível enviar o comando `BOOTSEL` no prompt `PANADIGA> ` para entrar diretamente em modo UF2 sem pressionar o botão físico.)*

---

## 5. Conexão Serial USB CDC (Terminal & Tauri)

Após a reinicialização, o Pico enumera uma porta serial USB CDC (`COMx` no Windows, `/dev/ttyACM0` no Linux, `/dev/cu.usbmodem*` no macOS ou USB-OTG no Android):
* **Baud Rate:** `115200` (USB CDC 12 Mbps nativo)
* **Configuração:** 8 bits de dados, sem paridade, 1 stop bit (`8N1`), sem controle de fluxo

Ao conectar ou pressionar `<ENTER>`, o console exibe o banner oficial de [`p7501_cli_help.c`](../../firmware/pico-p7501/cli/p7501_cli_help.c):

```text
====================================================================
   PANADIGA: Panasonic DMR-EH55 P7501 Async UART & Boot Bench Probe
   MCU Host : RP2040 (Pico / ARM Cortex-M0+)
   Bus      : P7501 (B6B-ZR-3R4) 115200 8N1 (PIO1) + SW_UART / XRESET / NSW5V(Hi-Z)
   [GDB-RSP] Active Server Mode (RP2040 8 KB Cache)
   Type HELP for available P7501 UART, PIO1, FMU, diag, Flash, LED & GDB commands.
====================================================================

PANADIGA> 
```

---

## 6. Referência Completa de Comandos (`p7501_cli.c` / `p7501_cli_help.c`)

O firmware atende tanto operadores humanos quanto automação em Rust (`panadiga`), o script de bancada (`scripts/bench/`) e a UI reativa Tauri (`SSOT` em `crates/panadiga-core/src/ssot/spec.rs`):

| Comando | Sintaxe | Função Operacional Verificada |
| :--- | :--- | :--- |
| **`PING`** | `PING` | Responde `PONG` para verificação rápida de conectividade. |
| **`STATUS` / `VDD`** | `STATUS` \| `VDD` | Amostra `GP26 (ADC0)` em `P7501` Pino 1 (`SW_UART`) e imprime `VDD: X.XXV (OK / LOW / FLOATING)`. |
| **`BANNER`** / **`HELP`** | `BANNER` \| `HELP` \| `?` | Imprime banner de identificação ou lista completa de comandos. |
| **`BAUD`** | `BAUD [bps]` | Consulta ou ajusta o baud rate da máquina **`PIO1` (`SM0`/`SM1`)** via divisor fracionário FP16 (`300..921600 bps`). |
| **`BAUDSCAN`** | `BAUDSCAN [ms] [NORMAL\|JIG[:ms]\|NOPULL] [APPLY]` | Mede largura mínima/máxima de pulso LOW em `GP2` e estima o baud rate (com `APPLY` opcional em `PIO1`). |
| **`UARTSTREAM`** | `UARTSTREAM [ms] [NORMAL\|JIG[:ms]\|NOPULL]` | Captura em tempo real via `PIO1` com suporte a soltura temporizada de `SW_UART` (`JIG:350`) e restauração automática em `ADC0`. |
| **`HEXCAP`** | `HEXCAP [ms] [NORMAL\|JIG[:ms]\|NOPULL]` | Captura em RAM (`HEXCAP_RAM_BUF_CAP` = `2048` bytes) com delta `+us` e dump hex+ASCII após a janela (zero `overflow` a `115200 bps`). |
| **`UARTTXHEX`** | `UARTTXHEX [LOW:]<hex\|BRK> [rx_ms]` | Injeta sequência arbitrária em hexadecimal ou pulso `BREAK` de `120 ms` via `PIO1 TX` (`Q7514` requer `SW_UART` liberado, salvo prefixo `LOW:`). |
| **`WAITTX`** | `WAITTX <ms> <NORMAL\|JIG[:ms]\|NOPULL> "<trig>" <hex\|BRK> [post_ms]` | Aguarda substring de gatilho em `GP2` (ex.: `"wait 1"` após `JIG:350`) e injeta payload/BREAK pela `PIO1 TX` mantendo `NSW5V` em `Hi-Z`. |
| **`BRIDGE`** | `BRIDGE [baud] [NORMAL\|JIG[:ms]\|NOPULL]` | Ponte transparente bit/byte USB-CDC ↔ `SC0` (`PIO1`), encerrada pela sequência de escape `~~~` (ou timeout). |
| **`PWRTRACE`** | `PWRTRACE [ms] [NORMAL\|JIG[:ms]\|NOPULL]` | Amostra min/max de `GP26_SWUART` via `ADC0` e contabiliza bordas em `RXD` (`GP2`), `NSW5V` (`GP3`) e `TXD` (`GP4`) durante o boot. |
| **`BOOTCAP`** | `BOOTCAP [ms]` | Reseta `XRESET` em Open-Drain com `SW_UART` liberado e captura stream UART `115200 8N1` no boot (`PIO0`). |
| **`UARTMON`** | `UARTMON [ms]` | Escuta passivamente `GP2` (`UARTP2M_T`) sem forçar reset ou `SW_UART`. |
| **`JIGBOOT`** | `JIGBOOT [ms]` | Executa boot com `SW_UART = LOW` (`XFWE` / modo JIG) após pulso `XRESET` e captura `GP2` a `115200 8N1`. |
| **`UARTSEND`** | `UARTSEND [LOW:]<texto>` | Transmite comandos ASCII/bytes pela linha `GP4` (`UARTM2P_T`) em `115200 8N1` (`Hi-Z` padrão permite passagem por `Q7514`; prefixo `LOW:` força `SW_UART` baixo). |
| **`UARTPROBE`** | `UARTPROBE [sec]` | Executa varredura serial em `P7501` e restaura `GP26` ao estado `ADC0 Hi-Z` ao término. |
| **`FMUSCAN` / `FMUTEST` / `FMUBATCH`** | `FMUSCAN` \| `FMUTEST` \| `FMUBATCH` | Sondas de temporização e varredura de bytes na Bootrom/FMU via `P7501`. |
| **`PINCHECK`** | `PINCHECK` | Matriz elétrica dos 6 pinos de `P7501` (`Hi-Z`, pull-down, pull-up em `GP26`, contagem de bordas em `GP2`, tempo de subida `XRESET`, guarda em `NSW5V`/`GP3`). |
| **`BUSDIAG` / `SNIFF`** | `BUSDIAG` \| `SNIFF [ms]` | Diagnóstico de atividade de barramento e contagem passiva de transições. |
| **`ISOLATE` / `HIZ`** | `ISOLATE` \| `HIZ` | Desativa drivers TX e coloca os pinos de interface em alta impedância segura (`NSW5V` e `XRESET` em `Hi-Z`, `GP26` em `ADC0`). |
| **`ENGAGE`** | `ENGAGE` | Coloca `GP4` (`UARTM2P_T`) em idle HIGH para transmissão UART ativa, mantendo `NSW5V` (`GP3`) estritamente em `Hi-Z Input`. |
| **`RESET`** | `RESET` | Executa pulso Open-Drain ativo em nível baixo em `XRESET` (`GP5`) e libera para `Hi-Z`. |
| **`HANDSHAKE` / `SCAN` / `BREAK_IN`** | `BREAK_IN [0..3]` \| `SCAN` | Perfis discretos de sequência `SW_UART + XRESET` (`0..3`) e verificação de resposta no barramento. |
| **`DUMP` / `AUTODETECT` / `CFI` / `FLASH_SIZE`** | `DUMP [addr] [words]` \| `FLASH_SIZE [2\|4\|8\|16]` | Leitura de abertura no monitor, detecção fail-loud (`DCSR=0xFFFFFFFF` aborta), consulta CFI `QRY` e seleção de geometria `EEMUL`. |
| **`INJECT_LOADER`** | `INJECT_LOADER CONFIRM` | Injeta micro-driver AM34 (`ram_loader_am34_target.c`) em `0x00004000` (exige token `CONFIRM` e monitor síncrono ativo). |
| **`ERASE_NVRAM`** | `ERASE_NVRAM CONFIRM` | Dispara limpeza JEDEC nos micro-setores NVRAM/EEMUL (exige token `CONFIRM` e acesso ativo ao barramento alvo). |
| **`GDB`** | `GDB` | Alterna o canal USB CDC para o servidor **GDB Remote Serial Protocol (RSP)**. |
| **`LED`** | `LED [IDLE\|ACTIVE\|GDB\|FAULT\|OFF\|AUTO\|STATUS]` | Seleciona padrão visual com `MANUAL_LOCK` (`IDLE`, `ACTIVE`, `GDB`, `FAULT`, `OFF`), devolve ao controle automático (`AUTO`) ou consulta estado atual (`STATUS`). |
| **`BOOTSEL`** | `BOOTSEL` | Reinicia o Pico diretamente no bootloader USB UF2 ROM sem exigir botão físico. |

---

## 7. Exemplos Práticos de Sessão

### Exemplo 1: Inspeção Elétrica Inicial (Modo Seguro sem TX Ativo)
```text
PANADIGA> PING
PONG
PANADIGA> STATUS
VDD: 3.25V (OK)
PANADIGA> PINCHECK
...
PANADIGA> ISOLATE
[BUS] P7501 pins placed in High-Impedance Isolation (NSW5V/XRESET Hi-Z, GP26 ADC0).
```

### Exemplo 2: Captura Multi-Baud PIO1, JIG Temporizado & Controle LED
```text
PANADIGA> LED GDB
[LED] Pico GP25 (Standard Pico / RP2040 / RP2350) | mode=GDB_RSP | lock=MANUAL | pin=1
LED: OK (GDB_RSP, MANUAL_LOCK)

PANADIGA> BAUDSCAN 3500 NORMAL
... edges=2124 min_low_us=8 max_low_us=54 raw_bps=125000 snapped_baud=115200
BAUDSCAN: OK

PANADIGA> HEXCAP 3000 NORMAL
... (348 bytes em RAM sem drop: DDR CHECK: OK .. Loader ver0.61 [06/03/06]) ...
HEXCAP: OK (348 bytes, framing_err=0, overflow=NO)

PANADIGA> PWRTRACE 3000 JIG:350
... GP26_SWUART: min=3.21V max=3.30V | Edges: RXD=2124 NSW5V=0 TXD=0 (final NSW5V=1)
PWRTRACE: OK

PANADIGA> WAITTX 8000 JIG:350 "wait 1" 0D 4000
... [WAITTX] Trigger matched! Injecting payload via Q7514 (SW_UART=Hi-Z)...
WAITTX: OK (triggered=YES, rx=715 bytes)

PANADIGA> LED AUTO
[LED] Pico GP25 (Standard Pico / RP2040 / RP2350) | mode=HEARTBEAT_IDLE | lock=AUTO | pin=1
LED: OK (HEARTBEAT_IDLE, AUTO)
```

> [!NOTE]
> Use preferencialmente os comandos **`PIO1` (`BAUD`, `BAUDSCAN`, `UARTSTREAM`, `HEXCAP`, `PWRTRACE`, `WAITTX`)** em novos testes de bancada. Em janelas com `JIG:350`, o temporizador libera automaticamente `SW_UART` (`GP26 -> ADC0 Hi-Z`) após `350 ms` para que injeções via `Q7514` alcancem `UARTM2P_T` sem interferir na leitura da bootrom. O comando `ISOLATE` continua obrigatório após sessões legadas (`UARTSEND` / `UARTPROBE`).

### Exemplo 3: Injeção / Apagamento com Trava `CONFIRM` (Quando Monitor Ativo)
```text
PANADIGA> INJECT_LOADER CONFIRM
[INJECT_LOADER] Target Address : 0x00004000 (AM34 Internal SRAM)
[INJECT_LOADER] Flash Geometry : 8 MB (NVRAM Window: 0x807F0000..0x807FFFFF)
...
INJECT_LOADER: OK

PANADIGA> ERASE_NVRAM CONFIRM
[ERASE_NVRAM] Launching JEDEC Sector Erase on Spansion Flash (8 MB)
[ERASE_NVRAM] Target Memory Range: 0x807F0000 to 0x807FFFFF (64 KB)
[ERASE_NVRAM] Micro-sectors      : SA134 to SA141 (8 x 8 KB)
...
ERASE_NVRAM: OK
```

*(Observação: Caso o gravador esteja em loop `U99` sem monitor interativo acessível em `P7501`, realize a extração física da Spansion `S29JL064H` [`IC5001`] em programador externo TSOP-48 e aplique `panadiga nvram clean`.)*

---

## 8. Modo Ponte GDB RSP (Depuração Interativa com Ghidra / IDA Pro)

O firmware embarca um servidor **GNU Remote Serial Protocol (RSP)** controlado por [`gdb/gdb_rsp.h`](../../firmware/pico-p7501/gdb/gdb_rsp.h):
* Comando serial: `GDB` (ou autodetecção de pacotes iniciados por `$`).
* Ponte TCP/Serial unificada na CLI Rust:
  ```bash
  panadiga gdb-bridge --port COM10 --listen 127.0.0.1:3333
  # ou:
  panadiga pico gdb-bridge --port COM10 --listen 127.0.0.1:3333
  ```
* Guia detalhado passo a passo para Ghidra, IDA Pro e `gdb-multiarch`:
  * 📖 **[Guia de Depuração Interativa de Hardware: GDB, Ghidra e IDA Pro](GUIA_DEPURACAO_GDB_GHIDRA_IDA.md)**
