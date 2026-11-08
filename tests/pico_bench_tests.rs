use panadiga::pico::{
    classify_usb_device, list_pico_devices, HostControllerPlatform, PicoArchitecture,
    ARDUINO_NANO_RP2040_CONNECT_PID, ARDUINO_VID, PICO_BOOTSEL_PID, PICO_RP2040_CDC_PID,
    PICO_RP2350_CDC_PID, RASPBERRY_PI_VID, SEEED_VID, SEEED_XIAO_RP2040_PID, SEEED_XIAO_RP2350_PID,
};

#[test]
fn test_pico_architecture_and_device_listing() {
    assert_eq!(RASPBERRY_PI_VID, 0x2E8A);
    assert_eq!(PICO_RP2040_CDC_PID, 0x000A);
    assert_eq!(PICO_RP2350_CDC_PID, 0x000F);
    assert_eq!(PICO_BOOTSEL_PID, 0x0003);

    assert_eq!(
        PicoArchitecture::Rp2040.to_string(),
        "Raspberry Pi Pico (RP2040)"
    );
    assert_eq!(
        PicoArchitecture::Rp2350.to_string(),
        "Raspberry Pi Pico 2 (RP2350)"
    );
    assert_eq!(
        PicoArchitecture::Bootloader.to_string(),
        "Raspberry Pi BOOTSEL Mode"
    );

    // Multi-Platform classification tests across vendors
    let (plat_pico, _) = classify_usb_device(RASPBERRY_PI_VID, PICO_RP2040_CDC_PID, None);
    assert_eq!(plat_pico, HostControllerPlatform::RaspberryPiPico);
    assert_eq!(plat_pico.sram_size_bytes(), 264 * 1024);
    assert_eq!(plat_pico.sector_cache_capacity_bytes(), 8 * 1024);
    assert!(plat_pico.supports_dwire32a_pio());

    let (plat_pico2, _) = classify_usb_device(RASPBERRY_PI_VID, PICO_RP2350_CDC_PID, None);
    assert_eq!(plat_pico2, HostControllerPlatform::RaspberryPiPico2Arm);
    assert_eq!(plat_pico2.sram_size_bytes(), 520 * 1024);
    assert_eq!(plat_pico2.sector_cache_capacity_bytes(), 64 * 1024);
    assert!(plat_pico2.supports_dwire32a_pio());

    let (plat_arduino, _) = classify_usb_device(ARDUINO_VID, ARDUINO_NANO_RP2040_CONNECT_PID, None);
    assert_eq!(plat_arduino, HostControllerPlatform::ArduinoNanoRp2040);
    assert_eq!(plat_arduino.architecture(), PicoArchitecture::Rp2040);

    let (plat_seeed2040, _) = classify_usb_device(SEEED_VID, SEEED_XIAO_RP2040_PID, None);
    assert_eq!(plat_seeed2040, HostControllerPlatform::SeeedXiaoRp2040);

    let (plat_seeed2350, _) = classify_usb_device(SEEED_VID, SEEED_XIAO_RP2350_PID, None);
    assert_eq!(plat_seeed2350, HostControllerPlatform::SeeedXiaoRp2350);
    assert_eq!(plat_seeed2350.architecture(), PicoArchitecture::Rp2350);

    // Enumerating system serial devices must execute cleanly without crashing
    let devices = list_pico_devices();
    assert!(devices.is_ok());
}

#[test]
fn test_all_host_controller_platforms_matrix() {
    // Pico W
    let (plat_w, _) = classify_usb_device(0x2E8A, 0x0014, None);
    assert_eq!(plat_w, HostControllerPlatform::RaspberryPiPicoW);
    assert_eq!(plat_w.sram_size_bytes(), 264 * 1024);
    assert_eq!(plat_w.sector_cache_capacity_bytes(), 8 * 1024);

    // Pico 2 W
    let (plat_2w, _) = classify_usb_device(0x2E8A, 0x001F, None);
    assert_eq!(plat_2w, HostControllerPlatform::RaspberryPiPico2W);
    assert_eq!(plat_2w.sram_size_bytes(), 520 * 1024);
    assert_eq!(plat_2w.sector_cache_capacity_bytes(), 64 * 1024);

    // Pico 2 RISC-V Hazard3
    let (plat_riscv, _) = classify_usb_device(0x2E8A, 0x000E, None);
    assert_eq!(plat_riscv, HostControllerPlatform::RaspberryPiPico2Riscv);
    assert_eq!(plat_riscv.sram_size_bytes(), 520 * 1024);
    assert_eq!(plat_riscv.sector_cache_capacity_bytes(), 64 * 1024);

    // Adafruit Feather RP2040
    let (plat_ada, _) = classify_usb_device(0x239A, 0x80F0, Some("Adafruit Feather RP2040"));
    assert_eq!(plat_ada, HostControllerPlatform::AdafruitRp2040);
    assert_eq!(plat_ada.architecture(), PicoArchitecture::Rp2040);

    // SparkFun Pro Micro RP2040
    let (plat_spark, _) = classify_usb_device(0x1B4F, 0x0025, None);
    assert_eq!(plat_spark, HostControllerPlatform::SparkfunRp2040);
    assert_eq!(plat_spark.architecture(), PicoArchitecture::Rp2040);

    // Waveshare RP2040
    let (plat_wave, _) = classify_usb_device(0x1209, 0x2040, Some("Waveshare RP2040-Zero"));
    assert_eq!(plat_wave, HostControllerPlatform::WaveshareRp2040);
    assert_eq!(plat_wave.architecture(), PicoArchitecture::Rp2040);

    // Custom DWire32A probe with banner string
    let (plat_custom, _) = classify_usb_device(
        0x9999,
        0x0001,
        Some("Panasonic DWire32A OCD Host Bridge [DW32A]"),
    );
    assert_eq!(plat_custom, HostControllerPlatform::GenericCompatibleCdc);
    assert_eq!(plat_custom.architecture(), PicoArchitecture::Unknown);
}
