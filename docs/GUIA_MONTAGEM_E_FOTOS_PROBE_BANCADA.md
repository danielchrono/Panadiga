# Guia Prático Ilustrado de Montagem da Sonda DWire32A e Procedimentos de Bancada
## Assembly da Ferramenta de Depuração In-Circuit para Panasonic DMR-EH55 (Placa VEP79138)

---

## 1. Visão Geral e Propósito de Engenharia

Este documento fornece as instruções detalhadas de montagem mecânica, elétrica e montagem de bancada para a sonda de diagnóstico **Matsushita DWire32A / On-Chip Debug (OCD)** utilizando o microcontrolador **Raspberry Pi Pico (RP2040)**.

A ferramenta permite:
1. Interceptar a inicialização a frio (*Cold Break-In*) do processador **UniPhier MN2DS0016** (núcleo AM34).
2. Depurar o firmware original via servidor **GDB RSP** na porta TCP 3333.
3. Injetar o microcódigo autônomo `ram_loader` na SRAM interna (`0x00004000`) para executar o apagamento cirúrgico dos microsetores de NVRAM `SA134..SA141` (`0x807F0000..0x807FFFFF`) na memória Flash Spansion `S29JL064H`, sanando definitivamente o erro **`U99`** sem necessidade de dessoldagem do encapsulamento TSOP-48.

---

## 2. AVISO CRÍTICO DE BANCADA: Identificação Inequívoca do Pino de Terra (GND)

> [!CAUTION]
> ### DISCLAIMER MANDATÓRIO DE BANCADA — PINO DO TERRA (`GND`)
> * **No Manual de Serviço Oficial da Panasonic (Esquemático P7501 [B6B-ZR-3R4], pág. 62):** O **`GND` é ESTRITAMENTE O PINO 4**.
> * **Na Contagem Física Visual (Do cabo flat de 10 vias `FP50003` para o de 8 vias `FP50004`):** O **`GND` corresponde ao PINO 3**.
> * **A REFERÊNCIA CANÔNICA E CORRETA DE ENGENHARIA É A DO MANUAL (Pino 4 = `GND`).**  
>   O desenhista do CAD da Panasonic espelhou a numeração dos pinos no conector do esquema em relação à contagem física da esquerda para a direita na placa:
>   $$\text{Pino Esquemático} = 6 - \text{Pino Físico} + 1$$
>   $$\text{Pino Físico do GND} = 6 - 4 + 1 = 3$$
> * **PERIGO DE CURTO-CIRCUITO DESTRUTIVO:** NUNCA assuma que o pino 6 é terra! No mapeamento físico, o pino 6 é a linha de alimentação `+3.3V VDD Sense`. Conectar o terra ali curto-circuita a barra principal de alimentação do silício UniPhier.
> * **REGRA DE OURO INVIOLÁVEL:** Antes de conectar o Raspberry Pi Pico ou qualquer fonte, **coloque o multímetro em teste de continuidade e confirme o bipe sonoro de $0.0\,\Omega$ entre o pino escolhido como terra e a carcaça de lata metálica desprovida de tinta do chassi do aparelho!**

---

## 3. Fotografias Técnicas e Topografia da Placa VEP79138

### 3.1 Esquemático Oficial do Conector P7501 (Manual de Serviço Panasonic)
Abaixo está o registro fotográfico de alta resolução extraído diretamente da página de esquemáticos da placa digital principal:

![Esquemático Oficial Panasonic P7501](images/schematic_p7501_panasonic_official.png)

*Detalhes visíveis no esquema oficial:*
- Conector identificado como **`P7501`**, part-number **`$[B6B-ZR-3R4]`**, legenda **`(TO WRITE JIGU)`**.
- **Pino 1:** `SW_UART / VDD`
- **Pino 2:** `XRESET` (`nRESET`)
- **Pino 3:** `RXD` (`SDATA` Half-Duplex)
- **Pino 4:** **`GND` (Terra com símbolo internacional de aterramento de chassi)**
- **Pino 5:** `NSW5V / SCLK` (`JCLK`)
- **Pino 6:** `TXD / EXTRG`

---

### 3.2 Topografia Física na Face Superior da Placa VEP79138
O conector ou ilhas SMD de teste situam-se na borda superior da placa digital, delimitados por dois cabos flat vitais:

![Topografia Física e Pinagem VEP79138](images/panasonic_dmr_eh55_pcb_p7501_orientation.png)

```text
       ORIENTAÇÃO FÍSICA NA PLACA VEP79138 (CONTAGEM DE 10V PARA 8V)
            ┌───────────┐
     Pino 1 │ ● EXTRG   │ Gatilho Externo OCD / Breakpoint [Pino 6 do Manual]
     Pino 2 │ ● SCLK    │ Clock Síncrono DWire32A (Master 1.0 MHz) [Pino 5 do Manual]
     Pino 3 │ ● GND     │ Ground Comum (0.0Ω para o Chassi Metálico!) [Pino 4 do Manual]
     Pino 4 │ ● SDATA   │ Dados Half-Duplex (Pull-up 4.7kΩ p/ 3.3V) [Pino 3 do Manual]
     Pino 5 │ ● nRESET  │ Linha de Reset Mestre (Controle Open-Drain) [Pino 2 do Manual]
     Pino 6 │ ● VDD     │ Target Power Sense (+3.3V Referência) [Pino 1 do Manual]
            └───────────┘
```

---

## 4. Esquema Elétrico de Montagem da Sonda (RP2040 $\leftrightarrow$ P7501)

Abaixo está o diagrama elétrico completo para confecção da placa adaptadora ou chicote de bancada:

![Esquema Elétrico de Montagem da Sonda Pico](images/dwire32a_pico_probe_assembly_schematic.png)

### 4.1 Tabela de Conexões e Componentes de Proteção

| Pino RP2040 (Pico) | Pino Esquemático P7501 (Manual) | Pino Físico (10V $\rightarrow$ 8V) | Sinal Matsushita | Rede de Proteção / Condicionamento | Comportamento Elétrico |
| :---: | :---: | :---: | :---: | :---: | :--- |
| **Pino 38 (`GND`)** | **Pino 4** | **Pino 3** | **`GND`** | **Ligação direta de baixa impedância** | **Referência de 0.0V ($0.0\,\Omega$ com a carcaça de lata do gravador).** |
| `GP4` (Pino 6) | Pino 3 | Pino 4 | `SDATA` | Resistor série de $100\,\Omega$ + Resistor pull-up de $4.7\text{ k}\Omega$ para 3.3V | Barramento bidirecional half-duplex síncrono. |
| `GP3` (Pino 5) | Pino 5 | Pino 2 | `SCLK` | Resistor série de $100\,\Omega$ | Linha de clock serial master gerada pelo RP2040 ($1.0\text{ MHz}$). |
| `GP2` (Pino 4) | Pino 6 | Pino 1 | `EXTRG` | Resistor série de $100\,\Omega$ | Gatilho externo para forçar interrupção OCD na CPU. |
| `GP5` (Pino 7) | Pino 2 | Pino 5 | `nRESET` | Resistor série de $100\,\Omega$ (**Configurado estritamente em Open-Drain**) | Linha de reset mestre `XRST`. **JAMAIS injetar 3.3V push-pull!** |
| `GP26 / ADC0` (Pino 31) | Pino 1 | Pino 6 | `VDD Sense` | Resistor série de $1.0\text{ k}\Omega$ | Sensor de tensão para garantir estabilidade da alimentação (+3.3V DC). |

---

## 5. Lista de Materiais (Bill of Materials — BOM)

Para montar a sonda de bancada com robustez profissional, utilize os seguintes componentes:

| Item | Qtd | Referência | Descrição Técnica | Função no Circuito |
| :---: | :---: | :--- | :--- | :--- |
| **1** | 1 | Microcontrolador | **Raspberry Pi Pico** (RP2040) ou Pico W | Controlador mestre PIO0 DWire32A e ponte GDB RSP |
| **2** | 4 | $R_1, R_2, R_3, R_4$ | Resistor de Filme Metálico **$100\,\Omega \pm 1\%$** ($1/8\text{W}$ ou $1/4\text{W}$) | Amortecimento de reflexões em `SDATA`, `SCLK`, `EXTRG`, `nRESET` |
| **3** | 1 | $R_{\text{pu}}$ | Resistor de Filme Metálico **$4.7\text{ k}\Omega \pm 1\%$** | Pull-up da linha `SDATA` para o barramento de 3.3V |
| **4** | 1 | $R_{\text{sense}}$ | Resistor de Filme Metálico **$1.0\text{ k}\Omega \pm 1\%$** | Proteção da porta analógica ADC0 (`GP26`) contra transientes |
| **5** | 1 | Chicote de Conexão | Conector JST ZR 6 vias fêmea (`ZHR-6`) passo 1.5mm **OU** Fios flexíveis AWG 30 (Kynar / PTFE) | Conexão ao conector original `P7501 [B6B-ZR-3R4]` |
| **6** | 1 | Fio de Destravamento | Fio flexível AWG 30 com Resistor de $1.0\text{ k}\Omega$ | Injeção temporária de 3.3V no pino 14 (`WP#`) da Flash Spansion se necessário |
| **7** | 1 | Fixação Mecânica | Fita Kapton (resistente a calor) ou Cola Quente de alívio de tensão | Fixação dos fios na PCB para evitar rompimento das trilhas SMD |

---

## 6. Procedimento de Montagem Passo a Passo

### Etapa 1: Preparação da Placa Digital VEP79138
1. Desconecte o cabo de energia AC do gravador Panasonic DMR-EH55.
2. Remova a tampa superior metálica (retirando os parafusos traseiros e laterais).
3. Localize visualmente a placa digital `VEP79138` e identifique o cabo flat de 10 vias `FP50003` e o cabo de 8 vias `FP50004`.
4. Entre os dois flats, localize o conector SMD `P7501` ou as 6 ilhas estanhadas adjacentes com a legenda *"TO WRITE JIGU"*.
5. Limpe as ilhas com álcool isopropílico 99.8% e aplique fluxo de solda no-clean.

### Etapa 2: Conexão Elétrica (Opção Conector ou Solda Direta)
* **Se a placa possuir o conector fêmea soldado:** Conecte diretamente o terminal JST `ZHR-6` crimpado.
* **Se a placa possuir apenas as ilhas de teste:**
  1. Estanhe suavemente as 6 ilhas com solda eutética Sn63/Pb37 ou Lead-Free.
  2. Solde 6 fios Kynar AWG 30 de aproximadamente 10 a 15 cm de comprimento.
  3. Prenda o feixe de fios na superfície da PCB com fita Kapton para prevenir esforço mecânico sobre as ilhas de cobre.

### Etapa 3: Checklist de Teste com Multímetro (OBRIGATÓRIO)
Antes de ligar o conector ao Raspberry Pi Pico:
1. **Teste de Continuidade do Terra:**
   - Ponta preta do multímetro no chassi de metal do aparelho.
   - Ponta vermelha no **Pino 3 físico (Pino 4 do manual)**.
   - O multímetro **DEVE emitir bipe sonoro ininterrupto indicando resistência $< 0.3\,\Omega$**.
   - Teste todos os outros 5 pinos contra o chassi: nenhum outro pino pode apitar!
2. **Teste de Ausência de Curto em VDD:**
   - Meça a resistência entre o Pino 6 físico (Pino 1 do manual) e o Pino 3 físico (GND).
   - A resistência deve ser alta (na faixa de kilo-ohms ou mega-ohms). Se acusar $0\,\Omega$, há ponte de solda!

### Etapa 4: Conexão ao Raspberry Pi Pico e Inicialização
1. Insira os resistores de proteção de $100\,\Omega$ em linha com os pinos de sinal conforme o esquema da Seção 4.
2. Ligue a ponta do terra ao pino 38 (`GND`) do Pico.
3. Conecte o cabo USB do Pico ao computador e inicie o Workbench do Panadiga Desktop:
   ```powershell
   npm run tauri dev
   ```
4. Navegue até o módulo **Pico Bench** e observe o painel **Conexões Físicas (Pico → CN55001)** para validar a leitura da tensão de referência de 3.3V no ADC.
