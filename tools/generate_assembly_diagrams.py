"""
Tools: Panadiga Assembly Diagrams Generator
Generates high-resolution technical diagrams for bench hardware assembly and documentation:
1. panasonic_dmr_eh55_pcb_p7501_orientation.png
2. dwire32a_pico_probe_assembly_schematic.png
"""

import os
import matplotlib.pyplot as plt
import matplotlib.patches as patches


def finalize_and_save_plot(fig: plt.Figure, ax: plt.Axes, output_path: str, max_x: float) -> None:
    """Finalizes axis layout, exports high-resolution image, and releases memory."""
    ax.set_xlim(0, max_x)
    ax.set_ylim(0, 10)
    ax.axis('off')
    plt.tight_layout()
    plt.savefig(output_path, dpi=300, bbox_inches='tight', facecolor=fig.get_facecolor())
    plt.close()
    print(f'[OK] Generated {output_path}')


def generate_orientation_diagram(output_path: str) -> None:
    fig, ax = plt.subplots(figsize=(14, 8), dpi=300)
    fig.patch.set_facecolor('#0f172a')  # slate-900
    ax.set_facecolor('#0f172a')

    # Title & Subtitle
    ax.text(7.0, 9.5, 'PANASONIC DMR-EH55 — TOPOGRAFIA FISICA E PINAGEM DO CONECTOR P7501 / CN55001',
            fontsize=13, fontweight='bold', color='#f8fafc', ha='center', fontfamily='monospace')
    ax.text(7.0, 9.05, 'Mapeamento de Bancada na Placa Digital VEP79138: Contagem Esquematica do Manual vs. Contagem Fisica',
            fontsize=9.5, color='#94a3b8', ha='center', fontfamily='sans-serif')

    # Draw PCB Base
    pcb = patches.FancyBboxPatch((0.5, 2.5), 13.0, 6.0, boxstyle='round,pad=0.2',
                                 edgecolor='#1e293b', facecolor='#022c22', linewidth=2)  # dark green pcb
    ax.add_patch(pcb)
    ax.text(1.0, 8.1, 'PLACA DIGITAL VEP79138 (FACE DE COMPONENTES)', fontsize=10, fontweight='bold', color='#34d399', fontfamily='monospace')

    # Draw Flat 10 vias (FP50003) on the left
    f10 = patches.Rectangle((1.5, 4.0), 1.8, 3.5, linewidth=2, edgecolor='#e2e8f0', facecolor='#334155')
    ax.add_patch(f10)
    for i in range(10):
        ax.plot([1.6 + i*0.18, 1.6 + i*0.18], [4.1, 7.4], color='#cbd5e1', linewidth=1.5)
    ax.text(2.4, 3.6, 'FP50003\nFlat 10 Vias\n(Motor/Spindle)', fontsize=9, fontweight='bold', color='#93c5fd', ha='center', va='top', fontfamily='monospace')

    # Draw Flat 8 vias (FP50004) on the right
    f8 = patches.Rectangle((10.7, 4.0), 1.6, 3.5, linewidth=2, edgecolor='#e2e8f0', facecolor='#334155')
    ax.add_patch(f8)
    for i in range(8):
        ax.plot([10.8 + i*0.2, 10.8 + i*0.2], [4.1, 7.4], color='#cbd5e1', linewidth=1.5)
    ax.text(11.5, 3.6, 'FP50004\nFlat 8 Vias\n(Mec. Gaveta)', fontsize=9, fontweight='bold', color='#93c5fd', ha='center', va='top', fontfamily='monospace')

    # Draw P7501 connector in the center
    p7501 = patches.Rectangle((4.5, 4.2), 5.0, 3.0, linewidth=2, edgecolor='#f59e0b', facecolor='#1e293b')
    ax.add_patch(p7501)
    ax.text(7.0, 6.9, 'CONECTOR P7501 [B6B-ZR-3R4] (TO WRITE JIGU)', fontsize=11, fontweight='bold', color='#fbbf24', ha='center', fontfamily='monospace')
    ax.text(7.0, 6.5, 'Passo 1.5mm | 6 Vias em Linha | Face Superior da VEP79138', fontsize=8, color='#cbd5e1', ha='center')

    # 6 Pins
    pin_x = [5.0, 5.8, 6.6, 7.4, 8.2, 9.0]
    pin_labels = [
        {'phys': 'Pino 1', 'man': 'Pino 6', 'sig': 'EXTRG', 'desc': 'Break Trigger', 'is_gnd': False},
        {'phys': 'Pino 2', 'man': 'Pino 5', 'sig': 'SCLK',  'desc': 'Clock 1.0MHz',  'is_gnd': False},
        {'phys': 'Pino 3', 'man': 'Pino 4', 'sig': 'GND',   'desc': '0.0 ohm Chassi','is_gnd': True},
        {'phys': 'Pino 4', 'man': 'Pino 3', 'sig': 'SDATA', 'desc': 'Data Bidirec',  'is_gnd': False},
        {'phys': 'Pino 5', 'man': 'Pino 2', 'sig': 'nRESET','desc': 'Open-Drain',    'is_gnd': False},
        {'phys': 'Pino 6', 'man': 'Pino 1', 'sig': 'VDD',   'desc': '+3.3V Sense',   'is_gnd': False},
    ]

    for i, p in enumerate(pin_labels):
        x = pin_x[i]
        circle_color = '#22c55e' if p['is_gnd'] else ('#f43f5e' if p['sig'] == 'VDD' else '#38bdf8')
        circ = patches.Circle((x, 5.5), 0.25, facecolor=circle_color, edgecolor='#f8fafc', linewidth=2)
        ax.add_patch(circ)
        
        # Text inside pin
        ax.text(x, 5.5, str(i+1), fontsize=9, fontweight='bold', color='#0f172a', ha='center', va='center')
        
        # Manual numbering above
        man_color = '#86efac' if p['is_gnd'] else '#fde047'
        ax.text(x, 6.0, p['man'] + '\n(Manual)', fontsize=8, fontweight='bold', color=man_color, ha='center', va='bottom')
        
        # Signal below
        sig_weight = 'bold' if p['is_gnd'] else 'normal'
        ax.text(x, 4.9, p['sig'], fontsize=9, fontweight=sig_weight, color='#f8fafc', ha='center', va='top')
        ax.text(x, 4.5, p['desc'], fontsize=7, color='#94a3b8', ha='center', va='top')

    # Arrow indicating physical counting direction
    ax.annotate('', xy=(9.2, 7.6), xytext=(4.8, 7.6),
                arrowprops=dict(arrowstyle='->', color='#38bdf8', lw=2))
    ax.text(7.0, 7.8, 'CONTAGEM FISICA DE BANCADA (DO FLAT 10 VIAS PARA O FLAT 8 VIAS)',
            fontsize=8, fontweight='bold', color='#38bdf8', ha='center', fontfamily='monospace')

    # Big Alert / Disclaimer Box at bottom
    disc = patches.FancyBboxPatch((0.5, 0.35), 13.0, 1.85, boxstyle='round,pad=0.15',
                                  edgecolor='#ef4444', facecolor='#450a0a', linewidth=2)
    ax.add_patch(disc)
    ax.text(7.0, 1.85, '[!] DISCLAIMER DE SEGURANCA CRITICO -- IDENTIFICACAO DO TERRA (GND) [!]',
            fontsize=10.5, fontweight='bold', color='#f87171', ha='center', fontfamily='monospace')
    disc_text = (
        '1. NO MANUAL OFICIAL DA PANASONIC (Esquematico P7501): O TERRA (GND) E ESTRITAMENTE O PINO 4.\n'
        '2. NA CONTAGEM FISICA VISUAL (Do cabo flat de 10 vias para o de 8 vias): O TERRA (GND) CORRESPONDE AO PINO 3.\n'
        '3. O CORRETO DO PONTO DE VISTA DO MANUAL E O PINO 4. O CAD da Panasonic numerou a serigrafia em ordem invertida.\n'
        '4. REGRA DE OURO DE BANCADA: SEMPRE teste continuidade com multimetro (0.0 ohm com a carcaca de lata) antes de ligar o Pico!'
    )
    ax.text(7.0, 1.45, disc_text, fontsize=8.5, color='#fecaca', ha='center', va='top', fontfamily='sans-serif', linespacing=1.35)

    finalize_and_save_plot(fig, ax, output_path, 14.0)


def generate_assembly_schematic(output_path: str) -> None:
    fig, ax = plt.subplots(figsize=(15, 9), dpi=300)
    fig.patch.set_facecolor('#0b1120')  # deep navy slate
    ax.set_facecolor('#0b1120')

    # Main Titles
    ax.text(7.5, 9.6, 'ESQUEMA ELETRICO DE BANCADA: RASPBERRY PI PICO <---> PANASONIC CN55001 / P7501',
            fontsize=13, fontweight='bold', color='#f8fafc', ha='center', fontfamily='monospace')
    ax.text(7.5, 9.2, 'Circuito de Protecao e Condicionamento de Sinais Matsushita DWire32A (3.3V LVTTL)',
            fontsize=9.5, color='#94a3b8', ha='center', fontfamily='sans-serif')

    # Raspberry Pi Pico Box (Left)
    pico = patches.FancyBboxPatch((0.8, 2.2), 3.8, 6.5, boxstyle='round,pad=0.2',
                                  edgecolor='#0284c7', facecolor='#0f172a', linewidth=2)
    ax.add_patch(pico)
    ax.text(2.7, 8.35, 'RASPBERRY PI PICO\n(RP2040 / RP2350)', fontsize=11, fontweight='bold',
            color='#38bdf8', ha='center', fontfamily='monospace')
    ax.text(2.7, 7.8, 'Firmware C11 + PIO0 Machine\n(SCLK Master = 1.0 MHz)', fontsize=8, color='#94a3b8', ha='center')

    # Target Connector P7501 Box (Right)
    target = patches.FancyBboxPatch((10.4, 2.2), 3.8, 6.5, boxstyle='round,pad=0.2',
                                    edgecolor='#f59e0b', facecolor='#0f172a', linewidth=2)
    ax.add_patch(target)
    ax.text(12.3, 8.35, 'CONECTOR P7501\n(PANASONIC VEP79138)', fontsize=11, fontweight='bold',
            color='#fbbf24', ha='center', fontfamily='monospace')
    ax.text(12.3, 7.8, 'Matsushita UniPhier MN2DS0016\n(On-Chip Debug DWire32A)', fontsize=8, color='#94a3b8', ha='center')

    # Net Lines with Protections in the middle
    nets = [
        {
            'name': 'GND (Terra Comum)',
            'pico_pin': 'Pino 38 (GND)',
            'target_man': 'Pino 4 (Manual)',
            'target_phys': 'Pino 3 (Fisico 10V->8V)',
            'target_sig': 'GND (0.0 ohm Chassi)',
            'y': 6.8,
            'color': '#22c55e',
            'prot': 'Ligacao Direta de Baixa Impedancia',
            'is_gnd': True
        },
        {
            'name': 'SDATA (Dados Bidirecionais)',
            'pico_pin': 'GP4 (Pino 6)',
            'target_man': 'Pino 3 (Manual)',
            'target_phys': 'Pino 4 (Fisico 10V->8V)',
            'target_sig': 'SDATA',
            'y': 5.8,
            'color': '#38bdf8',
            'prot': 'Resistor 100 ohm + Pull-up 4.7k ohm p/ 3.3V',
            'is_gnd': False
        },
        {
            'name': 'SCLK (Clock Sincrono)',
            'pico_pin': 'GP3 (Pino 5)',
            'target_man': 'Pino 5 (Manual)',
            'target_phys': 'Pino 2 (Fisico 10V->8V)',
            'target_sig': 'SCLK',
            'y': 4.9,
            'color': '#38bdf8',
            'prot': 'Resistor 100 ohm em Serie',
            'is_gnd': False
        },
        {
            'name': 'EXTRG (External Trigger)',
            'pico_pin': 'GP2 (Pino 4)',
            'target_man': 'Pino 6 (Manual)',
            'target_phys': 'Pino 1 (Fisico 10V->8V)',
            'target_sig': 'EXTRG',
            'y': 4.0,
            'color': '#38bdf8',
            'prot': 'Resistor 100 ohm em Serie',
            'is_gnd': False
        },
        {
            'name': 'nRESET (Target Reset)',
            'pico_pin': 'GP5 (Pino 7)',
            'target_man': 'Pino 2 (Manual)',
            'target_phys': 'Pino 5 (Fisico 10V->8V)',
            'target_sig': 'nRESET (XRST)',
            'y': 3.1,
            'color': '#a855f7',
            'prot': 'Modo Open-Drain ESTRITO (Nunca 3.3V Push-Pull!)',
            'is_gnd': False
        },
        {
            'name': 'VDD Sense (+3.3V)',
            'pico_pin': 'GP26 / ADC0 (Pino 31)',
            'target_man': 'Pino 1 (Manual)',
            'target_phys': 'Pino 6 (Fisico 10V->8V)',
            'target_sig': 'VDD Sense (+3.3V)',
            'y': 2.2,
            'color': '#f43f5e',
            'prot': 'Resistor 1.0k ohm em Serie (Protecao ADC)',
            'is_gnd': False
        },
    ]

    for net in nets:
        y = net['y']
        c = net['color']

        # Pico Pin Box
        ax.text(4.4, y, net['pico_pin'], fontsize=8.5, fontweight='bold', color=c, ha='right', va='center', fontfamily='monospace')
        
        # Wire line left
        ax.plot([4.5, 5.5], [y, y], color=c, linewidth=2)
        
        # Protection Component Box in Center
        box_w = 3.6
        pbox = patches.FancyBboxPatch((5.5, y - 0.22), box_w, 0.44, boxstyle='round,pad=0.08',
                                      edgecolor=c, facecolor='#1e293b', linewidth=1.5)
        ax.add_patch(pbox)
        ax.text(7.3, y, net['prot'], fontsize=7.5, fontweight='bold', color='#f8fafc', ha='center', va='center')

        # Wire line right
        ax.plot([5.5 + box_w, 10.3], [y, y], color=c, linewidth=2)
        
        # Target Pin Details
        target_text = f"{net['target_man']} | {net['target_phys']} | {net['target_sig']}"
        ax.text(10.5, y, target_text, fontsize=8, fontweight='bold', color=c, ha='left', va='center', fontfamily='monospace')

    # Bottom Safety Box
    bot_box = patches.FancyBboxPatch((0.8, 0.3), 13.4, 1.45, boxstyle='round,pad=0.15',
                                     edgecolor='#ef4444', facecolor='#450a0a', linewidth=2)
    ax.add_patch(bot_box)
    ax.text(7.5, 1.45, '[!] AVISO CRITICO DE BANCADA -- GND / TERRA DO MANUAL (PINO 4) VS FISICO (PINO 3) [!]',
            fontsize=10.5, fontweight='bold', color='#f87171', ha='center', fontfamily='monospace')
    bot_text = (
        '* No Manual de Servico Oficial da Panasonic (P7501): O GND E O PINO 4. Na contagem fisica da esquerda (flat 10V para flat 8V): E O PINO 3.\n'
        '* A referencia canonica correta do projeto e a do MANUAL (Pino 4). NUNCA conecte terra no Pino 6 ou sem teste previo no multimetro!\n'
        '* ANTES DE LIGAR: Verifique o bipe sonoro de 0.0 ohm entre a ponta de prova e o chassi metalico de lata do gravador.'
    )
    ax.text(7.5, 1.15, bot_text, fontsize=8, color='#fecaca', ha='center', va='top', fontfamily='sans-serif', linespacing=1.3)

    finalize_and_save_plot(fig, ax, output_path, 15.0)


if __name__ == '__main__':
    os.makedirs('docs/images', exist_ok=True)
    generate_orientation_diagram('docs/images/panasonic_dmr_eh55_pcb_p7501_orientation.png')
    generate_assembly_schematic('docs/images/dwire32a_pico_probe_assembly_schematic.png')
