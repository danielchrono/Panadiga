# Invariantes do Portão de Qualidade (Panadiga)

Qualquer alteração submetida deve satisfazer os 9 Quality Gates sem exceção:

1. **Rust Check & Tests**: `cargo check --workspace --all-targets` e `cargo test --workspace --verbose` devem passar com 0 falhas.
2. **TypeScript Strict**: `npm run typecheck` em `apps/desktop` com 0 erros.
3. **Vitest Unit**: `npm test` em `apps/desktop` com 100% de sucesso.
4. **Detector de Duplicação (jscpd)**: `threshold: 0` - 0 clones tolerados.
5. **Linter UX (uxlint)**: `npm run uxlint` em `apps/desktop` pontuação 100/100.
6. **Playwright E2E**: `npm run test:e2e` em `apps/desktop` com todos os cenários passando.
7. **Ghidra Native Generator**: `cargo run -- ghidra export-all --dir tools` executando sem erros e mantendo o SSOT em Rust sincronizado.
8. **Pico C11 Host Test**: Compilação e execução via GCC com `-Werror -DPICO_HOST_TEST=1`.
9. **Python Typer CLI & Pytest**: `python -m mypy --strict` e `python -m pytest` sem avisos ou falhas.
