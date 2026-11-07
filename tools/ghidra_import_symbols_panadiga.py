# Ghidra Script: Panasonic DMR-EH55 Symbol & Peripheral Importer
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
        (0x80000000, "_start_reset_vector", "AM34 Reset Vector - Cold Boot Entrypoint", True),
        (0x80000004, "vector_nmi", "Non-Maskable Interrupt Handler", True),
        (0x80000008, "vector_trap", "Software Trap Exception Handler", True),
        (0x8000000C, "vector_illegal_instr", "Illegal Instruction Exception Handler", True),

        # NVRAM 8KB Micro-Sectors (SA134..SA141)
        (0x807F0000, "NVRAM_SECTOR_SA134", "NVRAM Micro-Sector 0: System Configuration & Boot Flags", True),
        (0x807F2000, "NVRAM_SECTOR_SA135", "NVRAM Micro-Sector 1: Tuner Channel Map & Presets", True),
        (0x807F4000, "NVRAM_SECTOR_SA136", "NVRAM Micro-Sector 2: Scheduled Timer Recordings & EPG", True),
        (0x807F6000, "NVRAM_SECTOR_SA137", "NVRAM Micro-Sector 3: User Preferences & AV Modes", True),
        (0x807F8000, "NVRAM_SECTOR_SA138", "NVRAM Micro-Sector 4: Optical Drive Pickup Laser Calibration", True),
        (0x807FA000, "NVRAM_SECTOR_SA139", "NVRAM Micro-Sector 5: HDD Serial Binding & Volume Identification", True),
        (0x807FC000, "NVRAM_SECTOR_SA140", "NVRAM Micro-Sector 6: Factory Diagnostics & Self-Test Flags", True),
        (0x807FE000, "NVRAM_SECTOR_SA141_U99", "NVRAM Micro-Sector 7: Locus of U99 Corrupted Hardware Error Logs", True),

        # UniPhier MMIO Peripherals
        (0xC0000000, "MMIO_CHIP_ID", "Panasonic UniPhier MN2DS0016 Silicon Revision ID", True),
        (0xC0000004, "MMIO_RESET_CTRL", "Subsystem Reset Register (Software Reset Controller)", True),
        (0xC0000010, "MMIO_PLL_CLOCK_CFG", "PLL Multiplier and Clock Gating Control", True),
        (0xC0000100, "MMIO_UART0_DATA", "Serial UART0 TX/RX FIFO Register (DWire / Diagnostic bus)", True),
        (0xC0000104, "MMIO_UART0_STATUS", "UART0 Status: TX Empty, RX Ready", True),
        (0xC0000108, "MMIO_UART0_BAUD_DIV", "UART0 Baud Rate Clock Prescaler / Divisor", True),

        # IDE PATA Interface (MEIHDFS HDD)
        (0xD4000000, "PATA_DATA_16BIT", "PATA 16-bit PIO / DMA Data Register", True),
        (0xD4000001, "PATA_ERROR_FEATURES", "PATA Error Register (Read) / Features Register (Write)", True),
        (0xD4000002, "PATA_SECTOR_COUNT", "PATA Sector Count Register", True),
        (0xD4000007, "PATA_STATUS_COMMAND", "PATA Status Register (Read) / Command Register (Write)", True),

        # In-Circuit SRAM RAM Loader
        (0x00004000, "RAM_LOADER_ENTRY", "Panadiga In-Circuit RAM Loader Entrypoint", True),
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
