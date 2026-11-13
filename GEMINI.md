# Antigravity & Gemini Pair Programming Directives — Panadiga Ecosystem

Este arquivo define as diretrizes de projeto para Antigravity e assistentes baseados em Gemini.

## Contexto do Projeto
- **Nome do Projeto**: Panadiga (Panasonic DIGA DMR-EH55 Engineering, Forensics, and Hardware Lab)
- **Idiomas Principais**: Rust 2021, TypeScript (React/Tauri), C11 (RP2040), Python 3.10+ (Typer, Pytest, Mypy Strict)
- **Objetivo**: Engenharia reversa, recuperação forense do sistema de arquivos de disco MEIHFS, unbrick cirúrgico de NVRAM (erro U99) e diagnóstico via interface UART assíncrona `P7501` (`115200 8N1`).

## Diretrizes de Execução do Agente
1. **Verificação de Invariantes Físicas**:
   - Nunca sugira aplicar > 3.3V nos pinos LVTTL do SoC AM34 e mantenha `GP3` (`P7501 Pino 5 NSW5V`, ~5.15V upstream) sempre em `Hi-Z Input` sem pulls (nunca push-pull).
   - Nunca modifique a faixa de bootloader `0x80000000 - 0x807EFFFF` de imagens Flash.
   - Sempre valide hashes SHA-256 antes e após qualquer operação de sanitização de NVRAM (`0x7F0000 - 0x7FFFFF`).

2. **Delegação a Subagentes**:
   - Ao executar tarefas de alta complexidade em áreas específicas, consulte e instancie os subagentes localizados em `.agents/agents/`.

3. **Validação Contínua**:
   - Ao concluir modificações em código fonte (Rust, C, Python, TypeScript), rode `npm run validate` (ou `python scripts/validate.py`).
   - Mantenha o limiar de duplicação `jscpd` em 0% (0 clones).
   - Mantenha `mypy --strict` passando sem erros nas ferramentas Python.

4. **Ferramentas MCP**:
   - Utilize as ferramentas expostas pelo servidor MCP nativo em Rust (`panadiga mcp`, configurado em `.mcp.json`).
