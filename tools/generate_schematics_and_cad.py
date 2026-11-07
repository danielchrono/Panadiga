"""
Tools: Panadiga Professional Engineering Drawings & Schematics Generator
Generates standards-compliant electrical schematics (IEEE Std 315 / IEC 60617)
and precise mechanical connector layout drawings for JST ZR B6B-ZR-3R4 / P7501.
"""

import math
import os
import matplotlib.pyplot as plt
import matplotlib.patches as patches


def draw_ieee_resistor(
    ax: plt.Axes,
    x1: float,
    y1: float,
    x2: float,
    y2: float,
    label: str = "",
    value: str = "",
    opt_text: str = "",
    color: str = "black",
    lw: float = 1.3
) -> None:
    """Draws a standard IEEE zigzag resistor between (x1, y1) and (x2, y2)."""
    dx = x2 - x1
    dy = y2 - y1
    length = math.hypot(dx, dy)
    angle = math.atan2(dy, dx)

    lead = 0.22 * length
    body = 0.56 * length
    n_peaks = 6
    w = 0.15

    pts: list[tuple[float, float]] = [(0.0, 0.0), (lead, 0.0)]
    step = body / n_peaks
    for i in range(n_peaks):
        sign = 1 if i % 2 == 0 else -1
        pts.append((lead + (i + 0.5) * step, sign * w))
    pts.append((lead + body, 0.0))
    pts.append((length, 0.0))

    cos_a = math.cos(angle)
    sin_a = math.sin(angle)
    rot_pts = [(x1 + px * cos_a - py * sin_a, y1 + px * sin_a + py * cos_a) for px, py in pts]

    xs, ys = zip(*rot_pts)
    ax.plot(xs, ys, color=color, linewidth=lw, solid_capstyle='round')

    mid_x = (x1 + x2) / 2.0
    mid_y = (y1 + y2) / 2.0
    off_x = -sin_a * 0.24
    off_y = cos_a * 0.24

    # Text above resistor
    if label or value:
        top_lbl = f"{label}: {value}" if (label and value) else (label or value)
        ax.text(mid_x + off_x, mid_y + off_y, top_lbl, fontsize=6.8, fontweight='bold',
                color=color, ha='center', va='bottom', fontfamily='monospace')

    # Text below resistor
    if opt_text:
        ax.text(mid_x - off_x, mid_y - off_y, opt_text, fontsize=5.8, fontweight='bold',
                color='#475569', ha='center', va='top', fontfamily='monospace')


def draw_chassis_ground(ax: plt.Axes, x: float, y: float, color: str = "black", lw: float = 1.4) -> None:
    """Draws standard IEEE chassis ground symbol (3 diagonal lines)."""
    ax.plot([x, x], [y, y - 0.20], color=color, linewidth=lw)
    ax.plot([x - 0.20, x + 0.20], [y - 0.20, y - 0.20], color=color, linewidth=lw)
    for off in [-0.16, 0.0, 0.16]:
        ax.plot([x + off, x + off - 0.10], [y - 0.20, y - 0.35], color=color, linewidth=lw)


def draw_signal_ground(ax: plt.Axes, x: float, y: float, color: str = "black", lw: float = 1.4) -> None:
    """Draws standard IEEE signal ground symbol (triangle)."""
    ax.plot([x, x], [y, y - 0.16], color=color, linewidth=lw)
    tri = patches.Polygon([[x - 0.16, y - 0.16], [x + 0.16, y - 0.16], [x, y - 0.35]],
                          closed=True, edgecolor=color, facecolor='none', linewidth=lw)
    ax.add_patch(tri)


def create_drawing_canvas() -> tuple[plt.Figure, plt.Axes]:
    """Instantiates a standardized white drawing sheet for IEEE engineering drawings."""
    fig, ax = plt.subplots(figsize=(16, 10), dpi=300)
    fig.patch.set_facecolor('#ffffff')
    ax.set_facecolor('#ffffff')
    return fig, ax


def setup_title_block(
    ax: plt.Axes,
    header_org: str,
    header_sub: str,
    title_text: str,
    app_text: str,
    rev_text: str,
    date_scale: str
) -> None:
    """Renders standardized ISO/IEEE border and engineering title block in footer area."""
    # Outer border at y=0.45 to y=9.55
    ax.plot([0.5, 15.5, 15.5, 0.5, 0.5], [0.45, 0.45, 9.55, 9.55, 0.45], color='#000000', linewidth=2.0)
    # Inner border at y=0.58 to y=9.42
    ax.plot([0.65, 15.35, 15.35, 0.65, 0.65], [0.58, 0.58, 9.42, 9.42, 0.58], color='#333333', linewidth=0.8)

    # Title block rectangle (strictly bounded between y=0.58 and y=1.75)
    tb = patches.Rectangle((9.5, 0.58), 5.85, 1.17, edgecolor='#000000', facecolor='#f8fafc', linewidth=1.2)
    ax.add_patch(tb)
    ax.plot([9.5, 15.35], [1.36, 1.36], color='#000000', linewidth=0.8)
    ax.plot([9.5, 15.35], [0.97, 0.97], color='#000000', linewidth=0.8)
    ax.plot([12.8, 12.8], [0.58, 1.36], color='#000000', linewidth=0.8)

    ax.text(9.65, 1.54, f"{header_org} | {header_sub}", fontsize=6.8, fontweight='bold', fontfamily='monospace')
    ax.text(9.65, 1.16, title_text, fontsize=7.2, fontweight='bold', fontfamily='sans-serif')
    ax.text(9.65, 0.77, app_text, fontsize=6.8, fontfamily='sans-serif')
    ax.text(12.95, 1.16, rev_text, fontsize=6.8, fontweight='bold', fontfamily='monospace')
    ax.text(12.95, 0.77, date_scale, fontsize=6.8, fontfamily='monospace')


def finalize_and_save_drawing(fig: plt.Figure, ax: plt.Axes, output_path: str) -> None:
    """Closes and persists the vector figure to disk."""
    ax.set_xlim(0, 16)
    ax.set_ylim(0, 10)
    ax.axis('off')
    plt.tight_layout()
    abs_path = os.path.abspath(output_path)
    plt.savefig(abs_path, dpi=300, bbox_inches='tight', facecolor=fig.get_facecolor())
    plt.close(fig)
    print(f'[OK] Generated {abs_path}')


def generate_electrical_schematic(output_path: str) -> None:
    """Generates standard IEEE Std 315 / IEC 60617 electrical wiring schematic."""
    fig, ax = create_drawing_canvas()
    setup_title_block(
        ax,
        header_org='PANADIGA BENCH LAB — RIGOROUS IEEE SCHEMATIC',
        header_sub='IEEE Std 315 / IEC 60617',
        title_text='TITLE: Pico RP2040 to Panasonic P7501 Probe',
        app_text='TARGET: VEP79138 (DMR-EH55) / MN2DS0016',
        rev_text='REV: 2.4 (Header Frame Clearance Verified)',
        date_scale='DATE: 2026-11-07 | SCALE: NTS'
    )

    # Drawing Header (Top) — Cleanly positioned inside inner border (y=9.42)
    # y=9.05 leaves 0.37 units of pure margin above the text!
    ax.text(8.0, 9.05, 'ESQUEMÁTICO ELÉTRICO DE INTERFACE DE BANCADA (DWIRE32A / OCD)',
            fontsize=11.5, fontweight='bold', ha='center', fontfamily='monospace')
    ax.text(8.0, 8.75, 'Conexão Homologada: Raspberry Pi Pico (RP2040) <---> Conector Panasonic P7501 [B6B-ZR-3R4]',
            fontsize=8.0, color='#475569', ha='center', fontfamily='sans-serif')

    # RP2040 Box (Left): y from 2.05 to 8.25 (Height: 6.20)
    rp_box = patches.Rectangle((0.85, 2.05), 3.25, 6.20, edgecolor='#0f172a', facecolor='#f1f5f9', linewidth=1.5)
    ax.add_patch(rp_box)
    ax.text(2.47, 8.00, 'CONTROLE DE BANCADA\nRASPBERRY PI PICO\n(RP2040 3.3V LVTTL)',
            fontsize=8.0, fontweight='bold', ha='center', fontfamily='monospace')

    # P7501 Box (Right): y from 2.05 to 8.25 (Height: 6.20)
    target_box = patches.Rectangle((11.90, 2.05), 3.45, 6.20, edgecolor='#0f172a', facecolor='#f1f5f9', linewidth=1.5)
    ax.add_patch(target_box)
    ax.text(13.62, 8.00, 'PLACA DIGITAL VEP79138\nCONECTOR P7501\n(JST ZR 1.5mm B6B-ZR)',
            fontsize=8.0, fontweight='bold', ha='center', fontfamily='monospace')

    # Sequential pin layout: Pin 1 (Top, y=7.40) down to Pin 6 (Bottom, y=2.40)
    # Clearance above footer (y=1.75): 2.40 - 1.75 = 0.65 units!
    nets = [
        # (Y, Pico Pin, Pico Label, Resistor Name, Resistor Val, Resistor Opt, HasPullup, Target Pin Label, Target Func Label, IsGnd, IsOD)
        (7.40, 'Pino 31 (ADC0)', 'GP26', 'R5', '1.0 kΩ', '[OPCIONAL / FILTRO ADC]', False,
         'Pino 1 (Manual) / Pino 6 (Físico)', 'VDD Sense (+3.3V LVTTL)', False, False),
        
        (6.40, 'Pino 7 (GPIO)', 'GP5', 'R1', '100 Ω [OPCIONAL]', '', False,
         'Pino 2 (Manual) / Pino 5 (Físico)', 'nRESET (XRST - Modo Open-Drain)', False, True),
        
        (5.40, 'Pino 6 (GPIO)', 'GP4', 'R2', '100 Ω', '[OPCIONAL / JUMPER DIRETO]', True,
         'Pino 3 (Manual) / Pino 4 (Físico)', 'SDATA (Half-Duplex Bidirecional)', False, False),
        
        (4.40, 'Pino 38 (GND)', 'GND COMUM', None, None, None, False,
         'Pino 4 (Manual) / Pino 3 (Físico)', 'GND (Terra Chassi - 0.0Ω Metálico)', True, False),
        
        (3.40, 'Pino 5 (GPIO)', 'GP3', 'R3', '100 Ω', '[OPCIONAL / JUMPER DIRETO]', False,
         'Pino 5 (Manual) / Pino 2 (Físico)', 'SCLK (Clock Master 1.0 MHz)', False, False),
        
        (2.40, 'Pino 4 (GPIO)', 'GP2', 'R4', '100 Ω', '[OPCIONAL / JUMPER DIRETO]', False,
         'Pino 6 (Manual) / Pino 1 (Físico)', 'EXTRG (Gatilho OCD - Pad Quadrado)', False, False),
    ]

    for y, p_pin, p_lbl, r_name, r_val, r_opt, has_pu, t_pin, t_lbl, is_gnd, is_od in nets:
        # Pico Pin terminal and text
        ax.plot([4.10, 4.70], [y, y], color='#000000', linewidth=1.4)
        ax.plot(4.10, y, marker='o', markersize=3.5, color='#000000')
        ax.text(3.95, y, f"{p_pin}\n{p_lbl}", fontsize=7.0, fontweight='bold', ha='right', va='center', fontfamily='monospace')

        # Target Pin terminal and text
        ax.plot([11.30, 11.90], [y, y], color='#000000', linewidth=1.4)
        ax.plot(11.90, y, marker='o', markersize=3.5, color='#000000')
        ax.text(12.05, y, f"{t_pin}\n{t_lbl}", fontsize=7.0, fontweight='bold', ha='left', va='center', fontfamily='monospace')

        if is_gnd:
            # Direct ground bar
            ax.plot([4.70, 11.30], [y, y], color='#000000', linewidth=1.8)
            draw_signal_ground(ax, 5.10, y, color='#000000', lw=1.2)
            draw_chassis_ground(ax, 10.90, y, color='#000000', lw=1.4)
            ax.text(8.00, y + 0.22, 'MALHA DE TERRA EQUIPOTENCIAL COMUM (0.0Ω COM A LATA DO CHASSI)',
                    fontsize=7.0, fontweight='bold', color='#15803d', ha='center', fontfamily='monospace')
            ax.text(8.00, y - 0.22, 'ESQUEMA OFICIAL: PINO 4  <===>  CONTAGEM FÍSICA (10V -> 8V): PINO 3',
                    fontsize=7.2, fontweight='bold', color='#b91c1c', ha='center', fontfamily='monospace')
        else:
            # Series Resistor branch: resistor centered between x=6.20 and x=7.60
            ax.plot([4.70, 6.20], [y, y], color='#000000', linewidth=1.4)
            draw_ieee_resistor(ax, 6.20, y, 7.60, y, label=r_name or "", value=r_val or "",
                               opt_text=r_opt or "", color='#000000', lw=1.3)
            ax.plot([7.60, 11.30], [y, y], color='#000000', linewidth=1.4)

            if has_pu:
                # Place Pull-up branch at x=9.80, text offset to the right so it never overlaps the zigzag
                pu_x = 9.80
                ax.plot(pu_x, y, marker='o', markersize=4, color='#000000')
                ax.plot([pu_x, pu_x], [y, y + 0.35], color='#000000', linewidth=1.3)
                draw_ieee_resistor(ax, pu_x, y + 0.35, pu_x, y + 1.15, label="", value="",
                                   opt_text="", color='#000000', lw=1.3)
                ax.plot([pu_x, pu_x], [y + 1.15, y + 1.40], color='#000000', linewidth=1.3)
                ax.plot([pu_x - 0.20, pu_x + 0.20], [y + 1.40, y + 1.40], color='#000000', linewidth=1.8)
                ax.text(pu_x + 0.25, y + 0.75, 'R_pu: 4.7 kΩ\n[RECOMENDADO / PULL-UP]',
                        fontsize=6.2, fontweight='bold', ha='left', va='center', fontfamily='monospace', linespacing=1.1)
                ax.text(pu_x, y + 1.55, '+3.3V (VREG / Pino 36)', fontsize=6.8, fontweight='bold', ha='center', fontfamily='monospace')

            if is_od:
                # Placed below resistor R1 with clean vertical clearance
                ax.text(6.90, y - 0.36, 'MODO OPEN-DRAIN: GP5 DRENA EXCLUSIVAMENTE PARA 0V; LIBERA EM Hi-Z\n'
                                        'JAMAIS FORÇAR +3.3V PUSH-PULL (RISCO DE CURTO COM CIRCUITO DE RESET DA PLACA)',
                        fontsize=5.8, fontweight='bold', color='#c2410c', ha='center', fontfamily='monospace', linespacing=1.15)

    # Engineering Notes Box (Bottom Left): strictly within y=0.58 to y=1.75 (Height: 1.17)
    nt_box = patches.Rectangle((0.85, 0.58), 8.50, 1.17, edgecolor='#b91c1c', facecolor='#fef2f2', linewidth=1.2)
    ax.add_patch(nt_box)
    ax.text(0.98, 1.62, 'NOTAS DE ENGENHARIA DE BANCADA E DIRETRIZ DE LIGAÇÃO (RESISTORES vs FIO DIRETO):',
            fontsize=7.0, fontweight='bold', color='#991b1b', fontfamily='monospace')
    notes_str = (
        '• VEREDITO DE BANCADA (COM OU SEM RESISTORES): Para testes rápidos em bancada com cabos curtos (L <= 15cm), todos os pinos de sinal\n'
        '  podem ser ligados por FIO DIRETO (jumpers diretos). O RP2040 e a CPU UniPhier operam ambos em 3.3V LVTTL compatíveis nativamente.\n'
        '• USO RECOMENDADO DOS RESISTORES: R1..R4 (100Ω) evitam repiques (ringing) e protegem contra curto acidental; R_pu (4.7kΩ) garante\n'
        '  borda de subida limpa (<200ns) em 1.0MHz caso não utilize o pull-up interno do Pico; R5 (1.0kΩ) protege o conversor analógico ADC0.\n'
        '• IDENTIFICAÇÃO DO TERRA: No manual de serviço, GND = PINO 4. Na contagem física 10V->8V, GND = PINO 3 (0.0Ω com a carcaça de lata).'
    )
    ax.text(0.98, 1.48, notes_str, fontsize=6.2, color='#7f1d1d', ha='left', va='top', fontfamily='sans-serif', linespacing=1.22)

    finalize_and_save_drawing(fig, ax, output_path)


def generate_mechanical_cad_drawing(output_path: str) -> None:
    """Generates precise mechanical engineering CAD drawing of JST ZR B6B-ZR-3R4 header and PCB layout."""
    fig, ax = create_drawing_canvas()
    setup_title_block(
        ax,
        header_org='PANADIGA BENCH LAB — MECHANICAL CAD SPECIFICATION',
        header_sub='JST ZR 1.5mm Pitch',
        title_text='TITLE: JST ZR B6B-ZR-3R4 Pinout & Latch Detail',
        app_text='APPLICATION: VEP79138 Test Port P7501 Assembly',
        rev_text='REV: 2.4 (Header Frame Clearance Verified)',
        date_scale='UNITS: mm | SCALE: NTS'
    )

    # Header Title — Cleanly positioned inside inner border (y=9.42)
    ax.text(8.0, 9.05, 'DETALHAMENTO MECÂNICO DO CONECTOR JST ZR (B6B-ZR-3R4) E TOPOGRAFIA DE PCB',
            fontsize=11.5, fontweight='bold', ha='center', fontfamily='monospace')
    ax.text(8.0, 8.75, 'Geometria Física da Trava Plástica, Parede Grossa vs Fina, Chanfro do Pino 1 e Relação Esquemática vs Física',
            fontsize=8.0, color='#475569', ha='center', fontfamily='sans-serif')

    # SECTION 1: Top View of Connector (Mating Face)
    ax.text(4.8, 8.20, 'VISTA SUPERIOR: CONECTOR JST ZR (B6B-ZR-3R4) NA PLACA VEP79138',
            fontsize=8.5, fontweight='bold', ha='center', fontfamily='monospace')

    # Shroud Outer Body: 10.5mm x 4.5mm (y from 4.7 to 7.7)
    shroud = patches.Rectangle((1.0, 4.7), 7.6, 3.0, edgecolor='#000000', facecolor='#f8fafc', linewidth=2.0)
    ax.add_patch(shroud)

    # Reinforced Back Wall (Parede Traseira Mais Grossa - 0.60mm)
    back_wall = patches.Rectangle((1.0, 7.1), 7.6, 0.6, edgecolor='#000000', facecolor='#cbd5e1', linewidth=1.2)
    ax.add_patch(back_wall)
    ax.text(4.8, 7.4, 'PAREDE TRASEIRA REFORÇADA (GROSSA - 0.60mm) [VOLTADA P/ O INTERIOR DA PLACA]',
            fontsize=6.6, fontweight='bold', ha='center', va='center', fontfamily='sans-serif')

    # Front Wall with Locking Slot (Parede Frontal Mais Fina - 0.45mm)
    front_wall = patches.Rectangle((1.0, 4.7), 7.6, 0.45, edgecolor='#000000', facecolor='#e2e8f0', linewidth=1.2)
    ax.add_patch(front_wall)

    # Mechanical Locking Notch (Janela Central da Trava Plástica do Soquete ZHR-6)
    lock_notch = patches.Rectangle((3.8, 4.65), 2.0, 0.55, edgecolor='#b91c1c', facecolor='#fee2e2', linewidth=1.5)
    ax.add_patch(lock_notch)
    ax.text(4.8, 4.90, 'JANELA DE TRAVA (LOCKING NOTCH - BORDA FRONTAL)',
            fontsize=6.5, fontweight='bold', color='#b91c1c', ha='center', va='center', fontfamily='monospace')

    # Polarization Keying Ribs
    ax.plot([1.6, 1.6], [4.7, 5.4], color='#000000', linewidth=1.5)
    ax.plot([8.0, 8.0], [4.7, 5.4], color='#000000', linewidth=1.5)
    ax.text(1.6, 4.4, 'GUIA A', fontsize=6.5, fontweight='bold', ha='center', fontfamily='monospace')
    ax.text(8.0, 4.4, 'GUIA B', fontsize=6.5, fontweight='bold', ha='center', fontfamily='monospace')

    # Pin 1 Chamfer / Polarizing Bevel on plastic body (Chanfro Angular do Pino 1 a 45°)
    chamfer = patches.Polygon([[1.0, 7.1], [1.5, 7.7], [1.0, 7.7]], closed=True,
                              edgecolor='#b91c1c', facecolor='#fecaca', linewidth=1.5)
    ax.add_patch(chamfer)
    ax.text(1.0, 7.88, 'CHANFRO 45° (MARCADOR MECÂNICO PINO 1 FÍSICO)',
            fontsize=6.8, fontweight='bold', color='#b91c1c', ha='left', fontfamily='monospace')

    # 6 Pin Cavities (Pitch 1.5mm)
    pin_xs = [2.05, 3.15, 4.25, 5.35, 6.45, 7.55]
    pin_data = [
        # (Phys Pin, Man Pin, SigShort, SigSub, IsGnd)
        ('Pino 1 Fís', 'Pino 6 Man', 'EXTRG', '(OCD Break)', False),
        ('Pino 2 Fís', 'Pino 5 Man', 'SCLK', '(Master 1MHz)', False),
        ('Pino 3 Fís', 'Pino 4 Man', 'GND', '(0.0Ω Chassi)', True),
        ('Pino 4 Fís', 'Pino 3 Man', 'SDATA', '(Half-Duplex)', False),
        ('Pino 5 Fís', 'Pino 2 Man', 'nRESET', '(XRST Line)', False),
        ('Pino 6 Fís', 'Pino 1 Man', 'VDD', '(+3.3V Sense)', False),
    ]

    for i, px in enumerate(pin_xs):
        phys_p, man_p, sig_main, sig_sub, is_gnd = pin_data[i]
        if i == 0:
            # Pin 1 PCB Solder Pad: Strictly SQUARE PAD according to IPC standards
            pad_rect = patches.Rectangle((px - 0.25, 5.9), 0.5, 0.5, edgecolor='#b91c1c', facecolor='#fee2e2', linewidth=1.8)
            ax.add_patch(pad_rect)
            ax.text(px, 6.62, 'PAD QUADRADO\n(PINO 1 PCB)', fontsize=6.0, fontweight='bold', color='#b91c1c', ha='center', fontfamily='monospace')
        else:
            # Pins 2..6: Circular pads
            pad_color = '#dcfce7' if is_gnd else '#ffffff'
            edge_c = '#15803d' if is_gnd else '#000000'
            pad_circ = patches.Circle((px, 6.15), 0.25, edgecolor=edge_c, facecolor=pad_color, linewidth=1.5)
            ax.add_patch(pad_circ)

        # Pin Metal Contact
        ax.plot(px, 6.15, marker='.', markersize=6, color='#000000')

        # Numbering and labels
        ax.text(px, 5.45, f"{i+1}", fontsize=8, fontweight='bold', ha='center', va='center', fontfamily='monospace')
        ax.text(px, 3.95, f"{phys_p}\n{man_p}", fontsize=6.5, fontweight='bold', ha='center', va='top', fontfamily='monospace')
        sig_color = '#15803d' if is_gnd else '#000000'
        ax.text(px, 3.15, f"{sig_main}\n{sig_sub}", fontsize=6.8, fontweight='bold', color=sig_color, ha='center', va='top', fontfamily='monospace')

    ax.annotate('', xy=(3.15, 6.15), xytext=(2.05, 6.15), arrowprops=dict(arrowstyle='<->', color='#2563eb', lw=1.2))
    ax.text(2.6, 6.25, 'p=1.50mm', fontsize=6.8, color='#2563eb', ha='center', fontweight='bold', fontfamily='monospace')

    # Highlight GND Column (Physical 3 = Manual 4)
    gnd_box = patches.Rectangle((3.80, 2.90), 0.90, 4.90, edgecolor='#15803d', facecolor='none', linewidth=1.8, linestyle='--')
    ax.add_patch(gnd_box)
    ax.text(4.25, 2.45, 'TERRA (GND)\n0.0Ω Chassi', fontsize=7.2, fontweight='bold', color='#15803d', ha='center', fontfamily='monospace')

    # SECTION 2: PCB Placement Context
    ax.text(12.42, 8.20, 'TOPOGRAFIA REAL NA PCB VEP79138', fontsize=8.5, fontweight='bold', ha='center', fontfamily='monospace')

    pcb_ctx = patches.Rectangle((9.5, 3.7), 5.85, 4.0, edgecolor='#000000', facecolor='#f1f5f9', linewidth=1.5)
    ax.add_patch(pcb_ctx)

    # Flat 10 Vias Socket (FP50003) on the left
    f10_sock = patches.Rectangle((9.7, 4.3), 1.0, 2.9, edgecolor='#000000', facecolor='#e2e8f0', linewidth=1.2)
    ax.add_patch(f10_sock)
    ax.text(10.2, 7.35, 'FP50003\n(Flat 10 Vias)', fontsize=6.8, fontweight='bold', ha='center', fontfamily='monospace')
    for yoff in range(10):
        ax.plot([9.8, 10.6], [4.5 + yoff * 0.26, 4.5 + yoff * 0.26], color='#475569', linewidth=1.0)

    # Flat 8 Vias Socket (FP50004) on the right
    f8_sock = patches.Rectangle((14.15, 4.3), 1.0, 2.5, edgecolor='#000000', facecolor='#e2e8f0', linewidth=1.2)
    ax.add_patch(f8_sock)
    ax.text(14.65, 6.95, 'FP50004\n(Flat 8 Vias)', fontsize=6.8, fontweight='bold', ha='center', fontfamily='monospace')
    for yoff in range(8):
        ax.plot([14.25, 15.05], [4.5 + yoff * 0.27, 4.5 + yoff * 0.27], color='#475569', linewidth=1.0)

    # Connector P7501 in Center of context
    p75_ctx = patches.Rectangle((11.1, 4.7), 2.6, 2.0, edgecolor='#b91c1c', facecolor='#ffffff', linewidth=1.5)
    ax.add_patch(p75_ctx)
    ax.text(12.4, 5.85, 'P7501 [B6B-ZR]\n"TO WRITE JIGU"', fontsize=7.2, fontweight='bold', color='#b91c1c', ha='center', fontfamily='monospace')
    ax.text(12.4, 5.15, 'Trava voltada para a borda da PCB\nParede grossa voltada p/ o miolo', fontsize=6.2, ha='center', fontfamily='sans-serif')

    ax.annotate('', xy=(13.9, 4.05), xytext=(10.9, 4.05), arrowprops=dict(arrowstyle='->', color='#000000', lw=1.5))
    ax.text(12.4, 3.88, 'CONTAGEM FÍSICA: DA ESQUERDA P/ A DIREITA (10V -> 8V)', fontsize=6.3, fontweight='bold', ha='center', fontfamily='monospace')

    # SECTION 3: Bottom Engineering Notes (strictly between y=0.58 and y=1.75)
    nt_box = patches.Rectangle((0.85, 0.58), 8.50, 1.17, edgecolor='#000000', facecolor='#f8fafc', linewidth=1.2)
    ax.add_patch(nt_box)
    ax.text(0.98, 1.62, 'REGRAS DE CONFORMIDADE MECÂNICA E IDENTIFICAÇÃO VISUAL DE BANCADA:',
            fontsize=7.0, fontweight='bold', fontfamily='monospace')
    cad_notes = (
        '1. PAREDE GROSSA vs FINA: A parede traseira (0.60mm) fica para dentro da placa; a parede frontal fina (0.45mm) abriga a trava.\n'
        '2. PINO 1 MECÂNICO: Identificado pelo chanfro a 45° no plástico e pelo pad QUADRADO na PCB. É o pino mais próximo do flat 10V.\n'
        '3. TERRA CANÔNICO (0.0Ω): O pino de terra (GND) é o PINO 4 do esquema e o PINO 3 da contagem física (entre flat 10V e flat 8V).\n'
        '4. PREVENÇÃO DE CURTO: O pino 6 físico (pino 1 do manual) é +3.3V VDD Sense. Jamais aterre este pino contra o chassi!'
    )
    ax.text(0.98, 1.48, cad_notes, fontsize=6.2, color='#334155', ha='left', va='top', fontfamily='sans-serif', linespacing=1.22)

    finalize_and_save_drawing(fig, ax, output_path)


if __name__ == '__main__':
    os.makedirs('docs/images', exist_ok=True)
    generate_electrical_schematic('docs/images/dwire32a_pico_probe_assembly_schematic.png')
    generate_mechanical_cad_drawing('docs/images/panasonic_dmr_eh55_pcb_p7501_orientation.png')
