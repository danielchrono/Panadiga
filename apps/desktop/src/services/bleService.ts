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

export interface BleUartClient {
  readonly isSupported: () => boolean;
  readonly isConnected: () => boolean;
  readonly connect: () => Promise<BleDeviceInfo>;
  readonly send: (message: string) => Promise<void>;
  readonly sendAndAwaitReply: (command: string, timeoutMs?: number) => Promise<string>;
  readonly onReceive: (callback: BleDataCallback) => () => void;
  readonly disconnect: () => void;
}

interface CharacteristicValueEventTarget extends EventTarget {
  value?: DataView;
}

function extractCharacteristicDataView(event: Event): DataView | null {
  const target: EventTarget | null = event.target;
  if (!target || !('value' in target)) {
    return null;
  }
  const value = (target as CharacteristicValueEventTarget).value;
  return value instanceof DataView ? value : null;
}

/**
 * Pure closure factory encapsulating Web Bluetooth & Android BLE NUS transport state
 * without OOP classes or shared module globals.
 */
export function createBleUartService(): BleUartClient {
  let device: BluetoothDevice | null = null;
  let server: BluetoothRemoteGATTServer | null = null;
  let rxCharacteristic: BluetoothRemoteGATTCharacteristic | null = null;
  let txCharacteristic: BluetoothRemoteGATTCharacteristic | null = null;
  const receiveCallbacks = new Set<BleDataCallback>();
  let androidNativeConnected = false;
  let androidListenerBound = false;

  const cleanup = (): void => {
    androidNativeConnected = false;
    server = null;
    device = null;
    rxCharacteristic = null;
    txCharacteristic = null;
  };

  const ensureAndroidRxListener = (): void => {
    if (androidListenerBound || typeof window === 'undefined') return;
    androidListenerBound = true;
    window.addEventListener('panadiga-ble-rx', event => {
      const custom = event as CustomEvent<string>;
      if (typeof custom.detail === 'string') {
        receiveCallbacks.forEach(cb => cb(custom.detail));
      }
    });
  };

  const isSupported = (): boolean => {
    if (isAndroidNativeBridgeAvailable()) {
      return Boolean(getAndroidCapabilities()?.bleSupported);
    }
    return typeof navigator !== 'undefined' && 'bluetooth' in navigator;
  };

  const isConnected = (): boolean =>
    androidNativeConnected || Boolean(server?.connected);

  const onReceive = (callback: BleDataCallback): (() => void) => {
    receiveCallbacks.add(callback);
    return () => {
      receiveCallbacks.delete(callback);
    };
  };

  const connect = async (): Promise<BleDeviceInfo> => {
    if (isAndroidNativeBridgeAvailable()) {
      ensureAndroidRxListener();
      const res = connectAndroidBleNus(10000);
      if (!res.connected) {
        throw new Error(res.message || 'Falha ao conectar via BLE NUS no Android.');
      }
      androidNativeConnected = true;
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
      device = await navigator.bluetooth.requestDevice({
        filters: [
          { namePrefix: 'Pico' },
          { namePrefix: 'Panadiga' },
          { services: [NUS_SERVICE_UUID] },
        ],
        optionalServices: [NUS_SERVICE_UUID],
      });

      device.addEventListener('gattserverdisconnected', cleanup);

      if (!device.gatt) {
        throw new Error('GATT Server não disponível no dispositivo BLE.');
      }

      server = await device.gatt.connect();
      const service = await server.getPrimaryService(NUS_SERVICE_UUID);
      rxCharacteristic = await service.getCharacteristic(NUS_RX_CHARACTERISTIC_UUID);
      txCharacteristic = await service.getCharacteristic(NUS_TX_CHARACTERISTIC_UUID);

      await txCharacteristic.startNotifications();
      txCharacteristic.addEventListener('characteristicvaluechanged', (event: Event) => {
        const value = extractCharacteristicDataView(event);
        if (value) {
          const text = new TextDecoder('utf-8').decode(value);
          receiveCallbacks.forEach(cb => cb(text));
        }
      });

      return {
        id: device.id,
        name: device.name || 'Pico W P7501 UART / Panadiga',
        connected: true,
      };
    } catch (err: unknown) {
      cleanup();
      throw new Error(`Falha na conexão BLE: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const send = async (message: string): Promise<void> => {
    if (androidNativeConnected && isAndroidNativeBridgeAvailable()) {
      const res = sendAndroidBleCommand(message.trim(), 4000);
      if (!res.ok) {
        throw new Error(res.output || 'Falha no envio GATT NUS.');
      }
      if (res.output) {
        receiveCallbacks.forEach(cb => cb(res.output));
      }
      return;
    }

    if (typeof navigator === 'undefined' || !navigator.bluetooth) {
      throw new Error('Canal BLE não conectado. Use Android BLE nativo ou Web Bluetooth compatível.');
    }

    if (!rxCharacteristic) {
      throw new Error('Canal BLE não conectado. Impossível enviar dados.');
    }

    const data = new TextEncoder().encode(message.endsWith('\n') ? message : `${message}\n`);
    await rxCharacteristic.writeValue(data);
  };

  const sendAndAwaitReply = async (command: string, timeoutMs = 4000): Promise<string> => {
    if (androidNativeConnected && isAndroidNativeBridgeAvailable()) {
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
        cleanupRx = onReceive(chunk => {
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
        send(command).catch(err => {
          clearTimeout(timer);
          reject(err);
        });
      });
      return buffer.trim() || `${command}: OK`;
    } finally {
      cleanupRx();
    }
  };

  const disconnect = (): void => {
    if (androidNativeConnected) {
      disconnectAndroidBleNus();
    }
    if (server?.connected) {
      server.disconnect();
    }
    cleanup();
  };

  return Object.freeze({
    isSupported,
    isConnected,
    connect,
    send,
    sendAndAwaitReply,
    onReceive,
    disconnect,
  });
}

export const bleService = createBleUartService();
