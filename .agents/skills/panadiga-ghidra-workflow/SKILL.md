---
name: panadiga-ghidra-workflow
description: >-
  Exporta o mapa de memória oficial AM34, sincroniza símbolos reversos e configura o ambiente Ghidra para o firmware Panasonic.
  Use esta skill ao iniciar sessões de engenharia reversa no Ghidra, atualizar símbolos ou exportar scripts de automação.
---

# Fluxo de Trabalho Ghidra para Matsushita AM34 / UniPhier

O processador Matsushita AM34 (família MN103S) requer configuração do mapa de memória correto e importação dos símbolos de hardware mapeados.

## 1. Exportação dos Scripts via Rust SSOT
Execute o comando na raiz do projeto:
```bash
cargo run -- ghidra export-all --dir tools
```
Isso gera:
- `tools/ghidra_am34_memory_map.py`: Cria todas as regiões físicas de memória (SRAM, SDRAM, Flash, MMIOs, OCD) no projeto aberto do Ghidra.
- `tools/ghidra_import_symbols_panadiga.py`: Mapeia pontos de entrada, vetores de interrupção e funções identificadas do bootloader.
- `tools/ghidra_connect_debugger.py`: Conecta o depurador do Ghidra à ponte GDB RSP do Raspberry Pi Pico.

## 2. Execução Dentro do Ghidra
1. Abra o arquivo binário desempacotado no Ghidra selecionando a arquitetura `MN10300` / `AM33/AM34`.
2. No menu **Window -> Script Manager**, adicione a pasta `tools/` aos diretórios de scripts.
3. Execute primeiramente `ghidra_am34_memory_map.py` para estruturar os blocos de memória.
4. Em seguida, execute `ghidra_import_symbols_panadiga.py`.
