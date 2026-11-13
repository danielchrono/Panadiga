# Panadiga Workspace AI Rules

## Diretrizes Gerais de Engenharia
1. **Preservação de Dados e Integridade Forense**:
   - Nunca modifique arquivos de dump originais (`*.bin`, `*.FRM`) diretamente.
   - Qualquer operação de transformação deve gerar novo arquivo e validar hashes SHA-256.
2. **Segurança de Silício e Hardware de Bancada**:
   - No DMR-EH55 `P7501` (`UART 115200 8N1`), os sinais UART/controle (`SW_UART`/`XRESET`/`UARTM2P_T`/`UARTP2M_T`) operam em 3.3V LVTTL, e `GP3` (`Pino 5 NSW5V`, `~5.15V` upstream via `D7510`) deve permanecer **SEMPRE como `Hi-Z Input` sem pulls (NUNCA dirigir push-pull)**.
   - Na Flash NOR Spansion `S29JL064H` (`IC5001`, TSOP-48), a tensão máxima é rigorosamente **3.3V**; `XRESET` (`GP5`) deve ser acionado exclusivamente em **Open-Drain** (sem nunca forçar `+3.3V` push-pull).
3. **Padrão de Código e Qualidade**:
   - Zero clones no detector de duplicação `jscpd` (threshold 0).
   - Rust: código idiomático, memory safety estrita (`#![deny(unsafe_code)]` onde aplicável).
   - Python: tipagem completa verificada por `mypy --strict`.
   - TypeScript: modo estrito (`noImplicitAny`, etc.), testes Vitest e Playwright cobrindo novos fluxos.
