# Relatório 100% Fiel da Sessão de Engenharia Reversa e Depuração ao Vivo (`DMR-EH55` / `COM10` / `P7501`)

> **Período Documentado:** `2026-11-06` a `2026-11-13` (Etapas `1`–`50` de análise estática + capturas ao vivo via Pico RP2040 em `COM10` + auditoria elétrica CAD dos manuais Panasonic).  
> **Escopo:** Registro rigoroso e cronológico de **tudo o que foi testado, medido, provado e descartado** durante a sessão real na bancada e nos binários, diferenciando explicitamente hardware ao vivo (`DMR-EH55` / `Loader ver0.61 [06/03/06]`) da imagem de referência estática (`DMR-EH58` / `Loader ver0.63 [07/10/24]` em `PANA_DVD.FRM`).

---

## 1. Resumo Executivo dos Vereditos Finais Provados

1. **Protocolo Real de `P7501 (TO WRITE JIGU)` no `DMR-EH55`:**  
   `P7501` **NÃO é `DWire32A` / JTAG síncrono de debug (`CN55001`)**. Trata-se de uma porta **UART Assíncrona (`115200 8N1`) + Controle de Boot/Reset discreto**, confirmada ao mesmo tempo pelos manuais oficiais (`DMR-EH55P/PC North America`, `EC/EB/EF Vol. 1 & 2`, `EC/EE`) e por telemetria elétrica/lógica em `COM10`.
2. **Duas Falhas Independentes Confirmadas na Placa em Bancada (`U99`):**
   - **Falha 1 (Main PCB — Spansion NOR / `[nfrom]`/`BGO`):** No `***** normal mode *****`, o bootloader chama `BGO driver init` (`0x4400836f`) **antes** de acessar o HDD, recebe retorno `1` de `0x4401e636`, imprime **`BGO driver init failed[1].`** e salta o carregamento de `main soft` no HDD (`0x440065bc`), desviando direto para recuperação de emergência (`0x44007812`) — causando `Please wait` $\rightarrow$ `U99` no painel frontal.
   - **Falha 2 (Drive Óptico `MATSHITA R6-MV4` — Laser `0.5 mW`):** Em `CD diag boot mode` (ou fallback de `normal mode`), após ticks `59..53` com `X(80020401)` (*Logical Unit in process of becoming ready*), o drive falha no tick `52` com **`[RTSD] === DRIVE ACCESS ERROR === ( 80040904 )`** e **`SENSE[70 00 04 00 00 00 00 0A 00 00 83 A7 09 04 00 00 00 00]`**, zerando em `Drive start failed[-2147219196].` seguido de reset frio. A tabela Shift-JIS de `DRV_decomp.bin` (`0x400e8c1e`, stride `0x34`) prova que **`0x83A7`** é **`0.5mW発光ができない(HFｵﾝ密調整)`** (*Incapaz de emitir 0,5 mW de laser durante ajuste fino HF ON*), **não** uma flag de erro IDE gravada em NVRAM.
3. **Contorno do Bloqueio `BGO` via `Pino 1 (SW_UART = LOW)`:**  
   Puxar `P7501 Pino 1 (SW_UART / GP26)` para `LOW` durante `XRESET` aciona **`***** serial boot mode *****`**, que **ignora o erro `BGO`**, inicializa imediatamente **`*** timer correspond initialize ***`** (`startup reason = [70]`, `Main hung-up time changed 255[s].`) e executa a cadeia de recuperação (`fmu` $\rightarrow$ `SD init start...` $\rightarrow$ `*** CD diag boot mode ***`).

---

## 2. Cronologia Fiel da Sessão (Do Código Inicial à Bancada Real)

### Fase A — Reestruturação de Engenharia e Pipeline CI (2026-11-06 a 2026-11-11)
* Consolidação do repositório em `C:\Users\danie\Documents\Panadiga`, eliminando caminhos hardcoded antigos, unificando CLI e Desktop em `panadiga.exe` com carregamento transparente `.gz`, arquitetura MVVM/FP em Rust + React/TypeScript e **9 Gates obrigatórios** em `scripts/validate.py` (Rust `fmt/check/test`, `tsc`, Vitest, `jscpd 0%`, `uxlint`, Playwright, Ghidra export, host C11 `-Werror`/`/WX`, e Python `py_compile`/`mypy --strict`/`pytest`).
* Correções críticas de inicialização USB CDC no Pico (`DTR` + timeout/prompt framing) até estabelecer conexão estável em `COM10` (`VID:2E8A PID:000A` $\rightarrow$ `PING PONG`).

### Fase B — Descoberta do Mock Sintético e Falha do Adaptador Serial Passivo (2026-11-12)
* **Auditoria forense de `panasonic_dmr-eh55_flash_dump.bin` (`8.388.608 B`):** Comprovou-se em entropia/bytes que `8.388.509 B` (`99,999%`) eram `0xFF`, restando apenas `99` bytes sintéticos (`PANASONIC DMR-EH55 AM34 IPL BOOTLOADER VEP79138` e fixture `55 AA 10 00...` para testes Rust). A análise estática de código real passou imediatamente para o pacote `PANA_DVD.FRM` extraído de `DVDRAM.7z` (`DMR-EH58`, `BOOT.BIN` `ver0.63` + `DRV.BIN`).
* **Por que `FT232RL` e Arduino (sem ATmega) falharam antes do Pico:** Conversores USB-Serial passivos de 3 fios (`TX/RX/GND`) não controlam `Pino 1 (SW_UART)` nem o reset `Pino 2 (XRESET)` no timing necessário para chavear `XFWE` rumo à Digital PCB `VEP79138K`.
* **Por que `HANDSHAKE` síncrono `DWire32A` retornou `DCSR: 0xFFFFFFFF`:** No chicote plugado em `P7501`, `GP3` estava ligado em `Pino 5 (NSW5V)` (alimentação estática de pull-up, **não `SCLK`**) e `GP4` em `Pino 3 (MAX RXD)` (pull-up idle HIGH via `R7570`, **não `SDATA` bidirecional**).

### Fase C — Inspeção Elétrica Física (`PINCHECK` / `BUSDIAG`) e Manuais CAD (2026-11-12 a 2026-11-13)
* **Orientação física medida na placa:** O usuário identificou o conector **`P7501` (`B6B-ZR-3R4`, 6 pinos, passo `1.5 mm`)** espremido entre os flats do drive óptico **`FP50003`** (10/11 vias, Spindle Motor, à esquerda) e **`FP50004`** (8 vias, Loading Motor + `OPEN_SW`/`CLOSE_SW`, à direita) — ver [page_113.png](../assets/images/re_dmr_eh55/page_113.png) e [fp50004_zoom.png](../assets/images/re_dmr_eh55/fp50004_zoom.png).
* **Medição com multímetro pelo usuário:** Em todos os pinos exceto `GND`, o multímetro acusou **`~3.3V` em repouso**, aparentemente contradizendo a leitura inicial de rótulo `NSW5V (5.15V)`. A inspeção vetorial CAD em [`na_p62_zoom.png`](../assets/images/re_dmr_eh55/na_p62_zoom.png), [`na_p64_topleft.png`](../assets/images/re_dmr_eh55/na_p64_topleft.png), [`crop_p55_IC1506_zoom.png`](../assets/images/re_dmr_eh55/crop_p55_IC1506_zoom.png) e [`crop_p64_Q7510_Q7514_QR7503.png`](../assets/images/re_dmr_eh55/crop_p64_Q7510_Q7514_QR7503.png) reconciliou 100% dos números:

| Pino `P7501` | Sinal no Esquemático Panasonic | GPIO Pico (`COM10`) | Circuito Real Verificado (CAD `NA p.55/62/63/64/83`) | Medição Empírica (`PINCHECK` / `BUSDIAG` / `COM10`) | Regra de Segurança |
| :---: | :--- | :---: | :--- | :--- | :--- |
| **1** | `SW_UART` (`Linha D`) | `GP26 / ADC0` | Coletor de **`Q7510 (2SD0601A0L)`** + Base de **`QR7503 (UNR511300L)`** + pino 80 (`SW_UART`) de `IC7501`. Base de `Q7510` em `3.30V` (`R7562 4.7 kΩ`) e Emissor (`XFWE`) em `3.30V` (`R7559 10 kΩ`). | `Hi-Z: 3.24–3.25V`, **`PD: 2.68–2.69V`** ($3.30\text{V} - V_{BC}$), `PU: 3.26–3.27V`. | `LOW` força `serial boot`; **`Hi-Z` habilita TX via `Q7514`**. |
| **2** | `XRESET` (`Linha E`) | `GP5` | Malha de reset `XRESET` via `QR7503`. | `STRONG HIGH`; subida `Hi-Z` medida em **`20 µs`**. | Controlar exclusivamente em **Open-Drain** (`LOW` / `Hi-Z`). |
| **3** | `UARTM2P_T` (`MAX RXD`) | `GP4` (TX Pico) | Pull-up `R7570 (10 kΩ)` para `NSW5V`; entra no Coletor de **`Q7514`** rumo a `UARTM2P` (pino RX na Digital PCB `P59001`). | **`0` transições em `BUSDIAG`**; isolado fisicamente do SoC quando `SW_UART = LOW` (pois `Q7510` corta `Q7514` via `R7528 4.7 kΩ`). | Enviar comandos TX com **`SW_UART = Hi-Z`** (`Q7514` em condução). |
| **4** | `GND` | `GND` | Referência `0V`. *(Obs: em versões preliminares discutiu-se pino 3 pela contagem invertida; no conector oficial `B6B-ZR-3R4` p.62 é Pino 4).* | `0.00V` | Terra comum obrigatório. |
| **5** | `NSW5V` (`X_SW_5R2V`, `Linha C`) | `GP3` | Gerado por `IC1506 (SI-3011ZFE, ~5.15V)` via `D7510` rumo a `R7570/R7572 (10 kΩ)` e `IC7501 VDD` (em OR com `B7501`); em standby/corte varia conforme rail ativo. | **`0` transições em `BUSDIAG`** (`NÃO é SCLK`); leitura segura ~`3.29V` em Hi-Z. | **SEMPRE `Hi-Z Input` sem pulls — NUNCA dirigir push-pull!** |
| **6** | `UARTP2M_T` (`MAX TXD`) | `GP2` (RX Pico) | Saída `UARTP2M` do SoC AM34 via `P59001` com `R7572 (10 kΩ)`. | **Única linha UART ativa:** `872` / `1006` transições em `250 ms`; **`2124`** (`normal`) / **`3308`** (`JIG`) em `0..1.3 s` @ `115200 8N1`. | Leitura `Hi-Z` + pull-up interno fraco. |

* **Bug Descoberto no Firmware do Pico (`COM10`):**  
  Após rodar `JIGBOOT`, `UARTMON` ou `UARTSEND`, o comando subsequente `ISOLATE` deixava `GP26` (`VDD_ADC`) configurado em SIO/saída LOW, fazendo `STATUS` reportar **`VDD: 0.02V (LOW)`** mesmo com a placa alimentada. Na bancada, executar **`UARTPROBE 8` seguido de `ISOLATE`** restaurava `adc_gpio_init(GP26)` $\rightarrow$ `VDD: 2.70V (OK)` (`PINCHECK: 3.25V | 2.69V | 3.27V`). O bug foi corrigido definitivamente no código C11 em `firmware/pico-p7501/hal/p7501_hw.c` (`p7501_hw_read_vdd` e `p7501_hw_isolate_bus_hiz`).

---

## 3. Matriz Completa das 10 Capturas Reais em `COM10` (`DMR-EH55`, `Loader ver0.61`)

Todas as capturas abaixo foram preservadas em `scratch/` e decodificadas integralmente pelo módulo `scripts/bench/decoders.py`:

| Arquivo de Log | Tamanho / Encoding | Modo / Caminho Observado | Erros / Marcos Registrados Ao Vivo | Conclusão Experimental |
| :--- | :---: | :--- | :--- | :--- |
| `normal_boot_90s.log` | `24.362 B` (UTF-16LE) | `***** normal mode *****` | `DDR CHECK: OK`, `Chip select = 1`, **`BGO driver init failed[1].`**, fallback `CD diag` (`0x80020401` $\rightarrow$ `80040904` / `SENSE 83 A7 09 04`). | Prova que o `normal mode` aborta antes de ler o HDD por falha `[nfrom]`/`BGO`. |
| `bootcap_cover_shut_live.log` | `4.568 B` (UTF-8) | `***** normal mode *****` (tampa parafusada) | Mesmo caminho: `BGO driver init failed[1].`, `** can not close tray **`, `80040904` (`83 A7`). | Provou que parafusar a tampa com ímã não elimina `can not close tray` / `80040904`. |
| `jigboot_20s.log` | `2.026 B` (UTF-8) | `***** serial boot mode *****` | Pula `BGO`, entra em `!!! start fmu to read & exec fvu. !!!`, `wait 1 second.`, **`startup reason = [70]`**, `SD init failed[-90001]`. | Confirma destravamento de `IC7501` (`timer correspond initialize`). |
| `jigboot_70s.log` | `4.157 B` (UTF-8) | `***** serial boot mode *****` | Chega ao fim da contagem `59..0` em `*** CD diag boot mode ***`: `X(80040904)0,Drive start failed[-2147219196].` + reset frio. | Mapeou o ciclo completo de 60 segundos do timeout ATAPI até o reboot. |
| `jigboot_85s_live.log` | `13.353 B` (UTF-8) | `***** serial boot mode *****` | Sequência completa: `SD -90001` $\rightarrow$ `F/E Address = 0x90202000` $\rightarrow$ `X(80062900)X(80023a00)` $\rightarrow$ `80040904` (`83 A7`) $\rightarrow$ reinício `<00>DDR CHECK: OK`. | Evidência principal do ciclo contínuo `JIGBOOT`. |
| `sd_fmu_attempt1.log` | `3.019 B` (UTF-8) | `serial boot` + SD Fujifilm 2GB (`MMAGF02GWECA-MB`) | `[SDCD]SetBusWidth:SD Status ERROR -91006`, `SD init failed[-90002]`, segue para `80040904` (`83 A7`). | Slot detecta cartão e avança além de `-90001`, falhando em `ACMD13`/`SetBusWidth`. |
| `sd_fmu_attempt2.log` | `2.830 B` (UTF-8) | `serial boot` + SD regravado FAT16 32KB (`FVU`/`PANA_DVD.FRM`) | Mesmo erro **`[SDCD]SetBusWidth:SD Status ERROR -91006`** / `-90002`. | Justificou desmontar `GetSD_Status` (`0x44027826`) no `BOOT.BIN`. |
| `no_sd_flat_boot.log` | `2.884 B` (UTF-8) | `serial boot` sem cabo flat do leitor SD | Erro `-91006`/`-90002` persiste no slot controlador e `80040904` (`83 A7`) idêntico. | Descarta curto ou bloqueio oriundo do flat do painel/SD. |
| `no_ide_cable_boot.log` | `3.019 B` (UTF-8) | `serial boot` **sem cabo flat IDE** (StarTech/HDD removidos) | Idêntico (`80040904`, `SENSE 83 A7 09 04`). | **Refutou** hipótese de erro injetado em runtime pelo adaptador SATA/IDE no boot. |
| `no_molex_boot.log` | `2.732 B` (UTF-8) | `serial boot` **sem alimentação Molex do HDD** | Idêntico (`80040904`, `SENSE 83 A7 09 04`). | Isola o `80040904` estritamente ao subsistema óptico (`DRV`/OPU). |

---

## 4. Achados Estáticos Verificados em `BOOT.BIN` e `DRV_decomp.bin` (`Steps 1`–`50`)

* **Regra ISA `MN103S` provada empiricamente (`step41_verify_mn103_rel.py`):** Em `call16 (0xCD)`, `jmp16 (0xCC)`, `call32 (0xDD)`, `jmp32 (0xDC)` e `bra8 (0xCA)`, o registrador `PC` base é o **endereço da instrução atual** (`target = instruction_va + disp`), provado em `0x44004961 -> 0x44004808 (printf)`, `0x44009cb7 -> 0x44009b11`, e `0x4400b35d -> 0x440063d4`.
* **Caminho exato de `BGO driver init failed[1].` (`BOOT.BIN`):** `0x44004abd` $\rightarrow$ `0x4400836f` $\rightarrow$ `0x440041dc` (`0x84380000` autoselect) $\rightarrow$ `0x44001c49` $\rightarrow$ `0x4401e636` retorna `1` antes mesmo de `0x44003fd0` (`Control information init`). Como `0x4400836f` devolve `0xFFFFFFFF`, `0x44004acc` pula `0x440065bc(1)` (HDD) direto para `0x44007812`.
* **`wait 1 second.` em `serial boot mode` (`step40` / `step47`):** Executa apenas `delay_ms(1000)` (`0x440063d4`) seguido de `*** timer correspond initialize ***`. A função `0x4400488a` (`uart_gets`, chamando `uart_getc` `0x4400479f`) possui **0 chamadores em todo `BOOT.BIN`**.
* **`** can not close tray **`:** `0x44009c85` envia CDB ATAPI `1B 00 00 00 03` (`START/STOP UNIT`) via `0x44009b11`; qualquer código de retorno diferente de zero faz `0x44009cbe` imprimir `** can not close tray **`, independentemente da posição física da tampa.
* **`GetSD_Status` (`-91006` / `-90002`):** `0x44027eda -> 0x44027826` lê `ACMD13` e rejeita `SD_CARD_TYPE` na estrutura em `0x50574914` quando fora do perfil esperado pelo loader de 2006/2007.
* **Anatomia de `DRV_decomp.bin` (`1.441.792 B` = `0x160000`, descompactado de `DRV.BIN` + `0x40`):**
  - **Tabela CLI de fábrica (`0x400df5c0`, 62 entradas `<8sIII`):** `[00] SLOGON` até `[61] CDRW`; saída redirecionada para buffer circular `SLOG` em RAM (`0x889f1fe0`), drenado ao host apenas por comandos vendor ATAPI (`0x40020b51`, `0x40020be4`, `0x40020c12`).
  - **Tabela de despacho ATAPI (`0x400de4b2`, 256 entradas):** **74 opcodes ativos** (`62` SCSI/MMC + `12` Vendor Matsushita `0xE8..0xFE`, além da LUT em `0x400dea64`).
  - **Sense LUT (`0x400de35c`) e Dicionário Mecânico Shift-JIS (`0x400e779a..0x400e8f50`, stride `0x34`):** Índice `0x38` guarda `04 09 04`, enquanto `0x400e8c1e` guarda `0x83A7` = **`0.5mW発光ができない(HFｵﾝ密調整)`**.

---

## 5. Registro Honesto de Hipóteses Descartadas vs. Provas Obtidas

| Hipótese Inicial / Intermediária Durante a Sessão | Como Foi Testada / Auditada | Veredito Final 100% Fiel |
| :--- | :--- | :--- |
| `P7501` era a interface síncrona JTAG/OCD `DWire32A` (`CN55001`). | `PINCHECK` + `BUSDIAG` no Pico + CAD vetorial dos 3 manuais EH55 (`NA`/`EU`/`Vol.2`). | **Refutada para `DMR-EH55` (`P7501`).** `P7501` é UART Assíncrona `115200 8N1` + `SW_UART`/`XRESET`/`NSW5V`. `DWire32A` aplica-se apenas a alvos OCD legados (`CN55001`). |
| `panasonic_dmr-eh55_flash_dump.bin` (`8 MB`) era um dump real para disassemblar no Ghidra. | Auditoria de entropia e contagem de bytes em `STEP 13841`. | **Refutada.** Era mock sintético de testes Rust (`99,999%` bytes `0xFF`). Análise transferida para `PANA_DVD.FRM` real. |
| Partição 2 (`0x04000000`, `720.896 B`) seria módulo `OSD`. | Zlib decompress + strings + CLI/ATAPI tables (`step5`–`step6`). | **Refutada.** É o microcódigo `DRV.BIN` do drive óptico (`1.441.792 B` descompactados). |
| Todos os pinos de `P7501` puxariam para `D3R3V` ou precisariam FT232RL simples. | Análise CAD `NA p.55/62/63/64/83` + `PINCHECK`. | **Refinada com precisão.** `Pino 5 (NSW5V)` vem de `IC1506 (~5.15V)` / `D7510` (manter `GP3` em `Hi-Z`!); `Pino 1` lê `~3.25V` Hi-Z e grampeia em `~2.69V` via junção B-C de `Q7510` (`3.30V - V_BC`). |
| `UARTSEND VER` com `SW_UART=LOW` receberia prompt `FMU`/AMOBP no `wait 1 second.`. | Teste ao vivo `UARTSEND` (`0 bytes`) + CAD `Q7510`/`Q7514` + `step47_final_decode.py`. | **Duplamente refutada.** Em hardware, `SW_UART=LOW` corta `Q7514` isolando `GP4`. Em software, `BOOT.BIN` faz `delay_ms(1000)` simples e `uart_gets` (`0x4400488a`) tem **0 chamadores**. |
| `80040904` / `** can not close tray **` vinha de tampa aberta ou flag IDE em NVRAM/EEPROM `IC7404`. | Testes sem cabo IDE/Molex, tampa parafusada, `step15b`..`step50`. | **Refutada.** Duas supostas xrefs (`0x4006f5e1`, `0x401385eb`) eram `call32 printf/strlen`; `0x83A7` é falha física de emissão laser `0.5 mW` com HF ON (`DRV` + OPU), e `BOOT.BIN` não acessa `IC7404` I2C. |
| Após `JIGBOOT`/`UARTSEND`, `0.02V` em `STATUS` significava desligamento da placa. | Comparação `PINCHECK` + `UARTPROBE 8` em `COM10` + leitura de `firmware/pico-p7501/hal/p7501_hw.c`. | **Bug de firmware Pico identificado e corrigido.** `p7501_hw_isolate_bus_hiz()` e `p7501_hw_read_vdd()` agora limpam SIO e chamam `adc_gpio_init(GP26)` ao retornar ao ADC. |

---

## 6. Ferramentas Criadas a Partir Desta Sessão

- **Módulos Reutilizáveis Python:** [`scripts/re/mn10300.py`](../../scripts/re/mn10300.py) e [`scripts/re/pana_frm.py`](../../scripts/re/pana_frm.py)
- **Ecossistema de Bancada Não-Destrutivo:** [`scripts/bench/protocol.py`](../../scripts/bench/protocol.py), [`scripts/bench/decoders.py`](../../scripts/bench/decoders.py) e [`scripts/bench/runner.py`](../../scripts/bench/runner.py)
- **Catálogo Visual & Compilado de Scripts:** [`CATALOGO_VISUAL_ESQUEMAS_E_PCB_EH55.md`](./CATALOGO_VISUAL_ESQUEMAS_E_PCB_EH55.md) e [`COMPILADO_SCRIPTS_RE_STEPS_01_50.md`](./COMPILADO_SCRIPTS_RE_STEPS_01_50.md)
