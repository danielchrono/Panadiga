# Panasonic DWire32A / OCD Firmware Engine for Raspberry Pi Pico (RP2040)

Firmware oficial do controlador de bancada baseado em **Raspberry Pi Pico (RP2040)** para interfaceamento síncrono proprietário **Matsushita/Panasonic DWire32A** com o processador de mídia **Panasonic MN2DS0016 (UniPhier / AM34)** presente no gravador de DVD/HDD **Panasonic DMR-EH55** (placa digital `VEP79138`).

---

## 1. Visão Geral do Sistema

O firmware implementa em nível de silício:
1. **Transceptor PIO Síncrono Half-Duplex (1.0 MHz):** Controle determinístico sem jitter de tempo para reversão de direção de barramento na linha `SDATA`.
2. **Controle de Reset Seguro em Dreno Aberto (*Open-Drain*):** GP5 opera exclusivamente alternando entre `0V` (nível baixo ativo) e `Hi-Z` (alta impedância), prevenindo curto-circuito contra o circuito supervisor de reset `XRST` da Panasonic.
3. **Monitoramento Analógico de Tensão VDD:** Leitura de referência da placa via canal ADC0 (GP26) antes de qualquer injeção de clock ou comando.
4. **Console Serial Interativo USB CDC & Protocolo de Comandos:** Comunicação bidirecional compatível com terminais seriais (PuTTY, TeraTerm) e com o aplicativo desktop Tauri da suíte de engenharia.
5. **Injetor e Executor do RAM Loader AM34:** Injeção do micro-código nativo de posição independente na SRAM interna (`0x00004000`) e disparo da rotina JEDEC de apagamento dos 8 micro-setores da partição `EEMUL` (`0x007F0000` a `0x007FFFFF`), recuperando o aparelho do erro fatal `U99` sem necessidade de dessoldar a memória Flash TSOP-48.

---

## 2. Requisitos de Ferramentas de Compilação

Para compilar o firmware a partir do código-fonte:
* **CMake** $\ge$ 3.13 (disponível no PATH)
* **Compilador ARM GCC:** `arm-none-eabi-gcc` $\ge$ 10.3
* **Build System:** `Ninja` (recomendado) ou `MinGW Makefiles`
* **Raspberry Pi Pico SDK:** $\ge$ 1.5.0 (variável de ambiente `PICO_SDK_PATH` configurada, ou fetch automático via Git)

---

## 3. Como Compilar o Firmware Multi-Plataforma (RP2040 e RP2350)

O subsistema possui suporte nativo dual-target para **RP2040 (Pico 1)** e **RP2350 (Pico 2)**, com scripts de automação para Windows, Linux e macOS:

### Opção A: Linux / macOS / WSL (Bash POSIX)
```bash
cd dwire32a_jtag_research
chmod +x build_firmware.sh

# Compilar para ambos os targets (RP2040 e RP2350):
./build_firmware.sh all

# Ou compilar apenas para uma placa específica:
./build_firmware.sh pico    # RP2040
./build_firmware.sh pico2   # RP2350
```

### Opção B: Windows (PowerShell)
```powershell
cd dwire32a_jtag_research

# Compila ambos os alvos (padrão):
.\build_pico_firmware.ps1 -Board all

# Ou compila alvo individual:
.\build_pico_firmware.ps1 -Board pico     # RP2040
.\build_pico_firmware.ps1 -Board pico2    # RP2350
.\build_pico_firmware.ps1 -Clean -Board all
```

### Opção C: Windows (Prompt CMD Clássico)
```cmd
cd dwire32a_jtag_research
build_pico_firmware.cmd all
build_pico_firmware.cmd pico
build_pico_firmware.cmd pico2
```

### Artefatos Binários Gerados
A compilação produz artefatos UF2 separados e auto-contidos:
* **`panasonic_dwire32a_rp2040.uf2`** $\leftarrow$ Imagem para Raspberry Pi Pico / Pico W, Arduino Nano RP2040, XIAO RP2040 (264 KB SRAM).
* **`panasonic_dwire32a_rp2350.uf2`** $\leftarrow$ Imagem para Raspberry Pi Pico 2 (Dual ARM Cortex-M33 / Hazard3 RISC-V, 520 KB SRAM com 64 KB de cache local de setor).
* Imagens de depuração com símbolos (`.elf`) e mapas de memória (`.map`) em `build_pico/` e `build_pico2/`.

### Integração Contínua (CI/CD)
O repositório inclui esteira automatizada no GitHub Actions (`.github/workflows/ci.yml`), que valida a cada commit ou pull request:
1. Compilação e testes de regressão Rust no Linux, Windows e macOS;
2. Lint e formatação de código (`clippy`, `rustfmt`);
3. Build e checagem de tipos da aplicação desktop (`apps/desktop` com React/Vite);
4. Compilação cruzada dos dois firmwares com `arm-none-eabi-gcc` e upload automático dos arquivos `.uf2` como artefatos.
* `panasonic_dwire32a.bin` $\leftarrow$ Binário cru para gravadores SWD (Picoprobe/J-Link)
* `panasonic_dwire32a.hex` $\leftarrow$ Formato Intel Hex

---

## 4. Como Gravar o Arquivo `.uf2` no Raspberry Pi Pico

O microcontrolador RP2040 possui um bootloader USB nativo em ROM (UF2 Bootloader):

```text
       GRAVAÇÃO DRAG & DROP NO RASPBERRY PI PICO
  ┌────────────────────────────────────────────────────────┐
  │  1. Desconecte o cabo USB do Pico                      │
  │  2. Mantenha pressionado o botão branco BOOTSEL        │
  │  3. Conecte o cabo USB ao computador                   │
  │  4. Solte o botão BOOTSEL                              │
  │  5. O volume "RPI-RP2" aparecerá no Windows Explorer   │
  │  6. Arraste "panasonic_dwire32a.uf2" para "RPI-RP2"    │
  │  7. O Pico piscará e reiniciará instantaneamente!      │
  └────────────────────────────────────────────────────────┘
```

1. **Desconecte** o Raspberry Pi Pico da porta USB.
2. Pressione e segure o **botão branco BOOTSEL** localizado na placa do Pico.
3. Enquanto mantém o botão pressionado, **conecte o cabo USB** ao PC.
4. Após conectar o cabo, **solte o botão BOOTSEL**.
5. O Windows reconhecerá o Pico como um drive de armazenamento USB removível com o nome **`RPI-RP2`**.
6. Arraste e solte (ou copie e cole) o arquivo **`build/panasonic_dwire32a.uf2`** para dentro da unidade **`RPI-RP2`**.
7. O Pico processará o firmware, a unidade USB se desconectará automaticamente e o Pico reiniciará rodando o firmware DWire32A!

---

## 5. Conexão Serial USB CDC (Terminal & Tauri)

Após a reinicialização, o Pico se enumera como um dispositivo de comunicação serial USB:
* No Windows: Acesse o **Gerenciador de Dispositivos** $\rightarrow$ **Portas (COM e LPT)** e verifique a porta atribuída (exemplo: `COM3`, `COM4` ou `COM7`).
* Abra um emulador de terminal serial (PuTTY, TeraTerm ou VS Code Serial Monitor):
  * **Porta:** A porta COM detectada
  * **Baud Rate:** `115200` (o protocolo USB CDC opera na velocidade nativa do barramento USB 12 Mbps)
  * **Configuração:** 8 bits de dados, 1 stop bit, sem paridade (8N1), sem controle de fluxo.

Ao conectar ou enviar uma quebra de linha, o console exibirá o banner do sistema:

```text
====================================================================
   PANASONIC DWIRE32A / OCD INTERFACE RECONSTRUCTION ENGINE
   Architecture : Matsushita AM34 / MN103S (UniPhier 32-bit RISC)
   Target SoC   : MN2DS0016 (Panasonic DMR-EH55 Digital Board VEP79138)
   Flash Memory : Spansion S29JL064H70TFI00 (64 Mbit / 8 MB TSOP-48)
   Host Control : Raspberry Pi Pico (RP2040) via PIO0 Transceiver
   C Standard   : ISO C11 Embedded System Architecture
====================================================================
Type HELP for a list of available serial commands.

DWIRE32A> 
```

---

## 6. Referência Completa de Comandos

O firmware responde tanto a usuários humanos digitando no console quanto a chamadas automatizadas da interface gráfica Tauri.

| Comando | Sintaxe | Resposta do Firmware | Finalidade Operacional |
| :--- | :--- | :--- | :--- |
| **`PING`** | `PING` | `PONG` | Teste de batimento cardíaco (*heartbeat*) e presença do Pico |
| **`STATUS` / `VDD`**| `STATUS` | `VDD: 3.31V (OK)` | Mede o pino de sense analógico (GP26) para validar que a placa alvo está ligada |
| **`BREAK_IN`** | `BREAK_IN [0..3]` | `BREAK_IN: OK (Mode 0)` | Executa 32 pulsos de SCLK sob reset para congelar a CPU AM34 em OCD Halt |
| **`SCAN`** | `SCAN` | `SCAN: COMPLETED (Hits: X, Probes: Y)` | Realiza varredura de opcodes (0x00 a 0xFF) nos 4 modos temporais |
| **`INJECT_LOADER`**| `INJECT_LOADER` | `INJECT_LOADER: OK (80 bytes written, 20 words verified)` | Grava o RAM Loader AM34 na SRAM interna em `0x00004000` via PIO |
| **`ERASE_NVRAM`** | `ERASE_NVRAM` | `ERASE_NVRAM: OK (8 micro-sectors 0x007F0000..0x007FFFFF erased, U99 cleared)` | Apaga os 8 micro-setores `SA134`..`SA141` da Spansion eliminando o erro U99 |
| **`HELP`** | `HELP` | *(Menu descritivo)* | Lista todos os comandos disponíveis |

---

## 7. Exemplos Práticos de Sessão

### Exemplo 1: Verificação de Conexão e Alimentação
```text
DWIRE32A> PING
PONG
DWIRE32A> VDD
VDD: 3.31V (OK)
```

### Exemplo 2: Captura do Processador no Reset (Cold Break-In)
```text
DWIRE32A> BREAK_IN 0
[BREAK_IN] Executing Cold Break-In mode 0 (EXTRG High, Pre-Reset)...
BREAK_IN: OK (Mode 0)
```

### Exemplo 3: Recuperação Completa U99 (Injeção + Sanitização)
```text
DWIRE32A> INJECT_LOADER
[INJECT_LOADER] Initiating SRAM transfer via DWire32A...
[INJECT_LOADER] Target Address : 0x00004000 (AM34 Internal SRAM)
[INJECT_LOADER] Payload Size   : 80 bytes (20 x 32-bit words)
[INJECT_LOADER] Binary CRC32   : 0xCCB0151A (MATCH)
[INJECT_LOADER] Binary Sum32   : 0xFD457722 (MATCH)
[INJECT_LOADER] Streaming payload via DWire32A PIO state machine...
[INJECT_LOADER] Successfully injected 80 bytes into SRAM.
INJECT_LOADER: OK (80 bytes written, 20 words verified)

DWIRE32A> ERASE_NVRAM
[ERASE_NVRAM] Initiating Spansion flash NVRAM sector erasure...
[ERASE_NVRAM] ====================================================
[ERASE_NVRAM] Launching JEDEC Sector Erase on Spansion S29JL064H
[ERASE_NVRAM] Target Memory Range: 0x807F0000 to 0x807FFFFF (64 KB)
[ERASE_NVRAM] Micro-sectors      : SA134 to SA141 (8 x 8 KB)
[ERASE_NVRAM] Note               : Parameter block 0x007E0000 preserved
[ERASE_NVRAM] ====================================================
[ERASE_NVRAM] Loading AM34 PC <- 0x00004000...
[ERASE_NVRAM] Resuming AM34 core execution...
[ERASE_NVRAM] Erasing SA134 (Offset 0x7F0000, 8 KB)... [OK]
[ERASE_NVRAM] Erasing SA135 (Offset 0x7F2000, 8 KB)... [OK]
[ERASE_NVRAM] Erasing SA136 (Offset 0x7F4000, 8 KB)... [OK]
[ERASE_NVRAM] Erasing SA137 (Offset 0x7F6000, 8 KB)... [OK]
[ERASE_NVRAM] Erasing SA138 (Offset 0x7F8000, 8 KB)... [OK]
[ERASE_NVRAM] Erasing SA139 (Offset 0x7FA000, 8 KB)... [OK]
[ERASE_NVRAM] Erasing SA140 (Offset 0x7FC000, 8 KB)... [OK]
[ERASE_NVRAM] Erasing SA141 (Offset 0x7FE000, 8 KB)... [OK]
[ERASE_NVRAM] Polling for CPU OCD breakpoint ('pi' trap)...
[ERASE_NVRAM] Success: AM34 CPU trapped at breakpoint 'pi'!
[ERASE_NVRAM] Spansion array restored to normal Read Mode.
[ERASE_NVRAM] NVRAM sanitization complete: U99 condition resolved.
ERASE_NVRAM: OK (8 micro-sectors 0x007F0000..0x007FFFFF erased, U99 cleared)
```

---

## 7. Modo Ponte GDB RSP (Depuração Interativa com Ghidra / IDA Pro)

O firmware implementa um servidor nativo do **GNU Remote Serial Protocol (RSP)** para conexão de depuradores de alto nível:
* Comando terminal: `GDB` (ou auto-chaveamento ao receber `$`).
* Protocolo completo de registradores AM34 (`D0..D3`, `A0..A3`, `PC`, `SP`, etc.).
* Caching de alta performance: 64 KB no RP2350 (Pico 2) e 8 KB no RP2040.
* Ponte TCP automatizada no terminal:
  ```bash
  panadiga pico gdb-bridge --port COM3 --listen 127.0.0.1:3333
  ```

Para o tutorial completo de como conectar o **Ghidra**, **IDA Pro**, **gdb-multiarch** e **OpenOCD** passo a passo, consulte:
* 📖 **[Guia de Depuração Interativa de Hardware: GDB, Ghidra e IDA Pro](GUIA_DEPURACAO_GDB_GHIDRA_IDA.md)**

