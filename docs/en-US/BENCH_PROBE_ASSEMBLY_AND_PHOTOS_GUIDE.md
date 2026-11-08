# ILLUSTRATED BENCH PROBE ASSEMBLY & PRACTICAL LAB GUIDE
## In-Circuit DWire32A Debug Probe Assembly for Panasonic DMR-EH55 (VEP79138 Board)

**Document:** `BENCH_PROBE_ASSEMBLY_AND_PHOTOS_GUIDE.md`  
**Classification:** Laboratory Assembly & Practical Engineering Runbook  

---

## 1. Engineering Scope & Purpose

This document provides mechanical assembly instructions, electrical wiring steps, and laboratory procedures for the **Matsushita DWire32A / On-Chip Debug (OCD)** diagnostic probe based on the **Raspberry Pi Pico (RP2040 / RP2350)** microcontroller.

The tool enables:
1. Intercepting processor boot sequence (*Cold Break-In*) on the **UniPhier MN2DS0016** (AM34 core).
2. Debugging factory firmware via the **GDB Remote Serial Protocol (RSP)** server over TCP port 3333.
3. Injecting the standalone `ram_loader` microcode payload into internal SRAM (`0x00004000`) to execute in-circuit erasure of NVRAM sectors `SA134..SA141` (`0x807F0000..0x807FFFFF`) on the Spansion `S29JL064H` NOR Flash, curing fatal error **`U99`** without desoldering the TSOP-48 package.

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
> * **SHORT CIRCUIT HAZARD:** NEVER assume that Pin 1 or Pin 6 is ground! Pin 1 is the `+3.3V VDD Sense` line directly tied to the UniPhier core supply rail. Shorting Pin 1 to chassis ground will blow the mainboard power rail fuse or destroy the Matsushita power management IC.
> * **INVIOLABLE BENCH DIRECTIVE:** Prior to applying power or connecting the Raspberry Pi Pico, **set your digital multimeter to continuity mode and verify an audible $0.0\,\Omega$ short between the chosen ground wire and the unpainted metal chassis frame of the recorder.**

---

## 3. High-Resolution Visual Documentation

### 3.1 Official Panasonic Schematic (VEP79138 Test Port P7501)
![Official Panasonic P7501 Schematic](images/schematic_p7501_panasonic_official.png)

*Visible details in factory schematic:*
* Designated as **`P7501`**, part number **`B6B-ZR-3R4`**, labeled **`TO WRITE JIGU`**.
* **Pin 1:** `VDD Sense (+3.3V Ref)`
* **Pin 2:** `nRESET (Master XRST, Active LOW 0V)`
* **Pin 3:** `SDATA (Bidirectional Half-Duplex DWire32A)`
* **Pin 4:** **`GND (0.0Ω Solid Chassis Ground)`**
* **Pin 5:** `SCLK (Synchronous Master Clock 1.0 MHz)`
* **Pin 6:** `EXTRG (External OCD Boot Trigger / Break, Square Pad)`

### 3.2 Physical PCB Topography & CAD Layout Detail
![Physical Topography and P7501 Orientation](images/panasonic_dmr_eh55_pcb_p7501_orientation.png)

### 3.3 Laboratory Assembly Schematic
![Pico Probe Assembly Electrical Schematic](images/dwire32a_pico_probe_assembly_schematic.png)

---

## 4. Step-by-Step Harness Assembly Runbook

1. **Obtain JST ZR 1.5mm 6-Pin Harness:**
   Acquire a pre-crimped `ZHR-6` female connector with 6 flying leads to 2.54mm Dupont pins.
2. **Color-Code the Wiring:**
   - Pin 1 (VDD Sense): **Red**
   - Pin 2 (nRESET): **Yellow**
   - Pin 3 (SDATA): **Blue**
   - Pin 4 (GND): **Black**
   - Pin 5 (SCLK): **White**
   - Pin 6 (EXTRG): **Orange**
3. **Verify Wire Length:** Keep harness length $\le 15\text{ cm}$ for unbuffered direct connections.
4. **Bench Continuity Test:** Verify zero ohms between Black wire (Pin 4) and recorder metal casing.
5. **Connect to Raspberry Pi Pico Header:**
   - Red (Pin 1) $\rightarrow$ Pico Pin 31 (GP26 / ADC0)
   - Yellow (Pin 2) $\rightarrow$ Pico Pin 7 (GP5)
   - Blue (Pin 3) $\rightarrow$ Pico Pin 6 (GP4)
   - Black (Pin 4) $\rightarrow$ Pico Pin 38 (GND)
   - White (Pin 5) $\rightarrow$ Pico Pin 5 (GP3)
   - Orange (Pin 6) $\rightarrow$ Pico Pin 4 (GP2)
