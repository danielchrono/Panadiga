# Dossiê de Engenharia Reversa: Cifra Feistel (`FUN_4400a666`), Contêineres `.FRM` (`EXTRHEAD`) e Gates UART0/Debug (`DMR-EH55`)

> **Escopo & Integridade:** Este documento consolida, de forma persistente no repositório, as descobertas empíricas obtidas no disassembly MN103S de `BOOT.BIN` (**Trilha A**) e `EH55_MAIN_3410PQ.bin` (**Trilha B**), bem como sua materialização modular em Rust (`panadiga-core` / `panadiga-cli`) e Python (`scripts/re/pana_frm.py`). Todos os endereços, chaves, tabelas e invariantes aqui descritos foram validados diretamente sobre `firmware_and_images/PANA_DVD.FRM` e nos testes automatizados do workspace.

---

## 1. Trilha A — Cifra Feistel de 64 Bits (`BOOT.BIN`)

A camada de proteção de `PANA_DVD.FRM` e das partições de firmware do disco rígido (`MEIHDFS`) utiliza uma cifra de bloco Feistel de 64 bits em modo ECB com 16 rodadas, cuja rotina principal em `BOOT.BIN` reside em `FUN_4400a666`.

### 1.1 Artefatos Criptográficos Extraídos

| Componente | Origem em `BOOT.BIN` / `PROG061_BOOT` | Símbolo em `panadiga-core::constants` | Valor / Observação |
| :--- | :--- | :--- | :--- |
| Chave Parte 1 (`7 B`) | `.data` `0x50480278` (off `0x39AA8`) | `PANA_FEISTEL_KEY_PART1` | `72 EB CA 91 30 45 68` |
| Chave Parte 2 (`8 B`) | `.data` `0x5048027F` (off `0x39AAF`) | `PANA_FEISTEL_KEY_PART2` | `AD 99 DF 56 72 3A B8 10` |
| Chave Mestra (`7 B`) | Reconstruída (`reconstruct_master_key_7b`) | `PANA_FEISTEL_MASTER_KEY_7B` | `DF 62 15 C7 52 7F D0` |
| IV do Key Schedule (`4 B`) | `FUN_44011d42` | `PANA_FEISTEL_KEYSCHED_IV` | `35 7D FB 9A` |
| Subchave Expandida (`32 B`) | Saída de `FUN_44011d42` | `PANA_FEISTEL_ROUND_KEY_32B` | `A3 E7 9C 01 ... 25 51 F3 15` |
| S-Box Oficial (`256 B`) | `0x44035D84` (`BOOT`) / `0x50435C0A` (`PROG061`) | `PANA_FEISTEL_SBOX_256` | Permutação/substituição não linear de 256 bytes |

### 1.2 Reconstrução da Chave, Key Schedule e Rodada Feistel

1. **Reconstrução da Chave Mestra de 7 bytes (`reconstruct_master_key_7b` em `crates/panadiga-core/src/crypto/feistel.rs`):**
   - Para `i` em `0..7`: `master[i] = p2[i] ^ p1[i]`.
   - Em seguida: `master[1] ^= p2[7]` e `master[4] ^= p2[7]`.
2. **Função de Rodada F (`FUN_44012191` / `feistel_f_round`):**
   - Mistura o bloco de 4 bytes `[b0, b1, b2, b3]` com 4 bytes de subchave `[k0, k1, k2, k3]` usando rotações direitas (`rotate_right(2)` e `rotate_right(5)`), deslocamentos de bits e consulta à `PANA_FEISTEL_SBOX_256`, somando `s.rotate_left(3..0)` módulo `2^8` às 4 saídas.
3. **Expansão de Subchaves (`FUN_44011d42` / `derive_round_key_32b`):**
   - Copia os 7 bytes de `master` para o início do buffer `rk[0..32]` e inicializa `state = PANA_FEISTEL_KEYSCHED_IV`.
   - Executa 16 iterações indexando `off = (round & 7) * 4`, substituindo `rk[off..off+4]` pela saída de `feistel_f_round` e encadeando `state`.
4. **Cifra / Decifra de Bloco de 8 bytes (`FUN_44011ec0` / `FUN_44012029` / `FUN_4400a666`):**
   - Divide o bloco em `L[0..4]` e `R[0..4]`. Na cifragem percorre `round = 0..16`; na decifragem percorre em ordem reversa `round = 15 - step`.

### 1.3 Adler-32 Amostrado (`FUN_4401e5a3`)

O cabeçalho `EXTRHEAD` valida payloads descomprimidos utilizando uma variante de **Adler-32 amostrada por passo (`stride`)**, implementada em `FUN_4401e5a3` e portada em `adler32_panasonic_strided`:

- `MOD_ADLER = 65521`, `s1 = 1`, `s2 = 0`.
- Percorre `data` com passo `stride = step.max(1)` (padrão `PANA_EXTRHEAD_DEFAULT_ADLER_STEP = 16`):
  - `s1 = (s1 + byte) % 65521`
  - `s2 = (s2 + s1) % 65521`
- Retorna `s1 | (s2 << 16)`.

---

## 2. Estrutura de Contêineres `.FRM` e `EXTRHEAD`/`EXTRFOOT`

### 2.1 `PANA_DVD.FRM` (TOC Cifrada em Slots de 48 Bytes)

A região de cabeçalho `"Firm list"` ocupa `0x1800` bytes (6.144 B) iniciais cifrados via `FUN_4400a666`, composta por slots de **48 bytes (`0x30`)**, validados pelo somatório `list_sum` (`u16` LE dos primeiros 46 bytes):

| Slot | Offset TOC | Módulo | Offset Payload | Tamanho (`B`) | Hardware | Modelo | `list_sum` |
| :---: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `[0]` | `0x0000` | `DRV S211` | `0x00001800` | `0x000B0000` (`720.896`) | `M6930` | `NP-309` | `0x0AE1` (`ok=true`) |
| `[1]` | `0x0030` | `PROG061 ` | `0x000B1800` | `0x00040000` (`262.144`) | `M6930` | `NP-309` | `0x08F9` (`ok=true`) |
| `[2]` | `0x0060` | `MAIN3060NP` | `0x000F1800` | `0x0063D4E0` (`6.542.560`) | `M6930` | `NP-309` | `0x0B26` (`ok=true`) |

Cada entrada inclui ainda `fvu_checksum_u32` (soma de bytes do módulo em `+0x24`), `version_u16`, `target_flag` e `model_group`.

### 2.2 `PANA__SD.FRM` (TOC Plaintext em Slots de 64 Bytes) e `DIGADIAG.FRM`

- **`PANA__SD.FRM` (`FUN_4400a733` / `FUN_4400a6d0`):** utiliza até `128` slots em **plaintext** de `0x40` (64 B) bytes cada (`PANA_SD_TOC_REGION_SIZE = 0x2000` / 8.192 B). A rotina `FUN_4400a6d0` verifica `list_sum` em `+0x3E` como a soma `u16` dos primeiros `0x3E` (62) bytes (`"_fmu_error: list sum is invalid"` em caso de divergência).
- **`DIGADIAG.FRM` (`build_digadiag_frm`):** contêiner diagnóstico de tamanho fixo `0x20000` bytes (`PANA_DIGADIAG_FRM_SIZE = 131.072 B`) em modo plaintext, encapsulando um payload binário via `EXTRHEAD`/`EXTRFOOT` (`algo = 0`, zlib, endereço destino padrão `0x50400000`, preenchido com zeros até 128 KiB).

### 2.3 Encapsulamento `EXTRHEAD`/`EXTRFOOT` com Cifragem Parcial (`extrhead.rs`)

- **Cabeçalho de 64 bytes (`0x40`):** magic `"EXTRHEAD"`, `tag` (4 B), `flag` (`u16`), `algo` (`u16`: `0` = zlib, `3` = LZMA em `FUN_4400b6c6`), `dst_len` (`+0x10`), `dst_addr` (`+0x14`), `src_len` (`+0x18`), `footer_offset` (`+0x20`), `header_checksum` (`+0x28`), `adler_step` (`+0x2C`), `build_name` (`+0x30..0x40`), seguido do fluxo comprimido + rodapé `"EXTRFOOT"` + padding de 8 bytes.
- **Modos de cifragem parcial:**
  - `main` (`MAIN`): cifra/decifra apenas o prefixo de `0x19000` bytes (`PANA_EXTRHEAD_MAIN_CRYPT_SPAN = 102.400 B`) e o sufixo final alinhado de `0x19000` bytes antes de `aligned_end`.
  - `drv` (`DRV`): cifra/decifra apenas o prefixo de `0x2800` bytes (`PANA_EXTRHEAD_DRV_CRYPT_SPAN = 10.240 B`), deixando o restante em plaintext (incluindo `"EXTRFOOT"`).
  - `plain`: sem cifragem Feistel (`DIGADIAG.FRM` e imagens abertas).

---

## 3. Arquitetura Modular em Rust e CLI (`panadiga-core` & `panadiga-cli`)

Os protótipos iniciais em Python foram portados para módulos focados em Rust no crate `panadiga-core`:

- `crates/panadiga-core/src/crypto/feistel.rs` — chave mestra, `feistel_f_round`, `derive_round_key_32b`, cifra/decifra ECB de bloco/buffer e `adler32_panasonic_strided`.
- `crates/panadiga-core/src/crypto/codec.rs` — utilitários binários (`pad_to_block`, leituras/escritas LE e strings ASCII).
- `crates/panadiga-core/src/crypto/extrhead.rs` — parser, `pack_extrhead` e `unpack_extrhead` com `ExtrheadCryptMode::{Auto, Main, Drv, Plain}`.
- `crates/panadiga-core/src/crypto/frm_pack.rs` — `build_digadiag_frm` e `pack_frm` (`FrmContainerMode::{Dvd, Sd}`).
- `crates/panadiga-core/src/crypto/toc.rs` — análise estrutural, `decrypt_and_parse_frm_toc`, `parse_sd_frm_toc` e `dump_toc`.
- `crates/panadiga-cli/src/cli/handlers/frm_pack.rs` e `args/crypto.rs` — subcomandos `panadiga frm pack` (`--mode sd|dvd|digadiag`, `--out`, `--module NAME:PATH`, `--input`, `--dst-addr`, `--tag`, `--build`, `--hw`, `--model`, `--version-u16`, `--target-flag`, `--json`) e `panadiga frm unpack` (`<EXTRHEAD_FILE>`, `--out`, `--mode auto|main|drv|plain`, `--json`).

---

## 4. Trilha B — Gates de Hardware e Console UART0/Debug (`EH55_MAIN_3410PQ.bin`)

Análises no Ghidra sobre o binário principal `EH55_MAIN_3410PQ.bin` mapearam os pinos de hardware, variáveis de estado e tarefas RTOS que controlam a saída/entrada serial (`SC0TXB` / `SC0RXB` em `0xD4002000..0xD400200C`) e o monitor de exceções.

### 4.1 Mapeamento MMIO de Portas GPIO Matsushita (`0x57a52900..0x57a52c00`)

| String na Tabela | Endereço String | Registrador MMIO | Papel Observado |
| :--- | :--- | :--- | :--- |
| `"P0MD"` | `0x57a52bd1` | `0xDB000000` | Controle de modo / direção dos pinos Port 0 |
| `"P0IN"` | `0x57a52bcc` | `0xDB000004` | Leitura lógica dos pinos físicos Port 0 (`bit 1 = P01`) |
| `"P0OUT"` | `0x57a52bc6` | `0xDB000008` | Registrador de saída Port 0 |
| `"P0TMIO"` | `0x57a52bbf` | `0xDB00000C` | Controle Timer/IO Port 0 |
| `"P1MD"` | `0x57a52bba` | `0xDB000100` | Controle de modo Port 1 |

O bootloader `BOOT.BIN` também referencia a tabela a partir de `&PTR_P0MD_440356f0` (offsets `0x356f0`/`0x356f4`, configurando `P0MD`/`P0IN`).

### 4.2 Cadeia de Bootstrap e Gates de Console UART0 (`SC0`)

1. **`FUN_570007b4` (`0x570007b4`, 130 B, chamada de crt0/reset `0x5700042a`):**
   - Limpa os bits `[3:0]` de `P0MD` (`0xDB000000 &= 0xFFFFFFF0`), lê `P0IN` (`0xDB000004`) e testa `BTST 0x2, D3` (bit 1 = `P01`).
   - Se `P01 == 0`, testa ainda sonda de escrita em RAM de desenvolvimento (`0x80000000 = 0xA5C30000`).
2. **`FUN_57000dfa` (`0x57000dfa`, 161 B) e flags em `0x934E2EAC`:**
   - Controla bits `0x18` e `0x20` de `uRam934e2eac` e a assinatura `iRam934e3350 == 0xAA01`.
   - Quando `(in_D2 & 0x18) != 0` e `(in_D2 & 0x18) != 8`, invoca `FUN_57001fee()` (rotina de inicialização de depuração / UART0 em `0xD4002000` e `0xD4003002`). Se `(uRam934e2eac & 0x20) == 0x20` e `iRam934e3350 != 0xAA01`, entra em loop travado (`TM0MD = 1`).
3. **`FUN_574ef7b0` (`0x574ef7b0`, 34 B) → `FUN_574ef7d2` (`0x574ef7d2`):**
   - `FUN_574ef7b0`: limpa bits `[3:2]` de `P0MD` (`Ramdb000000 &= 0xFFF3`), lê `P0IN` e retorna `((P0IN >> 1) ^ 1) & 1`.
   - `FUN_574ef7d2`: faz `bRam515b2e39 |= 0x0A` (`0b1010`), chama `FUN_574ef7b0()` e grava **`uRam515b2e38 = (ret == 0)`**. Ou seja: quando **`P01` está em nível ALTO (`1`)**, `FUN_574ef7b0` retorna `0` e **`0x515b2e38 = 1` (UART console liberada)**; em placas de varejo com `P01` aterrado/BAIXO (`0`), `0x515b2e38 = 0`.
4. **`FUN_574efb97` (inicialização do menu UART) & `FUN_574efdda` (printf) / `FUN_574f00eb` (getchar):**
   - `FUN_574efb97` (chamada por `0x574f46ba`, `0x57010c09`, `0x570cd208`) configura `0x515b2e39 = 1`, chama `FUN_574ef7d2` e testa `BTST 0x2, (0x515b2e39)` (bit 1 ligado via `|= 0x0A`, resultando em `0x0B`), inicializando o buffer circular UART0 em `0x5316A250..0x5316AE88` e escrevendo `0xC085` em `0xD4002000`.
   - `FUN_574efdda` só transmite se `cRam522bbd54 == 0` (mute off), `(bRam515b2e39 & 3) == 3` e (`cRam515b2e38 != 0` ou `FUN_5770cb76(0) == 0`). `FUN_574f00eb` bloqueia se `(bRam515b2e39 & 3) != 3`.
5. **Descritores de Tarefas RTOS & Shell de Depuração (`Check Module` e `ndb>`):**
   - **`0x577E1C30` → `FUN_570cd1fc` (`"Check Module ver1.02"` / `2005/12/16 DRS` em `0x5788F042` / `0x5788F0DF`):** tarefa de menu interativo por caractere (`id/prio 0x001F0013`), configurada na tabela RTOS com **`auto_start = 0`**.
   - **`0x577E22C0` → `FUN_574f469d` (`"Please Wait ... \n"` / `"IIC_init\n"`):** chama `FUN_574efb97` em `0x574f46ba` (`id/prio 0x00010059`), configurada com **`auto_start = 1`**.
   - **`FUN_574ae7dc` (prompt `\x1b[31mndb> \x1b[m` em `0x57A53B89`):** monitor de baixo nível (`[task_id] bt`, hex dump/reg) invocado em `0x574aebcc` após rotina de erro/exceção (`Unknown Error`, `NMICR:%08X, sISR:%08X`, `Register Dump:`), escrevendo `0xFF` em `0x20000044` ao sair. O flag `0x515b2e38` também participa dos gates de `System assert!!` em `0x570b03ab` e `0x570b03f5`.

---

## 5. Correções de Infraestrutura, Daemon Ghidra Headless e Gates de Qualidade

- **Suite de Quality Gates (`scripts/validate.py`):** homologada em **9/9 Quality Gates PASS** no commit `a561249` (Rust workspace/fmt/tests/Ghidra export, TypeScript `tsc`, Vitest, `jscpd` 0% clones, UX state lint, Playwright E2E, C11 host tests, Python `mypy --strict`, `pytest`), com CI cruzada validada em RP2040/RP2350, macOS, Ubuntu e Android APK.
- **Servidor Headless `ghidra-mcp` (`BOOT.BIN`):** daemon persistente em `http://127.0.0.1:8081/mcp` (`C:\ghidra\start_ghidra_mcp_headless_boot.cmd`) mantido operacional em stream SSE HTTP 200 sem travamento de lock no projeto `Panadiga_MN103S`.
