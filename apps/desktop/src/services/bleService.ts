/**
 * @file bleService.ts
 * @brief Bluetooth Low Energy (BLE) Nordic UART Service (NUS) Client.
 *
 * Enables wireless communication between Android / Web clients and
 * Raspberry Pi Pico W / Pico 2 W running the DWire32A BLE bridge firmware.
 * Uses standard Web Bluetooth API (navigator.bluetooth) with zero native Kotlin code.
 */

// Nordic UART Service (NUS) Canonical UUIDs
export const NUS_SERVICE_UUID = '6e400001-b5a3-f393-e0a9-e50e24dcca9e';
export const NUS_RX_CHARACTERISTIC_UUID = '6e400002-b5a3-f393-e0a9-e50e24dcca9e'; // Host -> Pico (Write)
export const NUS_TX_CHARACTERISTIC_UUID = '6e400003-b5a3-f393-e0a9-e50e24dcca9e'; // Pico -> Host (Notify)

// Web Bluetooth API Standard Interface Declarations
interface BluetoothRemoteGATTCharacteristic extends EventTarget {
  startNotifications(): Promise<BluetoothRemoteGATTCharacteristic>;
  writeValue(value: BufferSource): Promise<void>;
  addEventListener(type: string, listener: (event: Event) => void): void;
}

interface BluetoothRemoteGATTService {
  getCharacteristic(characteristic: string): Promise<BluetoothRemoteGATTCharacteristic>;
}

interface BluetoothRemoteGATTServer {
  connected: boolean;
  connect(): Promise<BluetoothRemoteGATTServer>;
  disconnect(): void;
  getPrimaryService(service: string): Promise<BluetoothRemoteGATTService>;
}

interface BluetoothDevice extends EventTarget {
  id: string;
  name?: string;
  gatt?: BluetoothRemoteGATTServer;
  addEventListener(type: string, listener: () => void): void;
}

interface BluetoothRequestOptions {
  filters?: Array<{ namePrefix?: string; services?: string[] }>;
  optionalServices?: string[];
}

interface BluetoothNavigator {
  requestDevice(options: BluetoothRequestOptions): Promise<BluetoothDevice>;
}

declare global {
  interface Navigator {
    bluetooth?: BluetoothNavigator;
  }
}

export interface BleDeviceInfo {
  id: string;
  name: string;
  connected: boolean;
}

export type BleDataCallback = (data: string) => void;

class BleUartService {
  private device: BluetoothDevice | null = null;
  private server: BluetoothRemoteGATTServer | null = null;
  private rxCharacteristic: BluetoothRemoteGATTCharacteristic | null = null;
  private txCharacteristic: BluetoothRemoteGATTCharacteristic | null = null;
  private receiveCallbacks: Set<BleDataCallback> = new Set();
  private mockConnected = false;

  /**
   * Verifies if Web Bluetooth API is supported in current browser/webview.
   */
  public isSupported(): boolean {
    return typeof navigator !== 'undefined' && 'bluetooth' in navigator;
  }

  /**
   * Returns current connection state.
   */
  public isConnected(): boolean {
    if (this.mockConnected) return true;
    return !!this.server?.connected;
  }

  /**
   * Requests a BLE device advertising Nordic UART Service and establishes GATT connection.
   */
  public async connect(): Promise<BleDeviceInfo> {
    if (!this.isSupported() || !navigator.bluetooth) {
      // Simulate connection for environments without Web Bluetooth (e.g. desktop web mock)
      this.mockConnected = true;
      return {
        id: 'BLE-MOCK-PICO-W',
        name: 'Raspberry Pi Pico W (NUS Mock)',
        connected: true,
      };
    }

    try {
      this.device = await navigator.bluetooth.requestDevice({
        filters: [
          { namePrefix: 'Pico' },
          { namePrefix: 'Panadiga' },
          { services: [NUS_SERVICE_UUID] },
        ],
        optionalServices: [NUS_SERVICE_UUID],
      });

      this.device.addEventListener('gattserverdisconnected', () => {
        this.cleanup();
      });

      if (!this.device.gatt) {
        throw new Error('GATT Server não disponível no dispositivo BLE.');
      }

      this.server = await this.device.gatt.connect();
      const service = await this.server.getPrimaryService(NUS_SERVICE_UUID);

      this.rxCharacteristic = await service.getCharacteristic(NUS_RX_CHARACTERISTIC_UUID);
      this.txCharacteristic = await service.getCharacteristic(NUS_TX_CHARACTERISTIC_UUID);

      await this.txCharacteristic.startNotifications();
      this.txCharacteristic.addEventListener('characteristicvaluechanged', (event: Event) => {
        const target = event.target as unknown as { value: DataView };
        if (target?.value) {
          const decoder = new TextDecoder('utf-8');
          const text = decoder.decode(target.value);
          this.receiveCallbacks.forEach(cb => cb(text));
        }
      });

      return {
        id: this.device.id,
        name: this.device.name || 'Pico W DWire32A',
        connected: true,
      };
    } catch (err: unknown) {
      this.cleanup();
      throw new Error(`Falha na conexão BLE: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  /**
   * Sends raw string/command over BLE RX Characteristic to Pico.
   */
  public async send(message: string): Promise<void> {
    if (this.mockConnected) {
      // Emulate Pico response in mock mode
      setTimeout(() => {
        const reply = message.trim() === 'PING' ? 'PONG\r\n' : `[BLE MOCK ACK] ${message.trim()}\r\n`;
        this.receiveCallbacks.forEach(cb => cb(reply));
      }, 50);
      return;
    }

    if (!this.rxCharacteristic) {
      throw new Error('Canal BLE não conectado. Impossível enviar dados.');
    }

    const encoder = new TextEncoder();
    const data = encoder.encode(message.endsWith('\n') ? message : message + '\n');
    await this.rxCharacteristic.writeValue(data);
  }

  /**
   * Registers a callback for incoming notifications from Pico.
   */
  public onReceive(callback: BleDataCallback): () => void {
    this.receiveCallbacks.add(callback);
    return () => {
      this.receiveCallbacks.delete(callback);
    };
  }

  /**
   * Disconnects and cleans up BLE handles.
   */
  public disconnect(): void {
    if (this.server?.connected) {
      this.server.disconnect();
    }
    this.cleanup();
  }

  private cleanup(): void {
    this.mockConnected = false;
    this.server = null;
    this.device = null;
    this.rxCharacteristic = null;
    this.txCharacteristic = null;
  }
}

export const bleService = new BleUartService();
