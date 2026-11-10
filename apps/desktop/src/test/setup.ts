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
          success: true,
          message: "Connected to Raspberry Pi Pico (RP2040) on COM5",
          detected_architecture: "Raspberry Pi Pico (RP2040)",
          chip_family: "RP2040",
          sram_size_bytes: 270336,
          flash_sector_cache_size: 8192,
          am34_registers: {
            d0: 0x00000000,
            d1: 0x00000001,
            d2: 0x00000002,
            d3: 0x00000003,
            a0: 0x10000000,
            a1: 0x10000100,
            a2: 0x10000200,
            a3: 0x10000300,
            sp: 0x1003fff0,
            pc: 0x80000000,
            mdr: 0x00000000,
            psw: 0x00000040,
          },
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
