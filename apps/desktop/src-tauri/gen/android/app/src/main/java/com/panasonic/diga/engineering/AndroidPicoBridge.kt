package com.panasonic.diga.engineering

import android.Manifest
import android.annotation.SuppressLint
import android.app.PendingIntent
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.hardware.usb.UsbConstants
import android.hardware.usb.UsbDevice
import android.hardware.usb.UsbDeviceConnection
import android.hardware.usb.UsbEndpoint
import android.hardware.usb.UsbInterface
import android.hardware.usb.UsbManager
import android.os.Build
import android.os.ParcelUuid
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.core.content.ContextCompat
import org.json.JSONArray
import org.json.JSONObject
import java.nio.charset.StandardCharsets
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

private const val USB_PERMISSION_ACTION = "com.panasonic.diga.engineering.USB_PERMISSION"
private const val RASPBERRY_PI_VID = 0x2E8A
private const val PICO_BOOTSEL_PID = 0x0003

private val NUS_SERVICE_UUID = UUID.fromString("6E400001-B5A3-F393-E0A9-E50E24DCCA9E")
private val NUS_RX_CHAR_UUID = UUID.fromString("6E400002-B5A3-F393-E0A9-E50E24DCCA9E")
private val NUS_TX_CHAR_UUID = UUID.fromString("6E400003-B5A3-F393-E0A9-E50E24DCCA9E")
private val CCCD_UUID = UUID.fromString("00002902-0000-1000-8000-00805F9B34FB")

/**
 * Native Android Hardware Bridge exposed to the WebView (`window.__PANADIGA_ANDROID_BRIDGE__`).
 * Enables physical Raspberry Pi Pico communication on Android over:
 * 1. USB-C OTG CDC-ACM (Raspberry Pi VID 0x2E8A) via Android UsbManager
 * 2. Bluetooth Low Energy Nordic UART Service (NUS) for Pico W / Pico 2 W
 */
class AndroidPicoBridge(
    private val activity: TauriActivity,
    private val webView: WebView
) {
    private val usbManager: UsbManager? =
        activity.getSystemService(Context.USB_SERVICE) as? UsbManager

    private val ioLock = Any()
    private var usbConn: UsbDeviceConnection? = null
    private var usbControlIface: UsbInterface? = null
    private var usbDataIface: UsbInterface? = null
    private var usbInEp: UsbEndpoint? = null
    private var usbOutEp: UsbEndpoint? = null
    private var activeUsbDeviceName: String = ""

    private var bleGatt: BluetoothGatt? = null
    private var bleRxChar: BluetoothGattCharacteristic? = null
    private val bleRxBuffer = StringBuilder()

    @JavascriptInterface
    fun isAvailable(): Boolean = true

    @JavascriptInterface
    fun getCapabilitiesJson(): String {
        val btMgr = activity.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
        val btAdapter = btMgr?.adapter
        val obj = JSONObject()
        obj.put("platform", "android")
        obj.put("usbHostSupported", activity.packageManager.hasSystemFeature(PackageManager.FEATURE_USB_HOST))
        obj.put("bleSupported", activity.packageManager.hasSystemFeature(PackageManager.FEATURE_BLUETOOTH_LE))
        obj.put("bluetoothEnabled", btAdapter?.isEnabled == true)
        return obj.toString()
    }

    @JavascriptInterface
    fun requestPermissions(): Boolean {
        val needed = mutableListOf<String>()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            needed += Manifest.permission.BLUETOOTH_SCAN
            needed += Manifest.permission.BLUETOOTH_CONNECT
        } else {
            needed += Manifest.permission.ACCESS_FINE_LOCATION
        }
        val missing = needed.filter {
            ContextCompat.checkSelfPermission(activity, it) != PackageManager.PERMISSION_GRANTED
        }
        if (missing.isEmpty()) return true
        val latch = CountDownLatch(1)
        var granted = false
        activity.runOnUiThread {
            activity.requestPermissions(missing.toTypedArray()) { ok ->
                granted = ok == true
                latch.countDown()
            }
        }
        latch.await(15, TimeUnit.SECONDS)
        return granted
    }

    @JavascriptInterface
    fun listUsbOtgDevicesJson(): String {
        val arr = JSONArray()
        val mgr = usbManager ?: return arr.toString()
        for (dev in mgr.deviceList.values) {
            val isRpi = dev.vendorId == RASPBERRY_PI_VID
            val isBootsel = isRpi && dev.productId == PICO_BOOTSEL_PID
            val item = JSONObject()
            item.put("deviceName", dev.deviceName)
            item.put("vendorId", dev.vendorId)
            item.put("productId", dev.productId)
            item.put("productName", dev.productName ?: "USB Serial Device")
            item.put("isRaspberryPi", isRpi)
            item.put("isBootsel", isBootsel)
            arr.put(item)
        }
        return arr.toString()
    }

    @JavascriptInterface
    fun connectUsbOtg(targetDeviceName: String): String {
        val res = JSONObject()
        val mgr = usbManager
        if (mgr == null) {
            res.put("connected", false)
            res.put("message", "Android UsbManager indisponível neste dispositivo.")
            return res.toString()
        }

        val dev = mgr.deviceList.values.firstOrNull {
            if (targetDeviceName.isNotBlank() && !targetDeviceName.startsWith("ANDROID-USB-OTG")) {
                it.deviceName == targetDeviceName
            } else {
                it.vendorId == RASPBERRY_PI_VID && it.productId != PICO_BOOTSEL_PID
            }
        } ?: mgr.deviceList.values.firstOrNull()

        if (dev == null) {
            res.put("connected", false)
            res.put("message", "Nenhum dispositivo Raspberry Pi Pico conectado via cabo USB-C OTG.")
            return res.toString()
        }

        if (!ensureUsbPermission(mgr, dev)) {
            res.put("connected", false)
            res.put("message", "Permissão USB OTG recusada pelo usuário.")
            return res.toString()
        }

        synchronized(ioLock) {
            closeUsbLocked()
            val conn = mgr.openDevice(dev)
            if (conn == null) {
                res.put("connected", false)
                res.put("message", "Falha ao abrir interface USB OTG para ${dev.deviceName}.")
                return res.toString()
            }

            var ctrlIf: UsbInterface? = null
            var dataIf: UsbInterface? = null
            var inEp: UsbEndpoint? = null
            var outEp: UsbEndpoint? = null

            for (i in 0 until dev.interfaceCount) {
                val intf = dev.getInterface(i)
                if (intf.interfaceClass == UsbConstants.USB_CLASS_COMM && ctrlIf == null) {
                    ctrlIf = intf
                }
                for (e in 0 until intf.endpointCount) {
                    val ep = intf.getEndpoint(e)
                    if (ep.type == UsbConstants.USB_ENDPOINT_XFER_BULK) {
                        if (ep.direction == UsbConstants.USB_DIR_IN && inEp == null) inEp = ep
                        if (ep.direction == UsbConstants.USB_DIR_OUT && outEp == null) outEp = ep
                        if (dataIf == null) dataIf = intf
                    }
                }
            }

            if (dataIf == null || inEp == null || outEp == null) {
                conn.close()
                res.put("connected", false)
                res.put("message", "Interface CDC ACM Bulk IN/OUT não encontrada em ${dev.productName ?: dev.deviceName}.")
                return res.toString()
            }

            if (ctrlIf != null) conn.claimInterface(ctrlIf, true)
            if (!conn.claimInterface(dataIf, true)) {
                conn.close()
                res.put("connected", false)
                res.put("message", "Falha ao reivindicar interface CDC Data USB OTG.")
                return res.toString()
            }

            configureCdcAcmLine(conn)
            usbConn = conn
            usbControlIface = ctrlIf
            usbDataIface = dataIf
            usbInEp = inEp
            usbOutEp = outEp
            activeUsbDeviceName = dev.deviceName
        }

        res.put("connected", true)
        res.put("deviceName", activeUsbDeviceName)
        res.put("message", "USB OTG CDC conectado: ${dev.productName ?: dev.deviceName} (115200 bps)")
        return res.toString()
    }

    @JavascriptInterface
    fun disconnectUsbOtg() {
        synchronized(ioLock) { closeUsbLocked() }
    }

    @JavascriptInterface
    fun sendUsbOtgCommand(command: String, timeoutMs: Int): String {
        val outObj = JSONObject()
        synchronized(ioLock) {
            val conn = usbConn
            val outEp = usbOutEp
            val inEp = usbInEp
            if (conn == null || outEp == null || inEp == null) {
                outObj.put("ok", false)
                outObj.put("output", "USB OTG desconectado.")
                return outObj.toString()
            }
            val payload = (command.trim() + "\r\n").toByteArray(StandardCharsets.UTF_8)
            val sent = conn.bulkTransfer(outEp, payload, payload.size, 1500)
            if (sent < 0) {
                outObj.put("ok", false)
                outObj.put("output", "Falha ao transmitir comando pela porta USB OTG.")
                return outObj.toString()
            }

            val buf = ByteArray(maxOf(64, inEp.maxPacketSize))
            val sb = StringBuilder()
            val deadline = System.currentTimeMillis() + timeoutMs.coerceIn(400, 35000)
            while (System.currentTimeMillis() < deadline) {
                val n = conn.bulkTransfer(inEp, buf, buf.size, 250)
                if (n > 0) {
                    sb.append(String(buf, 0, n, StandardCharsets.UTF_8))
                    val text = sb.toString()
                    if (text.contains("P7501>") || text.contains(": OK") || text.contains("PONG") || text.contains("[HANDSHAKE]")) {
                        break
                    }
                } else if (sb.isNotEmpty()) {
                    break
                }
            }
            outObj.put("ok", true)
            outObj.put("output", sb.toString().trim())
            return outObj.toString()
        }
    }

    @SuppressLint("MissingPermission")
    @JavascriptInterface
    fun connectBleNus(timeoutMs: Int): String {
        val res = JSONObject()
        if (!requestPermissions()) {
            res.put("connected", false)
            res.put("message", "Permissões Bluetooth não concedidas pelo sistema Android.")
            return res.toString()
        }
        val btMgr = activity.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
        val adapter: BluetoothAdapter? = btMgr?.adapter
        val scanner = adapter?.bluetoothLeScanner
        if (adapter == null || !adapter.isEnabled || scanner == null) {
            res.put("connected", false)
            res.put("message", "Bluetooth desativado no dispositivo Android. Ative o Bluetooth e tente novamente.")
            return res.toString()
        }

        val scanLatch = CountDownLatch(1)
        var connectedOk = false
        var deviceLabel = "Panadiga-BLE-Pico"
        var failReason = "Nenhuma sonda Pico W/2W anunciando Nordic UART Service encontrada."

        val scanCb = object : ScanCallback() {
            override fun onScanResult(callbackType: Int, result: ScanResult) {
                val dev = result.device ?: return
                scanner.stopScan(this)
                deviceLabel = dev.name ?: dev.address
                bleGatt?.close()
                bleGatt = dev.connectGatt(activity, false, object : BluetoothGattCallback() {
                    override fun onConnectionStateChange(gatt: BluetoothGatt, status: Int, newState: Int) {
                        if (newState == BluetoothProfile.STATE_CONNECTED) {
                            gatt.discoverServices()
                        } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
                            failReason = "Desconectado do GATT ($status)"
                            scanLatch.countDown()
                        }
                    }

                    override fun onServicesDiscovered(gatt: BluetoothGatt, status: Int) {
                        val svc = gatt.getService(NUS_SERVICE_UUID)
                        val rx = svc?.getCharacteristic(NUS_RX_CHAR_UUID)
                        val tx = svc?.getCharacteristic(NUS_TX_CHAR_UUID)
                        if (rx == null || tx == null) {
                            failReason = "Nordic UART Service ausente no periférico."
                            scanLatch.countDown()
                            return
                        }
                        bleRxChar = rx
                        gatt.setCharacteristicNotification(tx, true)
                        val desc = tx.getDescriptor(CCCD_UUID)
                        if (desc != null) {
                            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                                gatt.writeDescriptor(desc, BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
                            } else {
                                @Suppress("DEPRECATION")
                                desc.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
                                @Suppress("DEPRECATION")
                                gatt.writeDescriptor(desc)
                            }
                        }
                        connectedOk = true
                        scanLatch.countDown()
                    }

                    @Deprecated("Used below API 33")
                    override fun onCharacteristicChanged(gatt: BluetoothGatt, characteristic: BluetoothGattCharacteristic) {
                        @Suppress("DEPRECATION")
                        val bytes = characteristic.value ?: return
                        handleIncomingBleBytes(bytes)
                    }

                    override fun onCharacteristicChanged(gatt: BluetoothGatt, characteristic: BluetoothGattCharacteristic, value: ByteArray) {
                        handleIncomingBleBytes(value)
                    }
                })
            }

            override fun onScanFailed(errorCode: Int) {
                failReason = "Falha BLE Scan (código $errorCode)"
                scanLatch.countDown()
            }
        }

        val filters = listOf(ScanFilter.Builder().setServiceUuid(ParcelUuid(NUS_SERVICE_UUID)).build())
        val settings = ScanSettings.Builder().setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY).build()
        scanner.startScan(filters, settings, scanCb)
        val completed = scanLatch.await(timeoutMs.coerceIn(3000, 15000).toLong(), TimeUnit.MILLISECONDS)
        if (!completed) {
            scanner.stopScan(scanCb)
        }
        res.put("connected", connectedOk)
        res.put("deviceName", deviceLabel)
        res.put("message", if (connectedOk) "BLE Nordic UART conectado: $deviceLabel" else failReason)
        return res.toString()
    }

    @SuppressLint("MissingPermission")
    @JavascriptInterface
    fun sendBleCommand(command: String, timeoutMs: Int): String {
        val outObj = JSONObject()
        val gatt = bleGatt
        val rx = bleRxChar
        if (gatt == null || rx == null) {
            outObj.put("ok", false)
            outObj.put("output", "BLE NUS não conectado.")
            return outObj.toString()
        }
        synchronized(bleRxBuffer) { bleRxBuffer.setLength(0) }
        val payload = (command.trim() + "\r\n").toByteArray(StandardCharsets.UTF_8)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            gatt.writeCharacteristic(rx, payload, BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT)
        } else {
            @Suppress("DEPRECATION")
            rx.value = payload
            @Suppress("DEPRECATION")
            gatt.writeCharacteristic(rx)
        }

        val deadline = System.currentTimeMillis() + timeoutMs.coerceIn(500, 35000)
        while (System.currentTimeMillis() < deadline) {
            val snap = synchronized(bleRxBuffer) { bleRxBuffer.toString() }
            if (snap.contains("P7501>") || snap.contains(": OK") || snap.contains("PONG") || snap.contains("[HANDSHAKE]")) {
                break
            }
            Thread.sleep(60)
        }
        val output = synchronized(bleRxBuffer) { bleRxBuffer.toString().trim() }
        outObj.put("ok", true)
        outObj.put("output", output)
        return outObj.toString()
    }

    @SuppressLint("MissingPermission")
    @JavascriptInterface
    fun disconnectBleNus() {
        bleGatt?.disconnect()
        bleGatt?.close()
        bleGatt = null
        bleRxChar = null
    }

    private fun handleIncomingBleBytes(bytes: ByteArray) {
        val text = String(bytes, StandardCharsets.UTF_8)
        synchronized(bleRxBuffer) { bleRxBuffer.append(text) }
        val escaped = JSONObject.quote(text)
        activity.runOnUiThread {
            webView.evaluateJavascript(
                "window.dispatchEvent(new CustomEvent('panadiga-ble-rx', { detail: $escaped }));",
                null
            )
        }
    }

    private fun ensureUsbPermission(mgr: UsbManager, dev: UsbDevice): Boolean {
        if (mgr.hasPermission(dev)) return true
        val latch = CountDownLatch(1)
        var granted = false
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                if (intent?.action == USB_PERMISSION_ACTION) {
                    granted = intent.getBooleanExtra(UsbManager.EXTRA_PERMISSION_GRANTED, false)
                    latch.countDown()
                }
            }
        }
        val filter = IntentFilter(USB_PERMISSION_ACTION)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            activity.registerReceiver(receiver, filter, Context.RECEIVER_NOT_EXPORTED)
        } else {
            @Suppress("DEPRECATION")
            activity.registerReceiver(receiver, filter)
        }
        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) PendingIntent.FLAG_MUTABLE else 0
        val pi = PendingIntent.getBroadcast(activity, 0, Intent(USB_PERMISSION_ACTION), flags)
        mgr.requestPermission(dev, pi)
        latch.await(20, TimeUnit.SECONDS)
        runCatching { activity.unregisterReceiver(receiver) }
        return granted
    }

    private fun configureCdcAcmLine(conn: UsbDeviceConnection) {
        // 115200 bps, 1 stop bit (0), no parity (0), 8 data bits (8)
        val lineCoding = byteArrayOf(0x00, 0xC2.toByte(), 0x01, 0x00, 0x00, 0x00, 0x08)
        conn.controlTransfer(0x21, 0x20, 0, 0, lineCoding, lineCoding.size, 500)
        // Assert DTR (bit 0) + RTS (bit 1) so TinyUSB stdio recognizes terminal active
        conn.controlTransfer(0x21, 0x22, 0x0003, 0, null, 0, 500)
    }

    private fun closeUsbLocked() {
        usbDataIface?.let { usbConn?.releaseInterface(it) }
        usbControlIface?.let { usbConn?.releaseInterface(it) }
        usbConn?.close()
        usbConn = null
        usbControlIface = null
        usbDataIface = null
        usbInEp = null
        usbOutEp = null
        activeUsbDeviceName = ""
    }
}
