/**
 *  * @file bleService.ts
 * @brief Bluetooth Low Energy (BLE) Nordic UART Service (NUS) Client.
 *
 * Enables wireless communication between Android Native Bridge / Web Bluetooth clients
 * and Raspberry Pi Pico W / Pico 2 W running the P7501 Async UART BLE bridge firmware.
 */

import {
  connectAndroidBleNus,
  disconnectAndroidBleNus,
  getAndroidCapabilities,
  isAndroidNativeBridgeAvailable,
  sendAndroidBleCommand,
} from './androidBridge';

export const NUS_SERVICE_UUID = '6e400001-b5a3-f393-e0a9-e50e24dcca9e';
export const NUS_RX_CHARACTERISTIC_UUID = '6e400002-b5a3-f393-e0a9-e50e24dcca9e';
export const NUS_TX_CHARACTERISTIC_UUID = '6e400003-b5a3-f393-e0a9-e50e24dcca9e';

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
  private connected = false;
  private androidNativeConnected = false;
  private androidListenerBound = false;

  private ensureAndroidRxListener(): void {
    if (this.androidListenerBound || typeof window === 'undefined') return;
    this.androidListenerBound = true;
    window.addEventListener('panadiga-ble-rx', event => {
      const custom = event as CustomEvent<string>;
      if (typeof custom.detail === 'string') {
        this.receiveCallbacks.forEach(cb => cb(custom.detail));
      }
    });
  }

  public isSupported(): boolean {
    if (isAndroidNativeBridgeAvailable()) {
      return Boolean(getAndroidCapabilities()?.bleSupported);
    }
    return typeof navigator !== 'undefined' && 'bluetooth' in navigator;
  }

  public isConnected(): boolean {
    return this.connected || this.androidNativeConnected || Boolean(this.server?.connected);
  }

  public async connect(): Promise<BleDeviceInfo> {
    if (isAndroidNativeBridgeAvailable()) {
      this.ensureAndroidRxListener();
      const res = connectAndroidBleNus(10000);
      if (!res.connected) {
        throw new Error(res.message || 'Falha ao conectar via BLE NUS no Android.');
      }
      this.androidNativeConnected = true;
      return {
        id: res.deviceName || 'ANDROID-BLE-PICO-W',
        name: res.deviceName || 'Pico W BLE (Android GATT)',
        connected: true,
      };
    }

    if (typeof navigator === 'undefined' || !navigator.bluetooth) {
      throw new Error('Web Bluetooth API indisponível neste navegador ou ambiente Desktop.');
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
          const text = new TextDecoder('utf-8').decode(target.value);
          this.receiveCallbacks.forEach(cb => cb(text));
        }
      });

      return {
        id: this.device.id,
        name: this.device.name || 'Pico W P7501 UART / Panadiga',
        connected: true,
      };
    } catch (err: unknown) {
      this.cleanup();
      throw new Error(`Falha na conexão BLE: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  public async send(message: string): Promise<void> {
    if (this.androidNativeConnected && isAndroidNativeBridgeAvailable()) {
      const res = sendAndroidBleCommand(message.trim(), 4000);
      if (!res.ok) {
        throw new Error(res.output || 'Falha no envio GATT NUS.');
      }
      if (res.output) {
        this.receiveCallbacks.forEach(cb => cb(res.output));
      }
      return;
    }

    if (typeof navigator === 'undefined' || !navigator.bluetooth) {
      throw new Error('Canal BLE não conectado. Use Android BLE nativo ou Web Bluetooth compatível.');
    }

    if (!this.rxCharacteristic) {
      throw new Error('Canal BLE não conectado. Impossível enviar dados.');
    }

    const data = new TextEncoder().encode(message.endsWith('\n') ? message : `${message}\n`);
    await this.rxCharacteristic.writeValue(data);
  }

  public async sendAndAwaitReply(command: string, timeoutMs = 4000): Promise<string> {
    if (this.androidNativeConnected && isAndroidNativeBridgeAvailable()) {
      const res = sendAndroidBleCommand(command.trim(), timeoutMs);
      if (!res.ok) {
        throw new Error(res.output || 'Falha na resposta BLE NUS.');
      }
      return res.output;
    }

    let buffer = '';
    let cleanupRx: () => void = () => {};
    try {
      await new Promise<void>((resolve, reject) => {
        const timer = setTimeout(() => resolve(), Math.min(timeoutMs, 1500));
        cleanupRx = this.onReceive(chunk => {
          buffer += chunk;
          if (
            buffer.includes('PANADIGA>') ||
            buffer.includes('P7501>') ||
            buffer.includes(': OK') ||
            buffer.includes('FAILED')
          ) {
            clearTimeout(timer);
            resolve();
          }
        });
        this.send(command).catch(err => {
          clearTimeout(timer);
          reject(err);
        });
      });
      return buffer.trim() || `${command}: OK`;
    } finally {
      cleanupRx();
    }
  }

  public onReceive(callback: BleDataCallback): () => void {
    this.receiveCallbacks.add(callback);
    return () => {
      this.receiveCallbacks.delete(callback);
    };
  }

  public disconnect(): void {
    if (this.androidNativeConnected) {
      disconnectAndroidBleNus();
    }
    if (this.server?.connected) {
      this.server.disconnect();
    }
    this.cleanup();
  }

  private cleanup(): void {
    this.connected = false;
    this.androidNativeConnected = false;
    this.server = null;
    this.device = null;
    this.rxCharacteristic = null;
    this.txCharacteristic = null;
  }
}

export const bleService = new BleUartService();
