"""
Tools: Panadiga Professional Engineering Drawings & Schematics Generator
Generates standards-compliant electrical schematics (IEEE Std 315 / IEC 60617)
and precise mechanical connector layout drawings for JST ZR B6B-ZR-3R4 / P7501.
Supports Canonical English (default) and Portuguese (fallback).
"""

import math
import os
import shutil
import time
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
    date_scale: str,
    sheet_title: str = "",
    sheet_subtitle: str = ""
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

    if sheet_title:
        ax.text(8.0, 9.05, sheet_title, fontsize=11.5, fontweight='bold', ha='center', fontfamily='monospace')
    if sheet_subtitle:
        ax.text(8.0, 8.75, sheet_subtitle, fontsize=8.0, color='#475569', ha='center', fontfamily='sans-serif')


def finalize_and_save_drawing(fig: plt.Figure, ax: plt.Axes, output_path: str) -> None:
    """Closes and persists the vector figure to disk."""
    ax.set_xlim(0, 16)
    ax.set_ylim(0, 10)
    ax.axis('off')
    plt.tight_layout()
    abs_path = os.path.abspath(output_path)
    os.makedirs(os.path.dirname(abs_path), exist_ok=True)
    tmp_path = abs_path + '.tmp.png'
    try:
        plt.savefig(tmp_path, dpi=300, bbox_inches='tight', facecolor=fig.get_facecolor())
        plt.close(fig)
        # Safe atomic replace on Windows with retry
        for attempt in range(5):
            try:
                if os.path.exists(abs_path):
                    os.remove(abs_path)
                os.replace(tmp_path, abs_path)
                break
            except OSError:
                time.sleep(0.15)
        else:
            if os.path.exists(tmp_path):
                shutil.move(tmp_path, abs_path)
    finally:
        plt.close('all')
        if os.path.exists(tmp_path):
            try:
                os.remove(tmp_path)
            except OSError:
                pass
    print(f'[OK] Generated {abs_path}')


def _render_vertical_conductive_vias(ax: plt.Axes, x_coords: list[float]) -> None:
    """Draws vertical conductive FFC vias with contact pads and ribbon traces."""
    for vx in x_coords:
        ax.plot([vx, vx], [4.80, 6.60], color='#334155', linewidth=1.2)
        ax.plot(vx, 4.85, marker='s', markersize=2.2, color='#0f172a')
        ax.plot(vx, 6.55, marker='.', markersize=2.0, color='#64748b')


def generate_electrical_schematic(output_path: str, lang: str = 'en') -> None:
    """Generates standard IEEE Std 315 / IEC 60617 electrical wiring schematic."""
    fig, ax = create_drawing_canvas()

    is_en = (lang == 'en')
    header_org = 'PANADIGA BENCH LAB — RIGOROUS IEEE SCHEMATIC' if is_en else 'PANADIGA BENCH LAB — ESQUEMÁTICO IEEE RIGOROSO'
    header_sub = 'IEEE Std 315 / IEC 60617'
    title_text = 'TITLE: Pico RP2040 to Panasonic P7501 Probe' if is_en else 'TÍTULO: Sonda Pico RP2040 para Panasonic P7501'
    app_text = 'TARGET: VEP79138 (DMR-EH55) / MN2DS0016'
    rev_text = 'REV: 2.5 (Canonical Manual Pinout 1..6)'
    date_scale = 'DATE: 2026-11-07 | SCALE: NTS'

    main_hdr = ('BENCH ELECTRICAL INTERFACE SCHEMATIC (DWIRE32A / OCD)' if is_en
                else 'ESQUEMÁTICO ELÉTRICO DE INTERFACE DE BANCADA (DWIRE32A / OCD)')
    sub_hdr = ('Homologated Interface: Raspberry Pi Pico (RP2040) <---> Panasonic Port P7501 [B6B-ZR-3R4]' if is_en
               else 'Conexão Homologada: Raspberry Pi Pico (RP2040) <---> Conector Panasonic P7501 [B6B-ZR-3R4]')

    setup_title_block(
        ax, header_org, header_sub, title_text, app_text, rev_text, date_scale,
        sheet_title=main_hdr, sheet_subtitle=sub_hdr
    )

    # RP2040 Box (Left): y from 2.05 to 8.25
    rp_box = patches.Rectangle((0.85, 2.05), 3.25, 6.20, edgecolor='#0f172a', facecolor='#f1f5f9', linewidth=1.5)
    ax.add_patch(rp_box)
    rp_lbl = ('BENCH CONTROLLER\nRASPBERRY PI PICO\n(RP2040 3.3V LVTTL)' if is_en
              else 'CONTROLE DE BANCADA\nRASPBERRY PI PICO\n(RP2040 3.3V LVTTL)')
    ax.text(2.47, 8.00, rp_lbl, fontsize=8.0, fontweight='bold', ha='center', fontfamily='monospace')

    # P7501 Box (Right): y from 2.05 to 8.25
    target_box = patches.Rectangle((11.90, 2.05), 3.45, 6.20, edgecolor='#0f172a', facecolor='#f1f5f9', linewidth=1.5)
    ax.add_patch(target_box)
    tgt_lbl = ('DIGITAL BOARD VEP79138\nCONNECTOR P7501\n(JST ZR 1.5mm B6B-ZR)' if is_en
               else 'PLACA DIGITAL VEP79138\nCONECTOR P7501\n(JST ZR 1.5mm B6B-ZR)')
    ax.text(13.62, 8.00, tgt_lbl, fontsize=8.0, fontweight='bold', ha='center', fontfamily='monospace')

    # Sequential pin layout: Pin 1 (Top, y=7.40) down to Pin 6 (Bottom, y=2.40)
    # Strictly aligned with official Panasonic Service Manual Pinout (Pins 1..6)
    opt_tag = '[OPTIONAL / PROTOTYPE DIRECT]' if is_en else '[OPCIONAL / JUMPER DIRETO]'
    nets = [
        (7.40, 'Pin 31 (ADC0)' if is_en else 'Pino 31 (ADC0)', 'GP26', 'R5', '1.0 kΩ',
         '[OPTIONAL / ADC FILTER]' if is_en else '[OPCIONAL / FILTRO ADC]', False,
         'Pin 1: VDD Sense' if is_en else 'Pino 1: VDD Sense', '+3.3V Target Reference', False, False),

        (6.40, 'Pin 7 (GPIO)' if is_en else 'Pino 7 (GPIO)', 'GP5', 'R1', '100 Ω',
         '', False,
         'Pin 2: nRESET' if is_en else 'Pino 2: nRESET', 'Master XRST (Open-Drain, 0V)', False, True),

        (5.40, 'Pin 6 (GPIO)' if is_en else 'Pino 6 (GPIO)', 'GP4', 'R2', '100 Ω',
         opt_tag, True,
         'Pin 3: SDATA' if is_en else 'Pino 3: SDATA', 'Half-Duplex DWire32A I/O', False, False),

        (4.40, 'Pin 38 (GND)' if is_en else 'Pino 38 (GND)', 'GND COMMON' if is_en else 'GND COMUM', None, None, None,
         False,
         'Pin 4: GND' if is_en else 'Pino 4: GND', 'Chassis Ground (0.0Ω Bond)' if is_en else 'Terra Chassi (0.0Ω Chassi)', True, False),

        (3.40, 'Pin 5 (GPIO)' if is_en else 'Pino 5 (GPIO)', 'GP3', 'R3', '100 Ω',
         opt_tag, False,
         'Pin 5: SCLK' if is_en else 'Pino 5: SCLK', 'Master Clock 1.0 MHz', False, False),

        (2.40, 'Pin 4 (GPIO)' if is_en else 'Pino 4 (GPIO)', 'GP2', 'R4', '100 Ω',
         opt_tag, False,
         'Pin 6: EXTRG' if is_en else 'Pino 6: EXTRG', 'OCD Boot Trigger / Break', False, False),
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
            gnd_msg = ('EQUIPOTENTIAL COMMON GROUND BUS (0.0Ω TO CHASSIS ENCLOSURE)' if is_en
                       else 'MALHA DE TERRA EQUIPOTENCIAL COMUM (0.0Ω COM A LATA DO CHASSI)')
            can_msg = ('CANONICAL MANUAL PIN 4: GROUND CONTINUITY VERIFICATION TARGET' if is_en
                       else 'MANUAL CANÔNICO: PINO 4 = TERRA (PONTO DE TESTE CONTINUIDADE 0.0Ω)')
            ax.text(8.00, y + 0.22, gnd_msg, fontsize=7.0, fontweight='bold', color='#15803d', ha='center', fontfamily='monospace')
            ax.text(8.00, y - 0.22, can_msg, fontsize=7.2, fontweight='bold', color='#b91c1c', ha='center', fontfamily='monospace')
        else:
            # Series Resistor branch
            ax.plot([4.70, 6.20], [y, y], color='#000000', linewidth=1.4)
            draw_ieee_resistor(ax, 6.20, y, 7.60, y, label=r_name or "", value=r_val or "",
                               opt_text=r_opt or "", color='#000000', lw=1.3)
            ax.plot([7.60, 11.30], [y, y], color='#000000', linewidth=1.4)

            if has_pu:
                pu_x = 9.80
                ax.plot(pu_x, y, marker='o', markersize=4, color='#000000')
                ax.plot([pu_x, pu_x], [y, y + 0.35], color='#000000', linewidth=1.3)
                draw_ieee_resistor(ax, pu_x, y + 0.35, pu_x, y + 1.15, label="", value="",
                                   opt_text="", color='#000000', lw=1.3)
                ax.plot([pu_x, pu_x], [y + 1.15, y + 1.40], color='#000000', linewidth=1.3)
                ax.plot([pu_x - 0.20, pu_x + 0.20], [y + 1.40, y + 1.40], color='#000000', linewidth=1.8)
                pu_txt = ('R_pu: 4.7 kΩ\n[RECOMMENDED PULL-UP]' if is_en
                          else 'R_pu: 4.7 kΩ\n[RECOMENDADO / PULL-UP]')
                v_txt = '+3.3V (Pin 1 VDD / VREG)' if is_en else '+3.3V (Pino 1 VDD / VREG)'
                ax.text(pu_x + 0.25, y + 0.75, pu_txt, fontsize=6.2, fontweight='bold', ha='left', va='center', fontfamily='monospace', linespacing=1.1)
                ax.text(pu_x, y + 1.55, v_txt, fontsize=6.8, fontweight='bold', ha='center', fontfamily='monospace')

            if is_od:
                od_msg = ('OPEN-DRAIN MODE: GP5 STRICTLY SINKS TO 0V; RELEASES IN Hi-Z\n'
                          'NEVER DRIVE +3.3V PUSH-PULL (SHORT CIRCUIT HAZARD WITH BOARD RESET SUPERVISOR)' if is_en
                          else 'MODO OPEN-DRAIN: GP5 DRENA EXCLUSIVAMENTE PARA 0V; LIBERA EM Hi-Z\n'
                               'JAMAIS FORÇAR +3.3V PUSH-PULL (RISCO DE CURTO COM CIRCUITO DE RESET DA PLACA)')
                ax.text(6.90, y - 0.36, od_msg, fontsize=5.8, fontweight='bold', color='#c2410c', ha='center', fontfamily='monospace', linespacing=1.15)

    # Engineering Notes Box (Bottom Left)
    nt_box = patches.Rectangle((0.85, 0.58), 8.50, 1.17, edgecolor='#b91c1c', facecolor='#fef2f2', linewidth=1.2)
    ax.add_patch(nt_box)
    nt_hdr = ('BENCH ENGINEERING NOTES & WIRING DIRECTIVE (RESISTORS vs DIRECT WIRE):' if is_en
              else 'NOTAS DE ENGENHARIA DE BANCADA E DIRETRIZ DE LIGAÇÃO (RESISTORES vs FIO DIRETO):')
    ax.text(0.98, 1.62, nt_hdr, fontsize=7.0, fontweight='bold', color='#991b1b', fontfamily='monospace')

    if is_en:
        notes_str = (
            '• PROTOTYPE WIRING DISCLAIMER: For quick bench testing with short wires (L <= 15cm / 6in), all series damping\n'
            '  resistors (R1..R4) and ADC protection (R5) are ABSOLUTELY UNNECESSARY. RP2040 and UniPhier share native 3.3V LVTTL.\n'
            '• FINISHED PRODUCT RECOMMENDATION: For permanent fixtures or long harnesses, R1..R4 (100Ω) eliminate ringing, R_pu (4.7kΩ)\n'
            '  guarantees sharp rise times (<200ns) on SDATA, and R5 (1.0kΩ) isolates the ADC0 converter against ESD transients.\n'
            '• CANONICAL PINOUT: Pin 1 = VDD, Pin 2 = nRESET, Pin 3 = SDATA, Pin 4 = GND (0.0Ω to chassis), Pin 5 = SCLK, Pin 6 = EXTRG.'
        )
    else:
        notes_str = (
            '• DISCLAIMER DE PROTÓTIPO: Para testes rápidos em bancada com fios curtos (L <= 15cm), todos os resistores de amortecimento\n'
            '  (R1..R4) e proteção ADC (R5) são ABSOLUTAMENTE DESNECESSÁRIOS. RP2040 e UniPhier operam nativamente em 3.3V LVTTL.\n'
            '• RECOMENDAÇÃO EM PRODUTOS ACABADOS: Em ferramentas permanentes ou cabos longos, R1..R4 (100Ω) evitam repiques (ringing),\n'
            '  R_pu (4.7kΩ) garante subida limpa (<200ns) em SDATA, e R5 (1.0kΩ) isola o conversor ADC0 contra transientes ESD.\n'
            '• PINAGEM CANÔNICA: Pino 1 = VDD, Pino 2 = nRESET, Pino 3 = SDATA, Pino 4 = GND (0.0Ω ao chassi), Pino 5 = SCLK, Pino 6 = EXTRG.'
        )
    ax.text(0.98, 1.48, notes_str, fontsize=6.2, color='#7f1d1d', ha='left', va='top', fontfamily='sans-serif', linespacing=1.22)

    finalize_and_save_drawing(fig, ax, output_path)


def generate_mechanical_cad_drawing(output_path: str, lang: str = 'en') -> None:
    """Generates precise mechanical engineering CAD drawing of JST ZR B6B-ZR-3R4 header and PCB layout."""
    fig, ax = create_drawing_canvas()

    is_en = (lang == 'en')
    header_org = 'PANADIGA BENCH LAB — MECHANICAL CAD SPECIFICATION' if is_en else 'PANADIGA BENCH LAB — ESPECIFICAÇÃO CAD MECÂNICA'
    header_sub = 'JST ZR 1.5mm Pitch'
    title_text = 'TITLE: JST ZR B6B-ZR-3R4 Pinout & Latch Detail' if is_en else 'TÍTULO: Pinagem e Detalhes da Trava JST ZR B6B-ZR-3R4'
    app_text = 'APPLICATION: VEP79138 Test Port P7501 Assembly'
    rev_text = 'REV: 2.5 (Canonical Manual Pinout 1..6)'
    date_scale = 'UNITS: mm | SCALE: NTS'

    main_hdr = ('MECHANICAL CAD SPECIFICATION OF JST ZR (B6B-ZR-3R4) & PCB TOPOGRAPHY' if is_en
                else 'DETALHAMENTO MECÂNICO DO CONECTOR JST ZR (B6B-ZR-3R4) E TOPOGRAFIA DE PCB')
    sub_hdr = ('Latch Geometry, Thick vs Thin Shroud Wall, Pin 1 Reference & Canonical Panasonic Schematic Pinout' if is_en
               else 'Geometria da Trava Plástica, Parede Grossa vs Fina, Chanfro de Polarização e Pinagem Canônica Panasonic')

    setup_title_block(
        ax, header_org, header_sub, title_text, app_text, rev_text, date_scale,
        sheet_title=main_hdr, sheet_subtitle=sub_hdr
    )

    # SECTION 1: Top View of Connector (Mating Face)
    sec1_hdr = ('TOP VIEW: JST ZR HEADER (B6B-ZR-3R4) ON VEP79138 PCB' if is_en
                else 'VISTA SUPERIOR: CONECTOR JST ZR (B6B-ZR-3R4) NA PLACA VEP79138')
    ax.text(4.8, 8.20, sec1_hdr, fontsize=8.5, fontweight='bold', ha='center', fontfamily='monospace')

    # Shroud Outer Body: 10.5mm x 4.5mm (y from 4.7 to 7.7)
    shroud = patches.Rectangle((1.0, 4.7), 7.6, 3.0, edgecolor='#000000', facecolor='#f8fafc', linewidth=2.0)
    ax.add_patch(shroud)

    # Reinforced Back Wall (0.60mm)
    back_wall = patches.Rectangle((1.0, 7.1), 7.6, 0.6, edgecolor='#000000', facecolor='#cbd5e1', linewidth=1.2)
    ax.add_patch(back_wall)
    bw_txt = ('REINFORCED REAR WALL (THICK 0.60mm) [TOWARDS BOARD INTERIOR]' if is_en
              else 'PAREDE TRASEIRA REFORÇADA (GROSSA 0.60mm) [VOLTADA P/ O INTERIOR DA PLACA]')
    ax.text(4.8, 7.4, bw_txt, fontsize=6.6, fontweight='bold', ha='center', va='center', fontfamily='sans-serif')

    # Front Wall with Locking Slot (0.45mm)
    front_wall = patches.Rectangle((1.0, 4.7), 7.6, 0.45, edgecolor='#000000', facecolor='#e2e8f0', linewidth=1.2)
    ax.add_patch(front_wall)

    # Mechanical Locking Notch (Cutout for Female Plug Latch on Front Edge)
    lock_notch = patches.Rectangle((3.6, 4.65), 2.4, 0.55, edgecolor='#b91c1c', facecolor='#fee2e2', linewidth=1.5)
    ax.add_patch(lock_notch)
    notch_txt = ('LOCKING NOTCH\n(FRONT BOARD EDGE)' if is_en
                 else 'JANELA DE TRAVA\n(BORDA FRONTAL PCB)')
    ax.text(4.8, 4.92, notch_txt, fontsize=5.2, fontweight='bold', color='#b91c1c',
            ha='center', va='center', fontfamily='sans-serif', linespacing=1.05)

    # Polarization Keying Ribs
    ax.plot([1.6, 1.6], [4.7, 5.4], color='#000000', linewidth=1.5)
    ax.plot([8.0, 8.0], [4.7, 5.4], color='#000000', linewidth=1.5)
    ax.text(1.6, 4.4, 'KEY A' if is_en else 'GUIA A', fontsize=6.5, fontweight='bold', ha='center', fontfamily='monospace')
    ax.text(8.0, 4.4, 'KEY B' if is_en else 'GUIA B', fontsize=6.5, fontweight='bold', ha='center', fontfamily='monospace')

    # Pin 6 Chamfer (Left end)
    chamfer = patches.Polygon([[1.0, 7.1], [1.5, 7.7], [1.0, 7.7]], closed=True,
                              edgecolor='#b91c1c', facecolor='#fecaca', linewidth=1.5)
    ax.add_patch(chamfer)
    ch_lbl = ('45° BODY BEVEL (PIN 6 / EXTRG PAD)' if is_en
              else 'CHANFRO 45° (PAD PINO 6 / EXTRG)')
    ax.text(1.0, 7.88, ch_lbl, fontsize=6.8, fontweight='bold', color='#b91c1c', ha='left', fontfamily='monospace')

    # 6 Pin Cavities (Pitch 1.5mm) - Strictly Canonical Panasonic Manual Pinout (1 to 6)
    # Left end (x=2.05, near FP50003 10V) is Pin 6 (EXTRG). Right end (x=7.55, near FP50004 8V) is Pin 1 (VDD).
    pin_xs = [2.05, 3.15, 4.25, 5.35, 6.45, 7.55]
    pin_data = [
        ('Pin 6' if is_en else 'Pino 6', 'EXTRG', '(OCD Break)' if is_en else '(Gatilho OCD)', False, True),
        ('Pin 5' if is_en else 'Pino 5', 'SCLK', '(1.0 MHz)', False, False),
        ('Pin 4' if is_en else 'Pino 4', 'GND', '(0.0Ω Chassis)' if is_en else '(0.0Ω Chassi)', True, False),
        ('Pin 3' if is_en else 'Pino 3', 'SDATA', '(DWire32A)', False, False),
        ('Pin 2' if is_en else 'Pino 2', 'nRESET', '(XRST Line)' if is_en else '(Linha XRST)', False, False),
        ('Pin 1' if is_en else 'Pino 1', 'VDD', '(+3.3V Sense)' if is_en else '(+3.3V Ref)', False, False),
    ]

    for i, px in enumerate(pin_xs):
        p_num, sig_main, sig_sub, is_gnd, is_square = pin_data[i]
        if is_square:
            pad_rect = patches.Rectangle((px - 0.25, 5.9), 0.5, 0.5, edgecolor='#b91c1c', facecolor='#fee2e2', linewidth=1.8)
            ax.add_patch(pad_rect)
            pad_txt = 'SQUARE PAD\n(PCB PIN 6)' if is_en else 'PAD QUADRADO\n(PINO 6 PCB)'
            ax.text(px, 6.62, pad_txt, fontsize=6.0, fontweight='bold', color='#b91c1c', ha='center', fontfamily='monospace')
        else:
            pad_color = '#dcfce7' if is_gnd else '#ffffff'
            edge_c = '#15803d' if is_gnd else '#000000'
            pad_circ = patches.Circle((px, 6.15), 0.25, edgecolor=edge_c, facecolor=pad_color, linewidth=1.5)
            ax.add_patch(pad_circ)

        ax.plot(px, 6.15, marker='.', markersize=6, color='#000000')

        # Numbering and labels
        ax.text(px, 5.45, p_num.split()[-1], fontsize=8, fontweight='bold', ha='center', va='center', fontfamily='monospace')
        ax.text(px, 3.95, p_num, fontsize=6.5, fontweight='bold', ha='center', va='top', fontfamily='monospace')
        sig_color = '#15803d' if is_gnd else '#000000'
        ax.text(px, 3.15, f"{sig_main}\n{sig_sub}", fontsize=6.8, fontweight='bold', color=sig_color, ha='center', va='top', fontfamily='monospace')

    ax.annotate('', xy=(3.15, 6.15), xytext=(2.05, 6.15), arrowprops=dict(arrowstyle='<->', color='#2563eb', lw=1.2))
    ax.text(2.6, 6.25, 'p=1.50mm', fontsize=6.8, color='#2563eb', ha='center', fontweight='bold', fontfamily='monospace')

    # Highlight GND Column (Canonical Pin 4)
    gnd_box = patches.Rectangle((3.80, 2.85), 0.90, 4.95, edgecolor='#15803d', facecolor='none', linewidth=1.8, linestyle='--')
    ax.add_patch(gnd_box)

    # SECTION 2: PCB Placement Context
    sec2_hdr = ('REAL TOPOGRAPHY ON VEP79138 PCB' if is_en else 'TOPOGRAFIA REAL NA PCB VEP79138')
    ax.text(12.425, 8.20, sec2_hdr, fontsize=8.5, fontweight='bold', ha='center', fontfamily='monospace')

    pcb_ctx = patches.Rectangle((9.5, 3.7), 5.85, 4.0, edgecolor='#000000', facecolor='#f1f5f9', linewidth=1.5)
    ax.add_patch(pcb_ctx)

    # Flat 10 Vias Socket (FP50003) on the left - vertical vias side-by-side
    f10_sock = patches.Rectangle((9.70, 4.60), 1.35, 2.25, edgecolor='#000000', facecolor='#e2e8f0', linewidth=1.2)
    ax.add_patch(f10_sock)
    f10_lbl = 'FP50003\n(Flat 10 Vias)' if is_en else 'FP50003\n(Flat 10 Vias)'
    ax.text(10.375, 7.00, f10_lbl, fontsize=6.8, fontweight='bold', ha='center', fontfamily='monospace')
    _render_vertical_conductive_vias(ax, [9.82 + i * 0.1233 for i in range(10)])

    # Connector P7501 in Center of context - Detailed Male Shrouded Header (B6B-ZR-3R4)
    p75_t1 = 'P7501 [B6B-ZR-3R4]'
    p75_t2 = '"TO WRITE JIGU" (MALE)' if is_en else '"TO WRITE JIGU" (MACHO)'
    ax.text(12.425, 7.30, p75_t1, fontsize=7.0, fontweight='bold', color='#b91c1c', ha='center', fontfamily='monospace')
    ax.text(12.425, 7.02, p75_t2, fontsize=6.2, fontweight='bold', color='#b91c1c', ha='center', fontfamily='monospace')

    # Shroud Outer Perimeter
    shroud = patches.Rectangle((11.25, 4.60), 2.35, 2.25, edgecolor='#b91c1c', facecolor='#f8fafc', linewidth=1.6)
    ax.add_patch(shroud)

    # Thick Rear Wall (0.40mm high in model, facing board interior)
    rear_wall = patches.Rectangle((11.25, 6.45), 2.35, 0.40, edgecolor='#b91c1c', facecolor='#cbd5e1', linewidth=1.0)
    ax.add_patch(rear_wall)
    wall_txt = 'THICK REAR WALL (INSIDE)' if is_en else 'PAREDE GROSSA (MIOLO)'
    ax.text(12.425, 6.65, wall_txt, fontsize=5.3, fontweight='bold', ha='center', va='center', fontfamily='sans-serif')

    # Thin Front Wall (0.30mm in model)
    front_wall = patches.Rectangle((11.25, 4.60), 2.35, 0.30, edgecolor='#b91c1c', facecolor='#e2e8f0', linewidth=1.0)
    ax.add_patch(front_wall)

    # Mechanical Locking Notch (Front Edge Cutout)
    notch = patches.Rectangle((12.125, 4.55), 0.60, 0.35, edgecolor='#b91c1c', facecolor='#fee2e2', linewidth=1.2)
    ax.add_patch(notch)
    notch_lbl = 'LATCH' if is_en else 'TRAVA'
    ax.text(12.425, 4.72, notch_lbl, fontsize=5.3, fontweight='bold', color='#b91c1c', ha='center', va='center', fontfamily='monospace')

    # Pin 6 Polarizing Chamfer (top-left)
    chamfer = patches.Polygon([[11.25, 6.45], [11.60, 6.85], [11.25, 6.85]], closed=True,
                              edgecolor='#b91c1c', facecolor='#fecaca', linewidth=1.0)
    ax.add_patch(chamfer)

    # Inner Shroud Cavity
    cavity = patches.Rectangle((11.38, 4.90), 2.09, 1.55, edgecolor='#94a3b8', facecolor='#ffffff', linewidth=0.8)
    ax.add_patch(cavity)

    # 6 Male Pin Posts: Pin 6 (left, near 10V flat) to Pin 1 (right, near 8V flat)
    for i in range(6):
        px = 11.55 + i * 0.35
        p_num = 6 - i
        is_gnd_pin = (p_num == 4)
        is_sq_pin = (p_num == 6)

        if is_sq_pin:
            pad = patches.Rectangle((px - 0.11, 5.39), 0.22, 0.22, edgecolor='#b91c1c', facecolor='#fee2e2', lw=0.9)
            ax.add_patch(pad)
        else:
            pad_fill = '#dcfce7' if is_gnd_pin else '#f1f5f9'
            pad_edge = '#15803d' if is_gnd_pin else '#64748b'
            pad = patches.Circle((px, 5.50), 0.11, edgecolor=pad_edge, facecolor=pad_fill, lw=0.9)
            ax.add_patch(pad)

        p_color = '#15803d' if is_gnd_pin else ('#b91c1c' if is_sq_pin else '#0f172a')
        ax.plot(px, 5.50, marker='s', markersize=3.6, color=p_color)
        ax.text(px, 5.95, str(p_num), fontsize=6.8, fontweight='bold', ha='center', va='center', fontfamily='monospace')

    male_indicator = 'MALE PCB HEADER (PIN POSTS PROTRUDE UP)' if is_en else 'CONECTOR MACHO NA PCB (PINOS P/ CIMA)'
    ax.text(12.425, 4.38, male_indicator, fontsize=5.6, fontweight='bold', color='#475569', ha='center', fontfamily='monospace')

    # Flat 8 Vias Socket (FP50004) on the right - vertical vias side-by-side
    f8_sock = patches.Rectangle((13.80, 4.60), 1.35, 2.25, edgecolor='#000000', facecolor='#e2e8f0', linewidth=1.2)
    ax.add_patch(f8_sock)
    f8_lbl = 'FP50004\n(Flat 8 Vias)' if is_en else 'FP50004\n(Flat 8 Vias)'
    ax.text(14.475, 7.00, f8_lbl, fontsize=6.8, fontweight='bold', ha='center', fontfamily='monospace')
    _render_vertical_conductive_vias(ax, [13.95 + i * 0.150 for i in range(8)])

    ax.annotate('', xy=(14.80, 4.15), xytext=(10.05, 4.15), arrowprops=dict(arrowstyle='->', color='#000000', lw=1.5))
    seq_lbl = ('CANONICAL ORDER: PIN 6 (10V FLAT) <---> PIN 1 (8V FLAT)' if is_en
               else 'ORDEM CANÔNICA: PINO 6 (FLAT 10V) <---> PINO 1 (FLAT 8V)')
    ax.text(12.425, 3.90, seq_lbl, fontsize=6.2, fontweight='bold', ha='center', fontfamily='monospace')

    # SECTION 3: Bottom Engineering Notes
    nt_box = patches.Rectangle((0.85, 0.58), 8.50, 1.17, edgecolor='#000000', facecolor='#f8fafc', linewidth=1.2)
    ax.add_patch(nt_box)
    cad_hdr = ('MECHANICAL CONFORMANCE RULES AND BENCH VISUAL IDENTIFICATION:' if is_en
               else 'REGRAS DE CONFORMIDADE MECÂNICA E IDENTIFICAÇÃO VISUAL DE BANCADA:')
    ax.text(0.98, 1.62, cad_hdr, fontsize=7.0, fontweight='bold', fontfamily='monospace')

    if is_en:
        cad_notes = (
            '1. THICK vs THIN WALL: Rear wall (0.60mm) faces board interior; thin front wall (0.45mm) houses the locking latch notch.\n'
            '2. HARNESS MATING: Female ZHR-6 connector plugs directly into P7501 header with Wire 1 matching Pin 1, Wire 4 matching GND.\n'
            '3. GROUND PIN 4 (0.0Ω): Pin 4 is GND, verified with direct zero-ohm continuity to metal chassis. Do NOT reverse with Pin 1.\n'
            '4. SHORT-CIRCUIT PREVENTION: Pin 1 is +3.3V VDD Sense. Never ground this pin or inject 5V into the Matsushita SoC!'
        )
    else:
        cad_notes = (
            '1. PAREDE GROSSA vs FINA: A parede traseira (0.60mm) fica para dentro da placa; a parede frontal fina (0.45mm) abriga a trava.\n'
            '2. CONEXÃO DO CHICOTE: O soquete fêmea ZHR-6 conecta diretamente ao conector P7501 com Fio 1 no Pino 1 e Fio 4 no GND.\n'
            '3. TERRA CANÔNICO PINO 4 (0.0Ω): O pino 4 é o GND canônico, verificado com continuidade 0.0Ω ao chassi metálico.\n'
            '4. PREVENÇÃO DE CURTO: O pino 1 é +3.3V VDD Sense. Jamais aterre este pino contra o chassi nem injete tensões >3.3V!'
        )
    ax.text(0.98, 1.48, cad_notes, fontsize=6.2, color='#334155', ha='left', va='top', fontfamily='sans-serif', linespacing=1.22)

    finalize_and_save_drawing(fig, ax, output_path)


def generate_all_assets() -> None:
    """Generates both Canonical English and Portuguese Fallback engineering assets."""
    target_dirs = [
        'docs/images',
        'docs/images/pt-br',
        'docs/pt-br/images',
        'docs/pt-BR/images',
        'docs/en-US/images',
    ]
    for d in target_dirs:
        os.makedirs(d, exist_ok=True)

    # 1. Canonical English Assets (default in docs/images/ and docs/en-US/images/)
    schem_en = 'docs/images/dwire32a_pico_probe_assembly_schematic.png'
    cad_en = 'docs/images/panasonic_dmr_eh55_pcb_p7501_orientation.png'
    generate_electrical_schematic(schem_en, lang='en')
    generate_mechanical_cad_drawing(cad_en, lang='en')

    # Mirror to docs/en-US/images/
    shutil.copy2(schem_en, 'docs/en-US/images/dwire32a_pico_probe_assembly_schematic.png')
    shutil.copy2(cad_en, 'docs/en-US/images/panasonic_dmr_eh55_pcb_p7501_orientation.png')

    # Copy official schematics & PCB crops if available
    for extra in ['schematic_p7501_panasonic_official.png', 'pcb_p7501_crop1.png', 'pcb_p7501_crop2.png']:
        src_path = os.path.join('docs/images', extra)
        if os.path.isfile(src_path):
            shutil.copy2(src_path, os.path.join('docs/en-US/images', extra))
            shutil.copy2(src_path, os.path.join('docs/pt-BR/images', extra))
            shutil.copy2(src_path, os.path.join('docs/pt-br/images', extra))

    # 2. Portuguese Fallback Assets (in docs/images/pt-br/, docs/pt-BR/images/, and docs/pt-br/images/)
    schem_pt = 'docs/images/pt-br/dwire32a_pico_probe_assembly_schematic.png'
    cad_pt = 'docs/images/pt-br/panasonic_dmr_eh55_pcb_p7501_orientation.png'
    generate_electrical_schematic(schem_pt, lang='pt')
    generate_mechanical_cad_drawing(cad_pt, lang='pt')

    for pt_dir in ['docs/pt-br/images', 'docs/pt-BR/images']:
        shutil.copy2(schem_pt, os.path.join(pt_dir, 'dwire32a_pico_probe_assembly_schematic.png'))
        shutil.copy2(cad_pt, os.path.join(pt_dir, 'panasonic_dmr_eh55_pcb_p7501_orientation.png'))

    print('[OK] Generated and mirrored CAD & schematic assets to docs/en-US/ and docs/pt-BR/')


if __name__ == '__main__':
    generate_all_assets()
