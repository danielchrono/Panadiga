# Acervo Geral de Manuais de Serviço, Esquemas Elétricos e Datasheets (`manuals_and_schematics/`)

Biblioteca técnica oficial de referência de engenharia, diagnóstico de hardware, diagramas esquemáticos e manuais de serviço cobrindo o ecossistema de gravadores digitais **Panasonic DIGA** (famílias DMR-E, DMR-EH, DMR-ES, DMR-EX), além dos datasheets dos circuitos integrados de silício fundamentais.

---

## 📚 Catálogo Estruturado por Família de Aparelho e Disciplina

### 1. Família Panasonic DMR-EH55 (Plataforma Base)

#### A. Manuais de Serviço e Esquemas Elétricos (`dmr-eh55/service/`)
| Arquivo PDF | Descrição Técnica & Escopo | Região / Mercado | Páginas |
| :--- | :--- | :--- | :---: |
| `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_vol2_schematics.pdf` | **Manual de Serviço Completo com Esquemas Elétricos e PCB** (Diagramas da placa digital `VEP79138`, pinagem do conector `P7501`, `IC55007` e fontes). | Global / Master | 177 |
| `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_vol1_diagnostics.pdf` | **Manual de Serviço Oficial Vol. 1** (Procedimentos mecânicos, autodiagnóstico `SELF-CHECK`, códigos de erro `U99` e montagem). | Global / Master | 97 |
| `dmr-eh55/service/panasonic_dmr_eh55p9_eh55pc9_service_manual_north_america.pdf` | **Manual de Serviço Oficial P9/PC9** (Modelos destinados aos Estados Unidos e Canadá). | América do Norte | 97 |
| `dmr-eh55/service/panasonic_dmr_eh55ec_service_manual_europe.pdf` | **Manual de Serviço Oficial EC** (Modelos da Europa Continental e Alemanha). | Europa | 92 |
| `dmr-eh55/service/panasonic_dmr_eh55ee_service_manual_east_europe.pdf` | **Manual de Serviço Oficial EE** (Modelos da Rússia, Leste Europeu e Oriente Médio). | Europa / Ásia | 91 |

#### B. Atualização e Recuperação de Firmware (`dmr-eh55/firmware_update/`)
| Arquivo PDF | Descrição Técnica & Escopo | Região / Mercado | Páginas |
| :--- | :--- | :--- | :---: |
| `dmr-eh55/firmware_update/panasonic_dmr_eh55_firmware_installation_guide.pdf` | **Guia Oficial de Instalação e Atualização de Firmware via CD-R**. | Global | 8 |

#### C. Manuais de Instrução e Guias de Usuário (`dmr-eh55/user/`)
| Arquivo PDF | Descrição Técnica & Escopo | Região / Mercado | Páginas |
| :--- | :--- | :--- | :---: |
| `dmr-eh55/user/panasonic_dmr_eh55_operating_instructions_global.pdf` | **Manual de Instruções e Operação do Usuário** (Sistemas de arquivos, gravação e formatação de mídia). | Global | 89 |
| `dmr-eh55/user/panasonic_dmr_eh55s_operating_instructions_silver.pdf` | **Manual de Operação Oficial do Modelo Silver (DMR-EH55S)**. | Global | 89 |
| `dmr-eh55/user/panasonic_dmr_eh55_eh56_operating_instructions_europe.pdf` | **Manual de Operação Integrado DMR-EH55 / DMR-EH56**. | Europa | 84 |
| `dmr-eh55/user/panasonic_diga_dmr_eh55_user_manual.pdf` | **Guia Completo do Usuário Panasonic DIGA**. | Global | 84 |
| `dmr-eh55/user/panasonic_dmr_eh55_user_manual_oceania.pdf` | **Manual de Operação para Austrália e Nova Zelândia**. | Oceania | 76 |
| `dmr-eh55/user/panasonic_dmr_eh55_australia_new_zealand_manual.pdf` | **Guia Complementar de Operação e Especificações Oceânicas**. | Oceania | 76 |
| `dmr-eh55/user/panasonic_dmr_eh55_hdmi_hdavi_control_guide.pdf` | **Guia de Controle e Integração HDMI HDAVI / VIErA Link**. | Global | 6 |
| `dmr-eh55/user/panasonic_dmr_eh55_quick_setup_guide.pdf` | **Guia Rápido de Instalação e Conexões**. | Global | 2 |

---

### 2. Família Panasonic DMR-EH56 / DMR-EH50 / DMR-EH65 / DMR-EH80V (Geração UniPhier AM34)

| Diretório / Arquivo PDF | Categoria | Descrição Técnica & Modelo | Escopo & Diagnósticos |
| :--- | :--- | :--- | :--- |
| `dmr-eh56/user/panasonic_dmr_eh56_operating_instructions.pdf` | Manual do Usuário | **Panasonic DMR-EH56** | Operação na Europa Continental, edição de títulos e Guide Plus+. |
| `dmr-eh50/user/panasonic_dmr_eh50_user_manual.pdf` | Manual do Usuário | **Panasonic DMR-EH50** | Predecessor imediato do EH55 (mesmo SoC MN2DS0016, 8MB Flash). |
| `dmr-eh65/user/panasonic_dmr_eh65_user_manual.pdf` | Manual do Usuário | **Panasonic DMR-EH65** | Variante de alta capacidade com HDD de 250 GB e saída HDMI nativa. |
| `dmr-eh80v/service/panasonic_dmr_eh80v_service_manual_europe.pdf` | **Manual de Serviço** | **Panasonic DMR-EH80V** | Combo triplo VHS + DVD + HDD (200 GB) com esquemas da placa digital. |

---

### 3. Família Panasonic DMR-ES10 / DMR-ES30V (Série ES Slim & Combo)

| Diretório / Arquivo PDF | Categoria | Descrição Técnica & Modelo | Escopo & Diagnósticos |
| :--- | :--- | :--- | :--- |
| `dmr-es10/service/panasonic_dmr_es10_service_manual_north_america.pdf` | **Manual de Serviço** | **Panasonic DMR-ES10** | Modelo DVD-RAM slim com diagnóstico do erro U99 (pág. 13) e esquemas. |
| `dmr-es30v/user/panasonic_dmr_es30v_user_manual.pdf` | Manual do Usuário | **Panasonic DMR-ES30V** | Combo VHS + DVD-RAM com funções dubbing e cópia de mídia. |

---

### 4. Família Panasonic DMR-E55 / DMR-E85H / DMR-E500H (Geração Anterior 2MB / 4MB Flash)

| Diretório / Arquivo PDF | Categoria | Descrição Técnica & Modelo | Escopo & Diagnósticos |
| :--- | :--- | :--- | :--- |
| `dmr-e55/user/panasonic_dmr_e55_operating_instructions.pdf` | Manual do Usuário | **Panasonic DMR-E55** | Gravador DVD-RAM 2 MB Flash (MN10300). NVRAM em `0x1F0000`. |
| `dmr-e85h/user/panasonic_dmr_e85h_user_manual.pdf` | Manual do Usuário | **Panasonic DMR-E85H** | Gravador pioneiro HDD+DVD 4 MB Flash. NVRAM em `0x3F0000` (Erro U99). |
| `dmr-e500h/user/panasonic_dmr_e500h_user_manual.pdf` | Manual do Usuário | **Panasonic DMR-E500H** | Gravador premium de rede e HDD massivo (400 GB) da geração E. |

---

### 5. Família Panasonic DMR-EX98V (Geração DVB-T / HDMI 16MB Flash)

| Diretório / Arquivo PDF | Categoria | Descrição Técnica & Modelo | Escopo & Diagnósticos |
| :--- | :--- | :--- | :--- |
| `dmr-ex98v/service/panasonic_dmr_ex98v_service_manual_vol1.pdf` | **Manual de Serviço** | **Panasonic DMR-EX98V** | Manual de serviço Vol. 1 com diagramas de blocos DVB-T e PCB digital. |

---

### 6. Datasheets e Manuais de Arquitetura de Silício (`datasheets/`)

| Arquivo PDF | Componente / Domínio | Fabricante | Descrição Técnica & Relevância de Bancada |
| :--- | :--- | :--- | :--- |
| `datasheets/matsushita_mn10300_series_cross_assembler_users_manual.pdf` | **CPU AM34 / MN10300** | Matsushita / Panasonic | Manual do Cross-Assembler oficial: conjunto de instruções, modos de endereçamento e convenções de registradores. |
| `datasheets/matsushita_mn10300_series_c_debugger_users_manual.pdf` | **Depurador OCD MN10300** | Matsushita / Panasonic | Manual de depuração em nível C e protocolo On-Chip Debug (OCD) via barramento síncrono serial. |
| `datasheets/rp2040_datasheet_raspberrypi.pdf` | **Microcontrolador RP2040** | Raspberry Pi Foundation | Especificação do hardware RP2040, temporização das máquinas de estado PIO0/PIO1 e registradores GPIO. |
| `datasheets/rp2350_datasheet_raspberrypi.pdf` | **Microcontrolador RP2350** | Raspberry Pi Foundation | Especificação do novo SoC RP2350 (Pico 2), melhorias de PIO, periféricos e segurança. |
| `datasheets/spansion_am29lv640d_nor_flash_datasheet.pdf` | **Flash NOR 64 Mbit (8 MB)** | Spansion / AMD | Manual da Flash NOR paralela TSOP-48: comandos JEDEC (AutoSelect, Sector Erase, Byte/Word Program) e tempos de ciclo. |
| `datasheets/spansion_s29gl128n_nor_flash_datasheet.pdf` | **Flash NOR 128 Mbit (16 MB)** | Spansion | Manual da Flash NOR MirrorBit para a série DMR-EX: arquitetura de setores de 128 KB e modo de leitura por página. |

---

## 🔍 Busca Global nos Manuais com a Ferramenta Nativa `panadiga`

Você pode indexar e buscar termos diagnósticos diretamente em todos os manuais acima usando o comando nativo:

```bash
# Busca global por código de erro U99 em todos os modelos:
panadiga manuals --query "U99"

# Busca filtrada por modelo específico (ex: DMR-ES10):
panadiga manuals --query "U99" --filter "dmr-es10"

# Busca por conector de depuração em esquemáticos:
panadiga manuals --query "P7501"
```
