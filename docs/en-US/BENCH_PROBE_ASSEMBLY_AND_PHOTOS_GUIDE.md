# ILLUSTRATED BENCH PROBE ASSEMBLY & PRACTICAL LAB GUIDE
## In-Circuit `P7501` Async UART (`115200 8N1`) Bench Probe Assembly for Panasonic DMR-EH55 (VEP79138 Board)

**Document:** `BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md`  
**Classification:** Laboratory Assembly & Practical Engineering Runbook  

---

## 1. Engineering Scope & Purpose

This document provides mechanical assembly instructions, electrical wiring steps, and laboratory procedures for the **`P7501` Asynchronous UART (`115200 8N1`) + Discrete Boot/Reset Control Probe** based on the **Raspberry Pi Pico (RP2040 / RP2350)** microcontroller.

The tool enables:
1. Intercepting processor boot sequences (such as `CD diag boot mode` / `XFWE` serial boot) on the **Matsushita AM34 / MN103S (`VEP79138`)** platform without desoldering `IC5001`.
2. Bridging UART/service console telemetry (`UARTP2M_T` / `UARTM2P_T`) and GDB Remote Serial Protocol (**RSP** over TCP port `3333`) for firmware analysis in Ghidra/IDA.
3. Monitoring live `SW_UART` (`GP26`), `XRESET` open-drain rise timing (`GP5`), and `NSW5V` rail presence (`GP3` kept strictly `Hi-Z Input` without internal pulls) while preserving `IC5001` (`S29JL064H` 8 MB NOR Flash).

> [!NOTE]
> ### Rigorous Engineering CAD vs. Fanciful AI Artwork
> Stylized "PowerPoint" drawings and dark-mode artistic illustrations have been strictly excluded from this repository. In physical hardware laboratories, visual ambiguity risks permanent silicon damage. All schematics and CAD assets adhere to:
> 1. **Official JST ZR Datasheet Specifications (`B6B-ZR-3R4`)**: Accurate shroud geometry, thin front latch wall vs. thick reinforced rear wall, 45° polarizing chamfer, and square PCB pad for Pin 6.
> 2. **IEEE Std 315 / IEC 60617 Compliance**: Clean black-and-white engineering title blocks, precise electrical symbology, signal vs. chassis ground distinction, and proper component references.

---

## 2. CRITICAL LAB SAFETY DIRECTIVE: Ground (GND) Verification

> [!CAUTION]
> ### MANDATORY CONTINUITY VERIFICATION — CHASSIS GROUND (GND)
> * **Official Panasonic Service Manual Reference (Schematic P7501 [B6B-ZR-3R4], p. 62):** **`GND` IS STRICTLY PIN 4**.
> * **SHORT CIRCUIT & `NSW5V` HAZARD:** NEVER assume that Pin 1 or Pin 6 is ground! Pin 1 (`SW_UART`) connects to the `Q7510`/`QR7503`/`IC7501` boot network (`~3.25V`), and Pin 5 (`NSW5V`) connects via `D7510` to `~5.15V` (`IC1506`, `~3.29V` resting) and MUST remain `Hi-Z Input` on `GP3`. Shorting Pin 1 or Pin 5 to ground will blow the board fuse or damage the recorder power/reset circuit.
> * **INVIOLABLE BENCH DIRECTIVE:** Prior to applying power or connecting the Raspberry Pi Pico, **set your digital multimeter to continuity mode and verify an audible $0.0\,\Omega$ short between the chosen ground wire and the unpainted metal chassis frame of the recorder.**

---

## 3. High-Resolution Visual Documentation

### 3.1 Official Panasonic Schematic (VEP79138 Test Port P7501)
![Official Panasonic P7501 Schematic](../assets/images/schematic_p7501_panasonic_official.png)

*Visible details in factory schematic:*
* Designated as **`P7501`**, part number **`B6B-ZR-3R4`**, labeled **`TO JIGU`**.
* **Pin 1:** `SW_UART (D)` (`~3.25V` Hi-Z / `~2.69V` PD via `Q7510`; LOW activates `XFWE` serial boot & cuts off `Q7514`)
* **Pin 2:** `XRESET` (`GP5`, Strict Open-Drain Active LOW)
* **Pin 3:** `UARTM2P_T` (`MAX RXD`, `115200 8N1` Pico TX -> AM34 RX via `Q7514` & `R7570 10 kΩ`)
* **Pin 4:** **`GND (0.0Ω Solid Chassis Ground)`**
* **Pin 5:** `NSW5V` (`X_SW_5R2V`, `~5.15V` upstream via `D7510`, `GP3` MUST remain `Hi-Z Input`!)
* **Pin 6:** `UARTP2M_T` (`MAX TXD`, `115200 8N1` AM34 TX -> Pico RX, `R7572 10 kΩ`, Square Pad)

### 3.2 Physical PCB Topography & CAD Layout Detail
![Physical Topography and P7501 Orientation](images/panasonic_dmr_eh55_pcb_p7501_orientation.png)

### 3.3 Laboratory Assembly Schematic
![Pico Probe Assembly Electrical Schematic](images/p7501_pico_probe_assembly_schematic.png)

---

## 4. Step-by-Step Harness Assembly Runbook

1. **Obtain JST ZR 1.5mm 6-Pin Harness:**
   Acquire a pre-crimped `ZHR-6` female connector with 6 flying leads to 2.54mm Dupont pins.
2. **Color-Code the Wiring:**
   - Pin 1 (`SW_UART`): **Red**
   - Pin 2 (`XRESET`): **Yellow**
   - Pin 3 (`UARTM2P_T`): **Blue**
   - Pin 4 (`GND`): **Black**
   - Pin 5 (`NSW5V` Hi-Z Sense ONLY): **White**
   - Pin 6 (`UARTP2M_T`, Square Pad): **Orange**
3. **Verify Wire Length:** Keep harness length $\le 15\text{ cm}$ for unbuffered direct connections.
4. **Bench Continuity Test:** Verify zero ohms between Black wire (Pin 4) and recorder metal casing.
5. **Connect to Raspberry Pi Pico Header:**
   - Red (Pin 1, `SW_UART`) $\rightarrow$ Pico Pin 31 (`GP26 / ADC0`)
   - Yellow (Pin 2, `XRESET`) $\rightarrow$ Pico Pin 7 (`GP5`)
   - Blue (Pin 3, `UARTM2P_T`) $\rightarrow$ Pico Pin 6 (`GP4`, UART TX)
   - Black (Pin 4, `GND`) $\rightarrow$ Pico Pin 38 (`GND`)
   - White (Pin 5, `NSW5V`) $\rightarrow$ Pico Pin 5 (`GP3` — strictly `Hi-Z Input` without pulls)
   - Orange (Pin 6, `UARTP2M_T`) $\rightarrow$ Pico Pin 4 (`GP2`, UART RX)
