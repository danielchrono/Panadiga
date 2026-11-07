//! Panasonic DMR-EH55 Ghidra Integration Generator
//!
//! Generates official Ghidra Jython/PyGhidra memory mapping and symbol annotation
//! scripts directly from Rust constants (`constants.rs`), guaranteeing that Rust
//! is the Single Source of Truth (SSOT) across all reverse engineering environments.

use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use crate::constants::*;

/// Generates the Ghidra Memory Map setup Python script.
pub fn generate_ghidra_memory_map() -> String {
    format!(
r#"# Ghidra Script: Panasonic DMR-EH55 Memory Map & Symbols Setup
# @category Forensics.Panasonic
# @author Panadiga Engineering Team (Generated from native Rust SSOT)
#
# Configures the complete memory map of Panasonic MN2DS0016 (AM34 / MN103S)
# matching the DMR-EH55 VEP79138 digital board layout.

from ghidra.program.model.address import AddressSpace
from ghidra.program.model.mem import MemoryAccessException

def setup_memory_map():
    mem = currentProgram.getMemory()
    addr_factory = currentProgram.getAddressFactory()
    default_space = addr_factory.getDefaultAddressSpace()

    regions = [
        # (Name, StartHex, SizeBytes, Read, Write, Execute, IsVolatile)
        ("INTERNAL_SRAM", 0x{sram_base:08X}, 0x{sram_size:08X}, True, True, True, False),
        ("CORE_MMIO",     0x{core_mmio_base:08X}, 0x{core_mmio_size:08X}, True, True, False, True),
        ("MAIN_SDRAM",    0x{sdram_base:08X}, 0x{sdram_size:08X}, True, True, True, False),
        ("FLASH_NOR_IPL", 0x{flash_base:08X}, 0x{flash_ipl_size:08X}, True, False, True, False),
        ("NVRAM_EEMUL",   0x{nvram_base:08X}, 0x{nvram_size:08X}, True, True, False, False),
        ("ASIC_MMIO",     0x{mmio_base:08X}, 0x{mmio_size:08X}, True, True, False, True),
        ("ATA_IDE_MMIO",  0x{pata_base:08X}, 0x{pata_size:08X}, True, True, False, True),
    ]

    print("[Panadiga] Configuring Panasonic DMR-EH55 Memory Map...")

    for name, start, size, r, w, x, vol in regions:
        addr = default_space.getAddress(start)
        existing = mem.getBlock(addr)
        if existing is None:
            try:
                block = mem.createUninitializedBlock(name, addr, size, False)
                block.setRead(r)
                block.setWrite(w)
                block.setExecute(x)
                block.setVolatile(vol)
                block.setComment("Panasonic DMR-EH55 HW Subsystem: " + name)
                print("  [+] Created block: %s at 0x%08X (size: 0x%X)" % (name, start, size))
            except Exception as e:
                print("  [-] Could not create block %s: %s" % (name, str(e)))
        else:
            print("  [*] Block %s already mapped at 0x%08X" % (existing.getName(), start))

    print("[Panadiga] Memory Map setup complete!")

if __name__ == "__main__":
    setup_memory_map()
"#,
        sram_base = AM34_INTERNAL_SRAM_BASE,
        sram_size = AM34_INTERNAL_SRAM_SIZE,
        core_mmio_base = AM34_MMIO_BASE,
        core_mmio_size = AM34_MMIO_SIZE,
        sdram_base = AM34_SDRAM_BASE,
        sdram_size = AM34_SDRAM_SIZE,
        flash_base = AM34_FLASH_BASE,
        flash_ipl_size = AM34_FLASH_IPL_SIZE,
        nvram_base = AM34_NVRAM_BASE,
        nvram_size = AM34_NVRAM_SIZE,
        mmio_base = AM34_ASIC_MMIO_BASE,
        mmio_size = AM34_ASIC_MMIO_SIZE,
        pata_base = AM34_PATA_MMIO_BASE,
        pata_size = AM34_PATA_MMIO_SIZE,
    )
}

/// Generates the Ghidra Symbols and Peripheral importer script.
pub fn generate_ghidra_symbols() -> String {
    format!(
r#"# Ghidra Script: Panasonic DMR-EH55 Symbol & Peripheral Importer
# @category Forensics.Panasonic
# @author Panadiga Engineering Team (Generated from native Rust SSOT)

from ghidra.program.model.symbol import SourceType

def import_symbols():
    sym_table = currentProgram.getSymbolTable()
    listing = currentProgram.getListing()
    addr_factory = currentProgram.getAddressFactory()
    default_space = addr_factory.getDefaultAddressSpace()

    symbols = [
        # AM34 Reset and Interrupt Vectors
        (0x{flash_base:08X}, "_start_reset_vector", "AM34 Reset Vector - Cold Boot Entrypoint", True),
        (0x{flash_base_plus_4:08X}, "vector_nmi", "Non-Maskable Interrupt Handler", True),
        (0x{flash_base_plus_8:08X}, "vector_trap", "Software Trap Exception Handler", True),
        (0x{flash_base_plus_c:08X}, "vector_illegal_instr", "Illegal Instruction Exception Handler", True),

        # NVRAM 8KB Micro-Sectors (SA134..SA141)
        (0x{nvram_base:08X}, "NVRAM_SECTOR_SA134", "NVRAM Micro-Sector 0: System Configuration & Boot Flags", True),
        (0x{nvram_sa135:08X}, "NVRAM_SECTOR_SA135", "NVRAM Micro-Sector 1: Tuner Channel Map & Presets", True),
        (0x{nvram_sa136:08X}, "NVRAM_SECTOR_SA136", "NVRAM Micro-Sector 2: Scheduled Timer Recordings & EPG", True),
        (0x{nvram_sa137:08X}, "NVRAM_SECTOR_SA137", "NVRAM Micro-Sector 3: User Preferences & AV Modes", True),
        (0x{nvram_sa138:08X}, "NVRAM_SECTOR_SA138", "NVRAM Micro-Sector 4: Optical Drive Pickup Laser Calibration", True),
        (0x{nvram_sa139:08X}, "NVRAM_SECTOR_SA139", "NVRAM Micro-Sector 5: HDD Serial Binding & Volume Identification", True),
        (0x{nvram_sa140:08X}, "NVRAM_SECTOR_SA140", "NVRAM Micro-Sector 6: Factory Diagnostics & Self-Test Flags", True),
        (0x{nvram_sa141:08X}, "NVRAM_SECTOR_SA141_U99", "NVRAM Micro-Sector 7: Locus of U99 Corrupted Hardware Error Logs", True),

        # UniPhier MMIO Peripherals
        (0x{mmio_base:08X}, "MMIO_CHIP_ID", "Panasonic UniPhier MN2DS0016 Silicon Revision ID", True),
        (0x{mmio_base_plus_4:08X}, "MMIO_RESET_CTRL", "Subsystem Reset Register (Software Reset Controller)", True),
        (0x{mmio_base_plus_10:08X}, "MMIO_PLL_CLOCK_CFG", "PLL Multiplier and Clock Gating Control", True),
        (0x{mmio_uart:08X}, "MMIO_UART0_DATA", "Serial UART0 TX/RX FIFO Register (DWire / Diagnostic bus)", True),
        (0x{mmio_uart_status:08X}, "MMIO_UART0_STATUS", "UART0 Status: TX Empty, RX Ready", True),
        (0x{mmio_uart_baud:08X}, "MMIO_UART0_BAUD_DIV", "UART0 Baud Rate Clock Prescaler / Divisor", True),

        # IDE PATA Interface (MEIHDFS HDD)
        (0x{pata_base:08X}, "PATA_DATA_16BIT", "PATA 16-bit PIO / DMA Data Register", True),
        (0x{pata_base_plus_1:08X}, "PATA_ERROR_FEATURES", "PATA Error Register (Read) / Features Register (Write)", True),
        (0x{pata_base_plus_2:08X}, "PATA_SECTOR_COUNT", "PATA Sector Count Register", True),
        (0x{pata_base_plus_7:08X}, "PATA_STATUS_COMMAND", "PATA Status Register (Read) / Command Register (Write)", True),

        # In-Circuit SRAM RAM Loader
        (0x{loader_base:08X}, "RAM_LOADER_ENTRY", "Panadiga In-Circuit RAM Loader Entrypoint", True),
    ]

    print("[Panadiga] Annotating Panasonic DMR-EH55 Symbols and Peripherals...")
    imported_count = 0

    for addr_val, name, comment, is_primary in symbols:
        addr = default_space.getAddress(addr_val)
        try:
            sym = sym_table.createLabel(addr, name, SourceType.USER_DEFINED)
            if is_primary:
                sym_table.setPrimary(sym)
            listing.setComment(addr, listing.PLATE_COMMENT, comment)
            imported_count += 1
            print("  [+] Labeled 0x%08X -> %s" % (addr_val, name))
        except Exception as e:
            listing.setComment(addr, listing.PLATE_COMMENT, comment)
            print("  [*] Updated comment at 0x%08X -> %s (%s)" % (addr_val, name, str(e)))

    print("[Panadiga] Symbol import complete! Annotated %d symbols." % imported_count)

if __name__ == "__main__":
    import_symbols()
"#,
        flash_base = AM34_FLASH_BASE,
        flash_base_plus_4 = AM34_FLASH_BASE + 0x04,
        flash_base_plus_8 = AM34_FLASH_BASE + 0x08,
        flash_base_plus_c = AM34_FLASH_BASE + 0x0C,
        nvram_base = AM34_NVRAM_BASE,
        nvram_sa135 = AM34_NVRAM_BASE + 0x2000,
        nvram_sa136 = AM34_NVRAM_BASE + 0x4000,
        nvram_sa137 = AM34_NVRAM_BASE + 0x6000,
        nvram_sa138 = AM34_NVRAM_BASE + 0x8000,
        nvram_sa139 = AM34_NVRAM_BASE + 0xA000,
        nvram_sa140 = AM34_NVRAM_BASE + 0xC000,
        nvram_sa141 = AM34_NVRAM_BASE + 0xE000,
        mmio_base = AM34_ASIC_MMIO_BASE,
        mmio_base_plus_4 = AM34_ASIC_MMIO_BASE + 0x04,
        mmio_base_plus_10 = AM34_ASIC_MMIO_BASE + 0x10,
        mmio_uart = AM34_ASIC_MMIO_BASE + 0x100,
        mmio_uart_status = AM34_ASIC_MMIO_BASE + 0x104,
        mmio_uart_baud = AM34_ASIC_MMIO_BASE + 0x108,
        pata_base = AM34_PATA_MMIO_BASE,
        pata_base_plus_1 = AM34_PATA_MMIO_BASE + 0x01,
        pata_base_plus_2 = AM34_PATA_MMIO_BASE + 0x02,
        pata_base_plus_7 = AM34_PATA_MMIO_BASE + 0x07,
        loader_base = AM34_SRAM_LOADER_BASE,
    )
}

/// Exports all Ghidra scripts to the target directory.
pub fn export_all_ghidra_scripts(target_dir: &Path) -> Result<()> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed creating directory {:?}", target_dir))?;
    }

    let map_path = target_dir.join("ghidra_am34_memory_map.py");
    let sym_path_en = target_dir.join("ghidra_import_symbols_panadiga.py");
    let sym_path_pt = target_dir.join("ghidra_importar_simbolos_panadiga.py");

    fs::write(&map_path, generate_ghidra_memory_map())
        .with_context(|| format!("Failed writing {:?}", map_path))?;
    fs::write(&sym_path_en, generate_ghidra_symbols())
        .with_context(|| format!("Failed writing {:?}", sym_path_en))?;

    let wrapper_pt = r#"# Ghidra Script: Panasonic DMR-EH55 Symbol Importer (Backward-Compatibility Wrapper)
# @category Forensics.Panasonic
# @author Panadiga Engineering Team
import runpy
from pathlib import Path

_target = Path(__file__).parent / "ghidra_import_symbols_panadiga.py"
runpy.run_path(str(_target), run_name="__main__")
"#;
    fs::write(&sym_path_pt, wrapper_pt)
        .with_context(|| format!("Failed writing {:?}", sym_path_pt))?;

    Ok(())
}
