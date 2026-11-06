# Acervo de Manuais de Serviço e Esquemas Elétricos (`manuals_and_schematics/`)

Biblioteca técnica oficial de referência para diagnóstico de hardware, diagramas de blocos, esquemas elétricos detalhados e procedimentos de serviço para os gravadores Panasonic DIGA da família **DMR-EH55** e variantes regionais.

---

## 📚 Catálogo do Acervo

| Arquivo PDF | Descrição Técnica | Região / Mercado | Páginas |
| :--- | :--- | :--- | :---: |
| `c2c7f1.pdf` | **Manual de Serviço Completo com Esquemas Elétricos Integrados** (Diagramas esquemáticos da placa principal `VEP79138`, pinagem de conectores `CN55001`, `IC55007` e fontes). | Global / Master | ~100 |
| `9e8a23.pdf` | **Manual de Serviço Oficial Vol. 1** (Procedimentos de desmontagem mecânica, autodiagnóstico `SELF-CHECK`, códigos de erro `U99` e fluxo de montagem). | Global / Master | ~80 |
| `panasonic_dmr-eh55p9_eh55pc9.pdf` | **Manual de Serviço Oficial P9/PC9** (Modelos destinados aos Estados Unidos e Canadá). | América do Norte | 97 |
| `panasonic_dmr-eh55ec.pdf` | **Manual de Serviço Oficial EC** (Modelos da Europa Continental e Alemanha). | Europa | ~90 |
| `panasonic_dmr-eh55ee.pdf` | **Manual de Serviço Oficial EE** (Modelos da Rússia, Leste Europeu e Oriente Médio). | Europa / Ásia | ~90 |
| `dmreh55.pdf` | **Guia Rápido de Especificações de Engenharia e Manutenção**. | Global | 2 |
| `Panasonic dmr-eh55 installation.pdf` | **Guia Oficial de Instalação e Procedimentos de Atualização de Firmware via CD-R**. | Global | ~10 |
| `panasonic_dmr-eh55,dmr-eh56_dvd_user_manual.pdf` | **Manual de Instruções e Operação do Usuário** (Sistemas de arquivos, funções de gravação e formatação de mídia). | Global | ~120 |
| `philips-dmr-eh55.pdf` | **Dossiê Comparativo de Mecanismos e Unidades de Gravação Óptica**. | Referência OEM | ~70 |
| Demais guias regionais (`6097ea.pdf`, `a5e678.pdf`, `cb6cee.pdf`, `22767039.pdf`...) | **Adendos de Engenharia e Atualizações de Peças de Reposição**. | Regional | Diversas |

---

## 🔍 Busca Automatizada nos Manuais com `panadiga`
Você pode indexar e buscar termos diretamente em todos os manuais acima usando o comando nativo:
```bash
cargo run --release -- manuals --term U99 --term VEP79138 --term CN55001
```
