# Panadiga Workspace AI Rules

## Diretrizes Gerais de Engenharia
1. **Preservação de Dados e Integridade Forense**:
   - Nunca modifique arquivos de dump originais (`*.bin`, `*.FRM`) diretamente.
   - Qualquer operação de transformação deve gerar novo arquivo e validar hashes SHA-256.
2. **Segurança de Silício e Hardware de Bancada**:
   - Tensão máxima em pinos de depuração (CN55001) e Flash (TSOP-48) é rigorosamente 3.3V.
   - Toda rotina de Cold Break-In do RP2040 deve inicializar com sinais em nível lógico alto (pull-up) e respeitar o timing Matsushita síncrono.
3. **Padrão de Código e Qualidade**:
   - Zero clones no detector de duplicação `jscpd` (threshold 0).
   - Rust: código idiomático, memory safety estrita (`#![deny(unsafe_code)]` onde aplicável).
   - Python: tipagem completa verificada por `mypy --strict`.
   - TypeScript: modo estrito (`noImplicitAny`, etc.), testes Vitest e Playwright cobrindo novos fluxos.
