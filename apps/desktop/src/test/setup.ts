import { vi } from "vitest";
import { mockNvramCleanReport } from "../services/mocks/mockNvram";
import { handleMockCommand } from "../services/mockDispatcher";

// Mock Tauri core IPC
vi.mock("@tauri-apps/api/core", () => ({
  isTauri: vi.fn(() => true),
  invoke: vi.fn().mockImplementation((cmd: string, args?: unknown) => {
    switch (cmd) {
      case "clean_nvram":
      case "analyze_flash_dump":
        return Promise.resolve(mockNvramCleanReport);
      case "list_pico_serial_ports":
        return Promise.resolve([
          {
            port_name: "COM5",
            vid: 0x2e8a,
            pid: 0x000a,
            product: "Raspberry Pi Pico (RP2040)",
            manufacturer: "Raspberry Pi",
            architecture: "Raspberry Pi Pico (RP2040)",
            chip_family: "RP2040",
            sram_size_bytes: 270336,
            sector_cache_capacity_bytes: 8192,
            supports_dwire32a: true,
          },
        ]);
      case "pico_connect_and_test":
        return Promise.resolve({
          connected: true,
          port_name: "COM5",
          baud_rate: 115200,
          vdd_voltage: 3.28,
          target_powered: true,
          firmware_banner: "Raspberry Pi Pico (RP2040) DWire32A v2.0",
          is_mock: false,
          message: "Conectado ao Raspberry Pi Pico (RP2040) na porta COM5",
        });
      case "pico_send_serial_command":
      case "pico_send_command":
        return Promise.resolve("VDD=3.28V | OCD=HALTED | PC=0x80000000 | MODE=2");
      case "pico_start_gdb_server":
        return Promise.resolve({
          port: 3333,
          is_running: true,
          connected_clients: 0,
        });
      case "pico_stop_gdb_server":
        return Promise.resolve({
          port: 3333,
          is_running: false,
          connected_clients: 0,
        });
      default:
        return handleMockCommand(cmd, args as Record<string, unknown>);
    }
  }),
}));

// Mock window matchMedia
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
