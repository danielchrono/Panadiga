# Panasonic P7501 Async UART & Boot Probe Firmware Engine (RP2040 / RP2350)

Firmware oficial do controlador de bancada baseado em **Raspberry Pi Pico (RP2040)** e **Raspberry Pi Pico 2 (RP2350)** para captura de boot, diagnóstico elétrico e comunicação UART assíncrona (**`115200 8N1`**) através do conector de serviço **`P7501` (`B6B-ZR-3R4`, legenda `TO JIGU`)** do gravador **Panasonic DMR-EH55** (processador **Matsushita `AM34` / `MN103S`** + Syscon `IC7501`, placa `VEP79138`).

---

## 1. Visão Geral do Sistema

O firmware modular em [`firmware/pico-p7501/`](../../firmware/pico-p7501/) implementa em hardware e ISO C11 estrito:

1. **Transceptor UART Assíncrono `115200 8N1` (`p7501_uart.pio` & `uart/`):** Recepção passiva determinística via máquina de estados `PIO0` em **`GP2`** (`P7501` Pino 6 `UARTP2M_T` / `MAX TXD`, oversampling de 8 ciclos/bit) e transmissão temporizada em **`GP4`** (`P7501` Pino 3 `UARTM2P_T` / `MAX RXD` via transistor `Q7514`).
2. **Controle Seguro de Reset em Dreno Aberto (`XRESET` / `GP5`):** `GP5` (`P7501` Pino 2) opera exclusivamente alternando entre `0V` (reset ativo) e `Hi-Z` (entrada sem pulls, tempo de subida medido `~20 µs` via `QR7503`), prevenindo curto-circuito contra o supervisor de reset da Panasonic.
3. **Proteção Estrita em `NSW5V` (`GP3` / `P7501` Pino 5) & Controle `SW_UART` (`GP26` / `ADC0`, Pino 1):**
   * **`GP3` (`NSW5V` / `X_SW_5R2V`):** Provém de `IC1506` (`~5.15V` upstream via `D7510`, repouso em `~3.29V` nos resistores `R7570`/`R7572`). **PERMANECE SEMPRE como `Hi-Z Input` sem pull-up/pull-down** e nunca é dirigido em push-pull.
   * **`GP26` (`SW_UART`):** Amostra a tensão via `ADC0` (`~3.25V` em Hi-Z, `~2.69V` sob pull-down) e força nível `LOW` apenas para entrar em modo serial boot (`XFWE`), restaurando `Hi-Z` quando TX em `GP4` precisa cruzar `Q7514`.
4. **Console Serial USB CDC + BLE Opcional (`PANADIGA> `):** Interface interativa a `115200 bps` compatível com terminais padrão (PuTTY, TeraTerm), `panadiga` CLI e aplicativo desktop/Android Tauri (`panadiga-desktop` / APK). Placas `pico_w` e `pico2_w` habilitam também ponte Bluetooth LE Nordic UART Service (`PANADIGA_BLE_ENABLED=1`).
5. **Ponte GDB RSP & Injeção/Sanitização Condicional:** Inclui servidor **GNU Remote Serial Protocol (RSP)** (`GDB`), além de primitivas `INJECT_LOADER CONFIRM` / `ERASE_NVRAM CONFIRM` (com trava de segurança obrigatória) voltadas para cenários em que haja acesso ao monitor/memória do alvo; para chips travados fora do monitor sem dessoldagem, a rota primária confirmada de gravação da Spansion `S29JL064H` (`IC5001`) é programador externo (`SVOD 4` / `XGecu T48`) + `panadiga nvram clean`.

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
A esteira em [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) e a validação local (`python scripts/validate.py` / `npm run validate`) executam verificações em Rust, TypeScript, C11 host tests (`firmware/pico-p7501/tests/test_gdb_rsp_host.c`), Python (`mypy --strict`, `pytest`), `jscpd`, Vitest e Playwright.

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

Ao conectar ou pressionar `<ENTER>`, o console exibe o banner oficial de [`p7501_cli.c`](../../firmware/pico-p7501/cli/p7501_cli.c):

```text
====================================================================
   PANADIGA: Panasonic DMR-EH55 P7501 Async UART & Boot Bench Probe
   MCU Host : RP2040 (Pico / ARM Cortex-M0+)
   Bus      : P7501 (B6B-ZR-3R4) 115200 8N1 + SW_UART / XRESET / NSW5V(Hi-Z)
   [GDB-RSP] Active Server Mode (RP2040 8 KB Cache)
   Type HELP for available P7501 UART, FMU, diagnostic, Flash & GDB commands.
====================================================================

PANADIGA> 
```

---

## 6. Referência Completa de Comandos (`p7501_cli.c`)

O firmware atende tanto operadores humanos quanto automação em Rust (`panadiga`) e a UI reativa Tauri:

| Comando | Sintaxe | Função Operacional Verificada |
| :--- | :--- | :--- |
| **`PING`** | `PING` | Responde `PONG` para verificação rápida de conectividade. |
| **`STATUS` / `VDD`** | `STATUS` | Amostra `GP26 (ADC0)` em `P7501` Pino 1 (`SW_UART`) e imprime `VDD: X.XXV (OK / LOW / FLOATING)`. |
| **`BANNER`** / **`HELP`** | `HELP` \| `?` | Imprime banner de identificação ou lista resumida de comandos. |
| **`BOOTCAP`** | `BOOTCAP [ms]` | Reseta `XRESET` em Open-Drain com `SW_UART` liberado e captura stream UART `115200 8N1` no boot. |
| **`UARTMON`** | `UARTMON [ms]` | Escuta passivamente `GP2` (`UARTP2M_T`) sem forçar reset ou `SW_UART`. |
| **`JIGBOOT`** | `JIGBOOT [ms]` | Executa boot com `SW_UART = LOW` (`XFWE` / modo JIG) após pulso `XRESET` e captura `GP2` a `115200 8N1`. |
| **`UARTSEND`** | `UARTSEND [LOW:]<texto>` | Transmite comandos ASCII/bytes pela linha `GP4` (`UARTM2P_T`) em `115200 8N1` (`Hi-Z` padrão permite passagem por `Q7514`; prefixo `LOW:` força `SW_UART` baixo). |
| **`UARTPROBE`** | `UARTPROBE [sec]` | Executa varredura serial em `P7501` e restaura `GP26` ao estado `ADC0 Hi-Z` ao término. |
| **`FMUSCAN` / `FMUTEST` / `FMUBATCH`** | `FMUSCAN` \| `FMUTEST` \| `FMUBATCH` | Sondas de temporização e varredura de bytes na Bootrom/FMU via `P7501`. |
| **`PINCHECK`** | `PINCHECK` | Matriz elétrica dos 6 pinos de `P7501` (`Hi-Z`, pull-down, pull-up em `GP26`, contagem de bordas em `GP2`, guarda em `NSW5V`/`GP3`). |
| **`BUSDIAG` / `SNIFF`** | `BUSDIAG` \| `SNIFF [ms]` | Diagnóstico de atividade de barramento e contagem passiva de transições. |
| **`ISOLATE` / `HIZ`** | `ISOLATE` \| `HIZ` | Desativa drivers TX e coloca os pinos de interface em alta impedância segura (`NSW5V` e `XRESET` em `Hi-Z`, `GP26` em `ADC0`). |
| **`ENGAGE`** | `ENGAGE` | Coloca `GP4` (`UARTM2P_T`) em idle HIGH para transmissão UART ativa, mantendo `NSW5V` (`GP3`) estritamente em `Hi-Z Input`. |
| **`RESET`** | `RESET` | Executa pulso Open-Drain ativo em nível baixo em `XRESET` (`GP5`) e libera para `Hi-Z`. |
| **`HANDSHAKE` / `SCAN` / `BREAK_IN`** | `BREAK_IN [0..3]` \| `BOOT_PROFILE` | Perfis discretos de sequência `SW_UART + XRESET` (`0..3`) e verificação de resposta no barramento. |
| **`DUMP` / `AUTODETECT` / `CFI` / `FLASH_SIZE`** | `DUMP [addr] [words]` \| `FLASH_SIZE [2\|4\|8\|16]` | Abertura de leitura no monitor/barramento, detecção por aliasing de endereço, consulta CFI `QRY` e seleção de geometria `EEMUL`. |
| **`INJECT_LOADER`** | `INJECT_LOADER CONFIRM` | Injeta micro-driver AM34 (`ram_loader_am34_target.c`) em `0x00004000` (exige token `CONFIRM` e monitor/escrita em SRAM ativos). |
| **`ERASE_NVRAM`** | `ERASE_NVRAM CONFIRM` | Dispara limpeza JEDEC nos micro-setores NVRAM/EEMUL (exige token `CONFIRM` e acesso ativo ao barramento alvo). |
| **`GDB`** | `GDB` | Alterna o canal USB CDC para o servidor **GDB Remote Serial Protocol (RSP)**. |
| **`LED` / `BOOTSEL`** | `LED <ACTIVE\|GDB\|FAULT\|IDLE>` \| `BOOTSEL` | Controla estado visual do LED onboard ou reinicia o Pico em bootloader UF2. |

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

### Exemplo 2: Captura de Boot JIG (`115200 8N1`) & Transmissão Interativa
```text
PANADIGA> JIGBOOT
... captura <00>DDR CHECK: OK e loop CD diag boot mode em GP2 (115200 8N1) ...

PANADIGA> ENGAGE
[BUS] P7501 UART TX (GP4) engaged idle HIGH; NSW5V (GP3) kept Hi-Z Input.
PANADIGA> UARTSEND VER
... (SW_UART mantido em Hi-Z para que Q7514 conduza UARTM2P_T) ...

PANADIGA> UARTPROBE 8
PANADIGA> ISOLATE
[BUS] P7501 pins placed in High-Impedance Isolation (NSW5V/XRESET Hi-Z, GP26 ADC0).
```

> [!NOTE]
> Se comandos como `UARTSEND`, `JIGBOOT` ou `UARTMON` deixarem leitura transitória baixa em `GP26`, executar `UARTPROBE 8` seguido de `ISOLATE` restaura `GP26` para leitura limpa (`~2.70V OK` em `STATUS` e `3.25V | 2.69V | 3.27V` no `PINCHECK`).

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
