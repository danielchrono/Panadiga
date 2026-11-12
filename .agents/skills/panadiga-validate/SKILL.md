---
name: panadiga-validate
description: >-
  Executa a suíte completa de 9 Quality Gates do ecossistema Panadiga
  (Rust, TypeScript, Vitest, jscpd 0%, uxlint, Playwright E2E, Ghidra export, C11 GCC host tests e Python Typer/pytest).
  Use esta skill sempre que modificar código ou antes de finalizar qualquer tarefa.
---

# Procedimento de Validação Universal — 9 Quality Gates

Esta skill orienta a execução e resolução de problemas da suíte de testes e qualidade do Panadiga.

## Como Executar
Na raiz do repositório (Windows, Linux ou macOS):
```bash
npm run validate
```

Ou diretamente via Python 3:
```bash
python scripts/validate.py
```

## Os 9 Portões Validados
1. **Rust Workspace**: `cargo check --workspace --all-targets` e `cargo test --workspace --verbose` (16 testes).
2. **TypeScript Strict**: `npm run typecheck` em `apps/desktop`.
3. **Frontend Vitest**: `npm test` em `apps/desktop` (26 testes).
4. **Copy/Paste Detector**: `npx --prefix apps/desktop jscpd --config .jscpd.json` (0 clones).
5. **UX Completeness**: `npm run uxlint` em `apps/desktop` (score 100/100).
6. **Playwright E2E**: `npm run test:e2e` em `apps/desktop` (8 testes em Chromium headless).
7. **Ghidra Native Generator**: `cargo run -- ghidra export-all --dir tools`.
8. **Pico C11 Host Tests**: Compilação e execução com GCC `-Werror -DPICO_HOST_TEST=1`.
9. **Python Typer & Pytest**: `python -m mypy --strict` e `python -m pytest tools/tests tools/rag_engine/tests`.

## Resolução de Falhas Comuns
- **Falha em jscpd**: Identifique blocos repetidos entre `apps/desktop/src`, `src`, `dwire32a_jtag_research` ou `tools` e extraia funções utilitárias compartilhadas.
- **Falha em mypy**: Certifique-se de que todas as funções Python possuem anotações de tipo completas em argumentos e retornos (`-> None`, etc.).
