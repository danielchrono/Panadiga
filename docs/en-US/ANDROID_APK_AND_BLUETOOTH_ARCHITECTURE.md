# Mobile Architecture (Android APK) and Wireless Connectivity (Bluetooth Low Energy)
## Panadiga Workbench Mobile — Native Rust Hardware Abstraction Layer Specification

> **Classification:** Systems Architecture and Hardware Engineering Specification  
> **Target Devices:** Android Mobile Devices (`aarch64-linux-android`) & Raspberry Pi Pico / Pico W / Pico 2 Probes  
> **Under-Test Target:** Panasonic DIGA DMR-EH55 (Digital PCB `VEP79138`, Header `P7501`)  
> **Status:** Normative / Validated in Multi-Platform CI/CD  

---

## 1. Solution Overview

To facilitate field bench operations, rapid forensic triage, and galvanic isolation without requiring a tethered desktop computer or long USB cables running to the recorder chassis, the **Panadiga** ecosystem provides a native mobile application for **Android (`.apk`)**, communicating over **Bluetooth Low Energy (BLE)** and **USB-C OTG**.

```mermaid
flowchart TD
    subgraph Mobile["Android Mobile Station (Smartphone / Tablet)"]
        direction TB
        UI["Reactive Panadiga Interface<br/>(React 18 + Tailwind CSS + TypeScript)<br/>Strict Presentation Layer (View / ViewModel)"]
        IPC["Typed Tauri IPC Bridge (invoke / listen)<br/>Zero Base64 / Zero WebView Shims"]
        TAURI_RUST["Tauri 2 Mobile Core & Native HAL (Rust)<br/>aarch64-linux-android<br/>Direct Port & Socket Management"]
        UI <--> IPC
        IPC <--> TAURI_RUST
    end

    subgraph NativeHAL["Native Rust Infrastructure Layer (Zero Kotlin)"]
        direction LR
        BLE_CORE["Native BLE Driver in Rust<br/>Nordic UART Service (NUS)<br/>NDK / BluetoothAdapter Bridge"]
        USB_CORE["Native USB Serial CDC Driver in Rust<br/>tauri-plugin-serialport / NDK USB Host<br/>Baud: 115200 bps | Latency: < 1 ms"]
        TAURI_RUST --- BLE_CORE
        TAURI_RUST --- USB_CORE
    end

    subgraph Probe["Microcontroller Bench Probe"]
        direction TB
        PICO["Raspberry Pi Pico W / Pico 2W<br/>RP2040 / RP2350 + CYW43439<br/>C11 Firmware + PIO State Machine"]
        BUFFER["Lossless Circular Telemetry Ring Buffer<br/>GDB RSP Server & P7501 Async UART"]
        PIO_ENG["Local PIO / UART Engine @ 115200 8N1<br/>Deterministic XRESET / SW_UART Timing<br/>NSW5V (GP3) Hi-Z Protection"]
        PICO --- BUFFER
        BUFFER --- PIO_ENG
    end

    subgraph Target["Target Hardware (Panasonic DMR-EH55)"]
        direction TB
        P7501["P7501 Header (B6B-ZR-3R4 / 6 Pins)<br/>Silkscreen 'TO JIGU'<br/>1: SW_UART | 2: XRESET | 3: UARTM2P_T<br/>4: GND | 5: NSW5V (Hi-Z) | 6: UARTP2M_T"]
        AM34["Matsushita AM34 / MN103S<br/>Main Board VEP79138<br/>Boot / CDDIAG Service Console"]
        P7501 <--> AM34
    end

    NativeHAL <-->|"BLE (NUS) or USB-C OTG"| Probe
    Probe <-->|"P7501 Async UART 115200 8N1 (3.3V LVTTL)"| Target
```

---

## 2. Architectural Decision: Native Rust HAL via Tauri 2.0 (Zero Kotlin / Zero WebUSB)

The mobile client architecture is governed by the principles of **Separation of Concerns (SoC), Clean Architecture, and Real-Time Determinism**:

### 2.1 Technical Refutation of WebUSB / Web Bluetooth in WebView (Path 1)
Using WebUSB (`navigator.usb`) and Web Bluetooth (`navigator.bluetooth`) from the WebView JavaScript engine was explicitly **rejected** for core engineering reasons:
1. **Android WebView Security Sandbox (`android.webkit.WebView`)**: The default Android WebView disables WebUSB by Google security policy. Enabling it would require authoring custom Java/Kotlin subclasses of `WebChromeClient` with manual wrappers around `android.hardware.usb.UsbManager`, violating the requirement of zero handcrafted Kotlin.
2. **Dependency Inversion Violation**: Hardware communication belongs strictly in the **Infrastructure / Hardware Abstraction Layer (HAL)**. Implementing device drivers in TypeScript couples hardware transport lifecycles to React UI component mounting.
3. **Garbage Collection (GC) Jitter & Serialization Overhead**: AM34 in-circuit debugging via GDB RSP requires sub-millisecond lexical packet turnaround. In the V8 engine, unpredictable GC pauses and Base64/JSON payload conversion introduce unacceptable latency into single-step (`$s#...`) and register dump (`$g#...`) operations.

### 2.2 The Approved Strategy: Native Rust HAL via Tauri 2.0 Core (Path 2)
1. **Native Rust Transport Layer**:
   - All USB CDC serial port control (`/dev/bus/usb`, ch34x, cdc-acm, FTDI) and Bluetooth Low Energy handling reside **100% inside the Rust Core Engine**, using official Tauri 2 plugins (`tauri-plugin-serialport` and native Rust BLE crates for Linux/Android).
   - Tauri 2 bundles the underlying JNI/NDK bindings internally, completely removing any need for manual Java or Kotlin code.
2. **Frontend Strictly as Presentation (View/ViewModel)**:
   - The React 18 / TypeScript interface has no direct knowledge of hardware buses. It strictly invokes strongly typed IPC commands (`invoke('pico_connect_usb')`, `invoke('pico_connect_ble')`) and listens to the unified logging event bus.
3. **Zero-Copy Processing over Immutable Slices (`&[u8]`)**:
   - Incoming RSP frames and probe telemetry arrive directly in Rust, are validated over immutable byte slices without serialization overhead, and are recorded to the `GdbTelemetryRingBuffer`.

---

## 3. Wireless Connectivity via Bluetooth Low Energy (BLE)

### 3.1 GATT Profile: Nordic UART Service (NUS)
The Raspberry Pi Pico W / Pico 2W firmware exposes the standard **Nordic UART Service (NUS)** on the Infineon/CYW43439 radio:

| GATT Attribute | Canonical UUID | Properties | Purpose |
| :--- | :--- | :--- | :--- |
| **Service (NUS)** | `6E400001-B5A3-F393-E0A9-E50E24DCCA9E` | Primary Service | Transparent virtual serial stream over BLE |
| **RX Characteristic** | `6E400002-B5A3-F393-E0A9-E50E24DCCA9E` | Write Without Response / Write | Commands sent from Rust Core to Pico (e.g., Cold Break-In, GDB RSP) |
| **TX Characteristic** | `6E400003-B5A3-F393-E0A9-E50E24DCCA9E` | Notify / Read | Telemetry streams and register responses from Pico to Rust Core |

### 3.2 Nanosecond Bus Timing vs. Bluetooth Latency
A common engineering question is whether Bluetooth latency compromises nanosecond-accurate silicon bus timings:

> [!IMPORTANT]
> **Bluetooth NEVER bit-bangs the hardware lines directly.**  
> `P7501` communicates at **`115200 8N1` UART** paired with sub-millisecond Open-Drain reset (`XRESET` / `GP5`) and `SW_UART` boot latching (`GP26`).  
> All real-time UART framing and reset pulses are executed **locally on the RP2040/RP2350** (`p7501_uart.pio` + HAL).  
> The Bluetooth channel operates solely on the upper transport layer (asynchronously streaming command and telemetry frames managed by the native Rust HAL), where latencies of $15\text{ ms}$ to $25\text{ ms}$ are completely transparent to the user and the asynchronous console/GDB RSP session.

---

## 4. Electrical Discipline: Pinout & P7501 Power Constraints

### 4.1 Strict Prohibition on Powering Pico via Panasonic P7501

> [!CAUTION]
> **CRITICAL PHYSICAL SAFETY RULE: NEVER POWER THE RASPBERRY PI PICO FROM HEADER P7501, AND KEEP `NSW5V` (`GP3`) Hi-Z ONLY.**  
> Pin 1 (`SW_UART`) only senses target logic (`~3.25V`, $I < 10\text{ mA}$), while Pin 5 (`NSW5V` / `GP3`) ties via `D7510` to `IC1506` (`~5.15V` upstream / `~3.29V` resting) to bias `R7570`/`R7572` (`10 kΩ`) and `IC7501` VDD.  
> A standard Pico draws $30\text{ mA}$ to $50\text{ mA}$, while a **Pico W / Pico 2W peaks between $150\text{ mA}$ and $300\text{ mA}$ during Wi-Fi/Bluetooth RF transmissions**.  
> Sourcing power from `P7501` or driving `GP3` (`NSW5V`) as push-pull will destabilize the board supply or overload the `Q7510`/`IC7501` supervisor network.

### 4.2 Grounding and External Power Rules
1. **Pico Power Supply**:
   - Power the Pico from an external $5.0\text{V}$ supply on pin `VSYS` (Pin 39) or via its micro-USB/USB-C connector (portable 5V powerbank or regulated 3.7V LiPo booster).
2. **Ground Reference (GND)**:
   - Header Pin 4 of `P7501` (Panasonic GND) **MUST** be bonded to a Pico GND pin (e.g., Pin 38) to equalize the UART ground reference ($0.0\,\Omega$ to chassis).
3. **Voltage Sensing (Sense Only)**:
   - Header Pin 1 (`SW_UART` -> `GP26` / `ADC0`) reports logic state (`~3.25V` Hi-Z, `~2.69V` Pull-Down), while Pin 5 (`NSW5V` -> `GP3`) remains permanently configured as **`Hi-Z Input` without internal pulls**.

### 4.3 Canonical P7501 Pinout Table (B6B-ZR-3R4)

| Pin (`P7501`) | Schematic Net | Matsushita Signal | Pico Pin (RP2040/RP2350) | Conditioning / PCB Network | Function |
| :---: | :---: | :---: | :---: | :---: | :--- |
| **1** | `SW_UART` | `SW_UART (D)` | `GP26 / ADC0` | $1.0\,\text{k}\Omega$ ADC series / `Q7510` | Serial boot latch (`LOW` drops `XFWE` & cuts off `Q7514`; `Hi-Z` enables TX). **DO NOT POWER PICO.** |
| **2** | `XRESET` | `XRESET` | `GP5` | Strict Open-Drain (`LOW`/Hi-Z) | Master system reset (`~20 µs` Hi-Z release rise time). Push-pull HIGH is strictly forbidden! |
| **3** | `MAX RXD` | `UARTM2P_T` | `GP4` (Pico TX) | Board `R7570 10 kΩ` to `NSW5V` + `Q7514` | Asynchronous `115200 8N1` TX to AM34 RX (requires `SW_UART` Hi-Z). No external 3.3V pull-up. |
| **4** | `GND` | `GND` | `GND` (Pin 38) | Direct low-impedance bond | **Mandatory common ground reference ($0.0\,\Omega$ to chassis)** |
| **5** | `NSW5V` | `X_SW_5R2V` | `GP3` | High-impedance sense via `IC1506`/`D7510` | **Static sense (`~3.29V` resting / `~5.15V` up). MUST remain `Hi-Z Input` without pulls!** |
| **6** | `MAX TXD` | `UARTP2M_T` | `GP2` (Pico RX) | Board `R7572 10 kΩ`, Square Pad | Asynchronous `115200 8N1` RX from AM34 (`<00>DDR CHECK: OK`, `CD diag boot mode`). |

---

## 5. Automated APK CI/CD Build Pipeline

The Android APK compilation is automated via GitHub Actions (`.github/workflows/ci.yml`) targeting `aarch64-linux-android`:

```mermaid
sequenceDiagram
    autonumber
    participant GA as GitHub Actions Runner (Ubuntu 24.04)
    participant SDK as Android SDK 34 & NDK r27c
    participant RUST as Rust Target (aarch64-linux-android)
    participant TAURI as Tauri 2 Mobile CLI
    participant ART as Artifact Storage (GitHub Releases)

    GA->>SDK: Configures ANDROID_HOME and NDK_HOME
    GA->>RUST: Adds toolchain aarch64-linux-android
    GA->>TAURI: Runs npx tauri android build --apk
    TAURI->>RUST: Compiles native libpanadiga.so in Rust (with serial & BLE HAL)
    TAURI->>SDK: Bundles Web assets and packages debug APK
    TAURI-->>ART: Uploads Panadiga-Workbench-Android-arm64.apk
```

### 5.1 Local Compilation Instructions
To build the APK locally on a development workstation:

```powershell
# 1. Export Android SDK and NDK paths
$env:ANDROID_HOME = "C:\Users\<User>\AppData\Local\Android\Sdk"
$env:NDK_HOME = "$env:ANDROID_HOME\ndk\27.0.12077973"

# 2. Add Rust ARM64 target
rustup target add aarch64-linux-android

# 3. Build APK cleanly from the repository root
npm run build:android
```

The resulting output APK is saved under:
`gen/android/app/build/outputs/apk/` (or `dist/releases/` when packaged via `scripts/build.py --android`)

---

## 6. Field Operation Runbook

1. **Powering the Probe**: Power the Raspberry Pi Pico W from an external 5V powerbank or bench supply. The on-board LED indicates BLE advertising.
2. **Safe Mechanical Mating**: With the Panasonic unit unplugged from AC mains, mate the JST ZR 1.5mm connector to header `P7501` on PCB `VEP79138`.
3. **App Initialization**: Launch **Panadiga Mobile** on the Android device.
4. **Pairing via Native Rust HAL**:
   - Tap **"Connect BLE (Pico W)"**.
   - The native Rust core scans and connects directly to the Nordic UART Service (`6E400001-...`).
5. **Telemetry and Unbrick**:
   - Power on the DMR-EH55. The **VDD Sense** indicator turns green ($3.3\text{V}$).
   - Execute **Cold Break-In** to halt AM34 execution before IPL boot.
   - Run NVRAM surgical sanitization or attach remote GDB sessions with zero GC jitter.
