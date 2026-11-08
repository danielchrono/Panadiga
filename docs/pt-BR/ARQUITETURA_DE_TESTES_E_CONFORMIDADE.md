# Arquitetura de Testes & Matriz de Conformidade do Ecossistema Panadiga

## Validação Multicamadas: FP Pure Domain, MVVM Rigoroso, GDB Introspectivo & SSOT

> **Versão:** 1.0.0 | **Data de Homologação:** Novembro de 2026
> **Status:** 100% dos Quality Gates Aprovados (41 Testes Rust, 33 Testes Vitest, 8 E2E Playwright, 27 Pytest, 7 C11 Host Tests)
> **Autores:** Equipe de Engenharia Reversa e Arquitetura de Sistemas Panadiga

---

## 1. Visão Geral & Filosofia de Testes

A evolução arquitetural do **Panadiga** exigiu uma reestruturação profunda da estratégia de testes. A abordagem anterior, baseada em testes pontuais e acoplados a arquivos físicos em disco, foi substituída por uma **Pirâmide de Testes Multicamadas**, onde cada teste protege estritamente os princípios de engenharia definidos:

```
                          PIRÂMIDE DE TESTES PANADIGA
                                       ▲
                                      / \
                                     /   \
                                    / E2E \   (8 testes Playwright Chromium + CI/CD Multiplataforma)
                                   /───────\
                                  /  MVVM   \  (33 testes Vitest: State Machines, Subcomponentes, Logger)
                                 /───────────\
                                /  GDB & HAL  \ (7 testes C11 Host + Telemetria Ring Buffer + Checksums)
                               /───────────────\
                              /  FP Pure Domain \ (41 testes Rust: fatias &[u8], Clap Matrix, SSOT Memory Map)
                             /───────────────────\
```

---

## 2. Camada 1: FP Pure Domain (Testes sobre Fatias Imutáveis `&[u8]`)

### Objetivo Arquitetural

Desacoplar a lógica forense de baixo nível de operações de E/S em disco. Todas as operações matemáticas, de parsing e de decodificação são funções puras sobre fatias de memória imutáveis (`&[u8]`).

### Suíte de Testes Implementada

| Arquivo de Teste                      | Função / Módulo Testado              | Garantia de Conformidade                                                                                                                                                |
| :------------------------------------ | :-------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/nvram/pure_fp_tests.rs`        | `clean_nvram_slice(&[u8])`            | Testa a sanitização cirúrgica de microsetores SA134..SA141 diretamente em memória RAM, verificando integridade do bootloader e zeramento estrito da partição U99. |
| `src/hdd/inspect/pure_hdd_tests.rs` | `inspect_hdd_slice_pure(&[u8])`       | Valida MBR proprietário Panasonic, tabela de espelho em LBA 57344 e partições de firmware sobre buffers sem tocar o sistema de arquivos.                             |
| `tests/entropy_math_tests.rs`       | `calculate_entropy(&[u8])`            | Verifica o cálculo de entropia de Shannon (0.0 para sequências puras, 1.0 para binários alternados, 8.0 para distribuição uniforme).                               |
| `tests/crypto_codebook_tests.rs`     | `test_real_frm_dump_toc_and_codebook` | Testa a dissecação da TOC periódica de 48 bytes e matriz de conhecidos pares de texto-plano do container `PANA_DVD.FRM`.                                           |

---

## 3. Camada 2: GDB Introspectivo & Protocolo de Hardware (DWire32A / RSP)

### Objetivo Arquitetural

Garantir fidelidade protocolar absoluta na ponte de depuração serial-para-TCP, validação estrita de integridade contra corrupção de pacotes e isolamento do microcontrolador RP2040/RP2350.

### Suíte de Testes Implementada

| Arquivo de Teste                                     | Função / Módulo Testado              | Garantia de Conformidade                                                                                                                                                                                                             |
| :--------------------------------------------------- | :-------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tests/debugger_gdb_tests.rs`        | `test_gdb_rsp_checksum_calculation`   | Validação formal do cálculo de checksum de dois dígitos hexadecimais (`sum % 256`) para comandos RSP (`qSupported`, `g`, `?`, `c`, `s`, `OK`).                                                                     |
| `src/desktop/pico/gdb_bridge.rs`                   | `test_verify_and_classify_rsp_packet` | Testa o classificador formal de pacotes RSP: validação de prefixo `$`, delimitador `#`, cálculo de checksum e extração do tipo de comando.                                                                                  |
| `src/desktop/pico/gdb_bridge.rs`                   | `test_telemetry_ring_capacity`        | Valida o buffer circular introspectivo (`RingBuffer<GdbPacketTelemetryEvent>`): capacidade máxima fixa, rotação FIFO sem vazamentos e ordenação cronológica.                                                                 |
| `dwire32a_jtag_research/tests/test_gdb_rsp_host.c` | Host C11 Firmware (`-Werror`)         | 7 testes de integração do protocolo RSP compilados com GCC C11 executados diretamente no ambiente de build: leitura/escrita em SRAM, rejeição de escrita em flash IPL (`E05`), registradores AM34 e controle de halt/continue. |

---

## 4. Camada 3: MVVM Rigoroso, Subcomponentes & Logger Dedicado

### Objetivo Arquitetural

Substituição definitiva de booleanos soltos por máquinas de estado fechadas (ADTs / Discriminated Unions) na camada React, decomposição de componentes de responsabilidade única e centralização de logs estruturados.

### Suíte de Testes Implementada (Vitest)

| Arquivo de Teste                 | Componente / ViewModel                               | Garantia de Conformidade                                                                                                                                                                                               |
| :------------------------------- | :--------------------------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `useInCircuitErase.test.ts`    | `useInCircuitErase`                                | Testa os estados discretos (`idle`, `confirming`, `erasing`, `success`, `error`) e callbacks de log/sucesso durante o unbrick cirúrgico.                                                                    |
| `useSerialConsole.test.ts`     | `useSerialConsole`                                 | Valida o terminal interativo: anel de histórico de comandos, buffers de log com limite de capacidade e despacho de comandos.                                                                                          |
| `useGdbBridge.test.ts`         | `useGdbBridge`                                     | Testa transições da ponte GDB (`stopped`, `starting`, `running`, `stopping`, `error`), telemetria assíncrona e despacho de logs para a UI.                                                                |
| `GdbServerComponents.test.tsx` | `GdbServerControls` & `GdbExternalToolLaunchers` | Testa a decomposição do monólito GDB: exibição correta dos badges de estado, início/parada do servidor TCP e acionamento dos disparadores de ferramentas (GDB Multiarch, OpenOCD AM34, Ghidra, HxD, PowerShell). |
| `logger.test.ts`               | `PanadigaLogger`                                   | Valida o subsistema de logging do frontend: filtragem por severidade (`minLevel`), despacho para múltiplos sinks desacoplados, descarte de instâncias e resiliência contra falhas em sinks.                       |
| `translations.test.ts`         | `i18n` Engine                                      | Valida paridade completa entre dicionários `pt-BR` e `en-US` e resolução de chaves canônicas.                                                                                                                  |

---

## 5. Camada 4: Decomposição do CLI & SSOT de Constantes

### Objetivo Arquitetural

Garantir que argumentos de linha de comando sejam fortemente validados via Clap, que handlers atuem de forma determinística e que nenhuma constante física ou caminho de arquivo seja hardcoded fora de `constants.rs`.

### Suíte de Testes Implementada

| Arquivo de Teste            | Módulo Testado                                   | Garantia de Conformidade                                                                                                                                                                                        |
| :-------------------------- | :------------------------------------------------ | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tests/firmware_scan_tests.rs`       | `test_cli_args_parsing_matrix`                  | Valida parsing e desserialização tipada de subcomandos isolados (`memory-map`, `gdb-config`, `pico list`, `nvram clean`, etc.), aliases (`tui`, `verify`) e rejeição de comandos desconhecidos. |
| `tests/system_constants_tests.rs`    | `test_constants_memory_map_and_candidate_paths` | Valida as 8 regiões de memória oficiais da arquitetura AM34/UniPhier, formatação de tabela ASCII para terminal, exportação de schema JSON canônico e presença de caminhos candidatos universais.        |
| `tests/pico_bench_tests.rs`          | `test_all_host_controller_platforms_matrix`     | Valida a matriz de identificação de hardware USB VID/PID para Raspberry Pi Pico, Pico W, Pico 2 (ARM/RISC-V) e compatíveis.                                                                                  |

---

## 6. Camada 5: End-to-End (E2E) & CI/CD Multiplataforma

### Cobertura Playwright (Chromium)

8 testes automatizados de ponta a ponta verificam o comportamento visual e reativo da aplicação:

1. `navigation.spec.ts`: Carregamento do Dashboard e métricas de sistema em tempo real.
2. `navigation.spec.ts`: Navegação fluida e sem quebra de estado entre todos os módulos de bancada.
3. `nvram.spec.ts`: Renderização da especificação da Spansion NOR Flash S29JL064H e card cirúrgico.
4. `nvram.spec.ts`: Detalhamento dos 8 microsetores (SA134..SA141) e telemetria de integridade.
5. `pico-bench.spec.ts`: Telemetria do host controller e identificação de arquitetura.
6. `pico-bench.spec.ts`: Tabela de pinagem do conector P7501 (DWire32A) com alertas de aterramento.
7. `pico-bench.spec.ts`: Alternância de modos do Cold Break-In (Modo 0, 1 e 2).
8. `pico-bench.spec.ts`: Interatividade do console serial com eco e envio de comandos.

### Matriz de Integração Contínua (GitHub Actions)

Todos os 8 jobs rodam sob cada push na branch `main`:

- **Android APK Build (Tauri 2.0 / ARM64)**: Compilação de binário Android com `cargo-ndk`.
- **Rust Workspace (Ubuntu Linux)**: `cargo check` e suíte completa de testes.
- **Ghidra Generator & Python Tools**: `py_compile`, `mypy --strict` e `pytest` (27 testes).
- **Desktop UI**: `tsc --noEmit`, Vitest (33 testes), `uxlint` (100/100) e Playwright E2E.
- **Rust Workspace (macOS latest ARM64)**: Validação em Apple Silicon.
- **Rust Workspace (Windows latest)**: Validação em ambiente Windows nativo.
- **Code Quality & Duplication**: `cargo fmt`, `clippy` e detector de clones `jscpd` (limite estrito 0%).
- **Pico Firmware Dual Target**: Compilação UF2 para RP2040 e RP2350 (Cortex-M33).

---

## 7. Matriz de Rastreabilidade: Requisito Arquitetural ➔ Teste Protetor

| Requisito Arquitetural               | Problema Mitigado                                                                  | Teste Automatizado de Proteção                                                                 |
| :----------------------------------- | :--------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------- |
| **FP Pure Domain**             | Regressão de decodificadores que dependiam de arquivos locais ou caminhos fixos.  | `pure_fp_tests.rs`, `pure_hdd_tests.rs`, `test_entropy_calculation`                        |
| **MVVM Rigoroso**              | Telas travando em estado intermediário devido a flags booleanas soltas.           | `useInCircuitErase.test.ts`, `useGdbBridge.test.ts`                                          |
| **GDB Introspectivo**          | Corrupção de pacotes RSP ou estouro de buffer de telemetria em alta frequência. | `test_gdb_rsp_checksum_calculation`, `test_telemetry_ring_capacity`, `test_gdb_rsp_host.c` |
| **SRP em Componentes**         | Regressão nos launchers rápidos ou controles do socket TCP de depuração.       | `GdbServerComponents.test.tsx`                                                                 |
| **Zero Raw Prints / Consoles** | Poluição de stdout que quebra integrações JSON-RPC de servidores MCP.          | `logger.test.ts`, `test_logger_custom_sink`, `test_panadiga_mcp_server.py`                 |
| **SSOT de Constantes**         | Conflito de endereços de memória entre Ghidra, TUI, CLI e Desktop.               | `test_constants_memory_map_and_candidate_paths`, `test_system_constants_integrity`           |
| **Decomposição do CLI**      | Parsing silencioso ou argumentos ignorados no novo entrypoint enxuto.              | `test_cli_args_parsing_matrix`                                                                 |
