# Ghidra Script: Panasonic DMR-EH55 Memory Map & Symbols Setup
# @category Forensics.Panasonic
# @author Panadiga Engineering Team
#
# Configures the complete memory map of Panasonic MN2DS0016 (AM34 / MN103S)
# matching the DMR-EH55 VEP79138 digital board layout.
#
# Usage in Ghidra:
# 1. Open the Spansion S29JL064H 8MB flash dump (Language: MN10300).
# 2. In Ghidra Script Manager, run this script.
# 3. All hardware regions (SRAM, Flash, NVRAM, ASIC MMIO, IDE) are created.

from ghidra.program.model.address import AddressSpace
from ghidra.program.model.mem import MemoryAccessException

def setup_memory_map():
    mem = currentProgram.getMemory()
    addr_factory = currentProgram.getAddressFactory()
    default_space = addr_factory.getDefaultAddressSpace()

    regions = [
        # (Name, StartHex, SizeBytes, Read, Write, Execute, IsVolatile)
        ("INTERNAL_SRAM", 0x00000000, 0x00040000, True, True, True, False),    # 256 KB Work RAM
        ("SRAM_LOADER",   0x00004000, 0x00001000, True, True, True, False),    # RAM Loader Injection Area
        ("SPANSION_FLASH",0x80000000, 0x00800000, True, False, True, False),   # 8 MB NOR Flash Mirror
        ("NVRAM_EEMUL",   0x807F0000, 0x00010000, True, True, False, False),   # 64 KB SA134..SA141 EEMUL
        ("ASIC_MMIO",     0xC0000000, 0x00010000, True, True, False, True),    # UniPhier System MMIO
        ("ATA_IDE_MMIO",  0xD4000000, 0x00001000, True, True, False, True),    # IDE PATA Controller Registers
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
