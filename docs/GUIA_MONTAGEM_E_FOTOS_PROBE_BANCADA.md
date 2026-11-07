# Guia Prático Ilustrado de Montagem da Sonda DWire32A e Procedimentos de Bancada
## Assembly da Ferramenta de Depuração In-Circuit para Panasonic DMR-EH55 (Placa VEP79138)

---

## 1. Visão Geral e Propósito de Engenharia

Este documento fornece as instruções detalhadas de montagem mecânica, elétrica e montagem de bancada para a sonda de diagnóstico **Matsushita DWire32A / On-Chip Debug (OCD)** utilizando o microcontrolador **Raspberry Pi Pico (RP2040)**.

A ferramenta permite:
1. Interceptar a inicialização a frio (*Cold Break-In*) do processador **UniPhier MN2DS0016** (núcleo AM34).
2. Depurar o firmware original via servidor **GDB RSP** na porta TCP 3333.
3. Injetar o microcódigo autônomo `ram_loader` na SRAM interna (`0x00004000`) para executar o apagamento cirúrgico dos microsetores de NVRAM `SA134..SA141` (`0x807F0000..0x807FFFFF`) na memória Flash Spansion `S29JL064H`, sanando definitivamente o erro **`U99`** sem necessidade de dessoldagem do encapsulamento TSOP-48.

> [!NOTE]
> ### Rigor Técnico de Bancada vs. Ilustrações Estilizadas (Refutação de Gráficos "PowerPoint")
> Ilustrações conceituais com tema escuro, efeitos visuais estilizados ou aproximações artísticas geradas por IA foram **terminantemente refutadas e descartadas desta documentação técnica**.  
> Em bancada de laboratório de hardware e microeletrônica, suposições visuais levam à queima de semicondutores. Todos os diagramas neste guia foram substituídos por:
> 1. **Desenhos Mecânicos CAD Baseados em Datasheet Oficial (JST ZR `B6B-ZR-3R4`)**: Geometria real da carcaça plástica, fenda central da trava, parede frontal fina vs. parede traseira reforçada grossa, chanfro a 45° no pino 1 e distinção de pads na PCB (pad quadrado no pino 1 vs. pads redondos nos pinos 2 a 6).
> 2. **Esquemáticos Elétricos em Conformidade com Normas IEEE Std 315 e IEC 60617**: Simbologia padrão internacional sobre papel técnico branco, blocos de título de engenharia (Title Block), rastreamento de nós, redes de amortecimento de impedância e clara distinção entre Terra de Sinal (IEEE Triangle) e Terra de Chassi Metálico (IEEE Chassis Ground).

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

### 3.2 Topografia Física na Face Superior da Placa VEP79138 e Especificação CAD JST ZR

O conector de teste **`P7501`** (ou suas ilhas SMD quando despopulado em unidades comerciais) situa-se na borda superior da placa digital `VEP79138`, delimitado com precisão milimétrica entre dois conectores flat vitais:
* **À Esquerda:** Conector flat de 10 vias **`FP50003`** (responsável pelo motor spindle da unidade óptica).
* **À Direita:** Conector flat de 8 vias **`FP50004`** (responsável pelo motor da gaveta e sensores de fim de curso).

Abaixo está o desenho mecânico vetorial de engenharia, extraído diretamente da especificação geométrica do fabricante JST e do footprint IPC-7351 da placa:

![Topografia Física e Pinagem VEP79138](images/panasonic_dmr_eh55_pcb_p7501_orientation.png)

#### Critérios Mecânicos e Invariantes Físicas do Conector JST ZR (B6B-ZR-3R4):
1. **Janela da Trava Plástica (Locking Notch)**:  
   A trava plástica do conector fêmea `ZHR-6` engata na **janela retangular vazada situada na parede frontal** (voltada para a borda externa da PCB). O clique de travamento deve ocorrer voltado para a borda frontal do chassi.
2. **Diferença de Parede (Parede Grossa vs. Parede Fina)**:  
   A carcaça de poliamida JST possui assimetria mecânica intencional:
   - **Parede Traseira (Grossa / Reforçada)**: Mede $0.60\text{ mm}$ de espessura de resina sólida, voltada para o miolo da placa VEP79138.
   - **Parede Frontal (Fina)**: Mede $0.45\text{ mm}$ de espessura para acomodar a flexão da lingüeta de retenção da trava do plugue `ZHR-6`.
3. **Chanfro Angular de 45° no Pino 1 (Molding Chamfer)**:  
   O canto externo plástico no lado do pino 1 possui um chanfro mecânico moldado a $45^\circ$, servindo como guia tátil e visual na montagem de bancada.
4. **Assinatura de PCB: Pad Quadrado no Pino 1 vs. Pads Redondos nos Pinos 2..6**:  
   Caso o conector plástico não esteja soldado na sua placa e você precise soldar fios diretamente nas ilhas de teste:
   - **Pino 1 Físico**: O pad de cobre é estritamente **QUADRADO** ($0.50 \times 0.50\text{ mm}$), indicando o pino 1 segundo a norma de fabricação IPC.
   - **Pinos 2 a 6 Físicos**: Os pads de cobre são estritamente **CIRCULARES / REDONDOS** ($\varnothing 0.50\text{ mm}$).
   - Isso elimina qualquer dúvida sobre qual lado é o pino 1, mesmo que a máscara de serigrafia esteja desgastada.
5. **Pitch (Passo Entre Pinos)**:  
   Exatos **$1.50\text{ mm}$** centro-a-centro. Comprimento total da carcaça plástica: $10.50\text{ mm}$.

```text
       ORIENTAÇÃO FÍSICA NA PLACA VEP79138 (CONTAGEM DA ESQUERDA P/ A DIREITA: 10V -> 8V)
            ┌───────────┐
     Pino 1 │ ■ EXTRG   │ [PAD QUADRADO] Gatilho Externo OCD / Break [Pino 6 do Manual]
     Pino 2 │ ● SCLK    │ [Pad Circular] Clock Síncrono DWire32A (Master 1.0 MHz) [Pino 5 do Manual]
     Pino 3 │ ● GND     │ [Pad Circular] Ground Comum (0.0Ω COM A LATA DO CHASSI!) [Pino 4 do Manual]
     Pino 4 │ ● SDATA   │ [Pad Circular] Dados Half-Duplex (Pull-up 4.7kΩ p/ 3.3V) [Pino 3 do Manual]
     Pino 5 │ ● nRESET  │ [Pad Circular] Linha de Reset Mestre (Open-Drain) [Pino 2 do Manual]
     Pino 6 │ ● VDD     │ [Pad Circular] Target Power Sense (+3.3V Referência) [Pino 1 do Manual]
            └───────────┘
```

---

## 4. Esquema Elétrico de Montagem da Sonda (RP2040 $\leftrightarrow$ P7501)

Abaixo está o diagrama elétrico oficial de engenharia, desenhado rigorosamente sob os padrões **IEEE Std 315 / IEC 60617** sobre papel milimetrado branco, com bloco de títulos e legendas normatizadas:

![Esquema Elétrico de Montagem da Sonda Pico](images/dwire32a_pico_probe_assembly_schematic.png)

#### Fundamentação Elétrica do Acoplamento & Veredito sobre Resistores:
1. **Veredito de Bancada: Precisa ou Não Ligar Resistores?**  
   - **MODO DIRETO (SEM RESISTORES / FIO DIRETO):**  
     Para testes rápidos de bancada com cabos curtos ($L \le 15\text{ cm}$), **NÃO É OBRIGATÓRIO instalar os resistores**. O RP2040 e a CPU Matsushita UniPhier operam nativamente na mesma tensão lógica de $3.3\text{V}$ LVTTL. Além disso, o microcódigo PIO do RP2040 ativa via software o pull-up interno no pino `SDATA` (`gpio_pull_up`), permitindo que a linha funcione com fio direto em cabos curtos.
   - **MODO PROTEGIDO (COM RESISTORES — RECOMENDADO EM FERRAMENTAS DEFINITIVAS):**  
     Para quem deseja montar uma ferramenta de bancada permanente e blindada:
     - **$R_1 \dots R_4 = 100\,\Omega$ [OPCIONAL / RECOMENDADO]:** Amortecem reflexões de alta frequência (*ringing*) causadas pela indutância dos cabos e limitam a corrente de transiente nos buffers CMOS caso haja colisão de barramento durante comutações de direção.
     - **$R_{\text{pu}} = 4.7\text{ k}\Omega$ [RECOMENDADO EM CABOS LONGOS]:** Garante subida rápida ($t_r < 200\text{ ns}$) no barramento bidirecional de 1.0 MHz, superando o pull-up interno fraco ($\approx 50\text{ k}\Omega$) do RP2040 em cabos acima de 15 cm.
     - **$R_{\text{sense}} = 1.0\text{ k}\Omega$ [OPCIONAL]:** Filtro passa-baixa e limitador de corrente para a entrada analógica ADC0 do Pico.
2. **Estágio Open-Drain em `nRESET` (`GP5`):**  
   O microcontrolador RP2040 comanda o pino `nRESET` estritamente drenando para $0\text{V}$ (nível baixo ativo) e soltando o pino em Alta Impedância (Hi-Z) para liberar a linha. **NUNCA configure o `GP5` como Push-Pull em nível alto (+3.3V)**, pois isso causaria curto com o supervisor de reset da placa VEP79138!
3. **Distinção de Terras Normatizada:**  
   O esquemático diferencia o Terra de Sinal do RP2040 (símbolo IEEE triangular) e o Terra de Chassi Metálico da Panasonic (símbolo IEEE de carcaça). A malha deve ser unificada em conexão equipotencial direta de baixíssima resistência ($< 0.1\,\Omega$).

### 4.1 Tabela de Conexões e Mapeamento Sequencial Pino a Pino (1 a 6)

| Pino Esquemático P7501 (Manual) | Pino Físico (10V $\rightarrow$ 8V) | Sinal Matsushita | Pino RP2040 (Pico) | Rede de Proteção (Bancada) | Veredito de Bancada |
| :---: | :---: | :---: | :---: | :---: | :--- |
| **Pino 1** | **Pino 6** | `VDD Sense` | `GP26 / ADC0` (Pino 31) | Resistor série de $1.0\text{ k}\Omega$ | **OPCIONAL** (Monitoramento da linha de 3.3V; pode ligar direto) |
| **Pino 2** | **Pino 5** | `nRESET` | `GP5` (Pino 7) | Resistor série de $100\,\Omega$ | **OPCIONAL** (Modo Open-Drain obrigatório; jumper direto OK) |
| **Pino 3** | **Pino 4** | `SDATA` | `GP4` (Pino 6) | Resistor $100\,\Omega$ + Pull-up $4.7\text{ k}\Omega$ | **RECOMENDADO** (Fio direto OK se cabo $\le 15\text{cm}$ com pull-up interno) |
| **Pino 4** | **Pino 3** | **`GND`** | **Pino 38 (`GND`)** | **Ligação direta de baixa impedância** | **MANDATÓRIO: $0.0\,\Omega$ com a lata metálica do chassi!** |
| **Pino 5** | **Pino 2** | `SCLK` | `GP3` (Pino 5) | Resistor série de $100\,\Omega$ | **OPCIONAL** (Clock master 1.0 MHz; jumper direto OK) |
| **Pino 6** | **Pino 1** | `EXTRG` | `GP2` (Pino 4) | Resistor série de $100\,\Omega$ | **OPCIONAL** (Pad quadrado na PCB; jumper direto OK) |

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
