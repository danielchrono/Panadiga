/**
 * Android Native Hardware Bridge Client (USB-C OTG CDC-ACM + Bluetooth LE NUS)
 * Wraps `window.__PANADIGA_ANDROID_BRIDGE__` injected by MainActivity.kt on Tauri Android APKs.
 */

export interface AndroidBridgeCapabilities {
  platform: 'android';
  usbHostSupported: boolean;
  bleSupported: boolean;
  bluetoothEnabled: boolean;
}

export interface AndroidUsbDeviceInfo {
  deviceName: string;
  vendorId: number;
  productId: number;
  productName: string;
  isRaspberryPi: boolean;
  isBootsel: boolean;
}

export interface AndroidBridgeConnectResult {
  connected: boolean;
  deviceName?: string;
  message: string;
}

export interface AndroidBridgeCommandResult {
  ok: boolean;
  output: string;
}

interface RawAndroidBridge {
  isAvailable(): boolean;
  getCapabilitiesJson(): string;
  requestPermissions(): boolean;
  listUsbOtgDevicesJson(): string;
  connectUsbOtg(targetDeviceName: string): string;
  disconnectUsbOtg(): void;
  sendUsbOtgCommand(command: string, timeoutMs: number): string;
  connectBleNus(timeoutMs: number): string;
  sendBleCommand(command: string, timeoutMs: number): string;
  disconnectBleNus(): void;
}

function getRawBridge(): RawAndroidBridge | null {
  if (typeof window === 'undefined') return null;
  const candidate = (window as unknown as Record<string, unknown>).__PANADIGA_ANDROID_BRIDGE__ as
    | RawAndroidBridge
    | undefined;
  return candidate && typeof candidate.isAvailable === 'function' ? candidate : null;
}

export function isAndroidNativeBridgeAvailable(): boolean {
  return getRawBridge()?.isAvailable() ?? false;
}

export function getAndroidCapabilities(): AndroidBridgeCapabilities | null {
  const bridge = getRawBridge();
  if (!bridge) return null;
  try {
    return JSON.parse(bridge.getCapabilitiesJson()) as AndroidBridgeCapabilities;
  } catch {
    return null;
  }
}

export function listAndroidUsbOtgDevices(): AndroidUsbDeviceInfo[] {
  const bridge = getRawBridge();
  if (!bridge) return [];
  try {
    return JSON.parse(bridge.listUsbOtgDevicesJson()) as AndroidUsbDeviceInfo[];
  } catch {
    return [];
  }
}

export function connectAndroidUsbOtg(deviceName = ''): AndroidBridgeConnectResult {
  const bridge = getRawBridge();
  if (!bridge) {
    return { connected: false, message: 'Bridge USB OTG disponível apenas no APK Android.' };
  }
  return JSON.parse(bridge.connectUsbOtg(deviceName)) as AndroidBridgeConnectResult;
}

export function sendAndroidUsbOtgCommand(
  command: string,
  timeoutMs = 3000
): AndroidBridgeCommandResult {
  const bridge = getRawBridge();
  if (!bridge) {
    return { ok: false, output: 'Bridge USB OTG indisponível.' };
  }
  return JSON.parse(bridge.sendUsbOtgCommand(command, timeoutMs)) as AndroidBridgeCommandResult;
}

export function disconnectAndroidUsbOtg(): void {
  getRawBridge()?.disconnectUsbOtg();
}

export function connectAndroidBleNus(timeoutMs = 10000): AndroidBridgeConnectResult {
  const bridge = getRawBridge();
  if (!bridge) {
    return { connected: false, message: 'Bridge BLE nativo disponível apenas no APK Android.' };
  }
  return JSON.parse(bridge.connectBleNus(timeoutMs)) as AndroidBridgeConnectResult;
}

export function sendAndroidBleCommand(
  command: string,
  timeoutMs = 4000
): AndroidBridgeCommandResult {
  const bridge = getRawBridge();
  if (!bridge) {
    return { ok: false, output: 'Bridge BLE nativo indisponível.' };
  }
  return JSON.parse(bridge.sendBleCommand(command, timeoutMs)) as AndroidBridgeCommandResult;
}

export function disconnectAndroidBleNus(): void {
  getRawBridge()?.disconnectBleNus();
}
