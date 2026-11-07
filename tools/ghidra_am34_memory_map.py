# Ghidra Script: Panasonic DMR-EH55 Memory Map & Symbols Setup
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
        ("INTERNAL_SRAM", 0x00000000, 0x00040000, True, True, True, False),
        ("SRAM_LOADER",   0x00004000, 0x00001000, True, True, True, False),
        ("SPANSION_FLASH",0x80000000, 0x00800000, True, False, True, False),
        ("NVRAM_EEMUL",   0x807F0000, 0x00010000, True, True, False, False),
        ("ASIC_MMIO",     0xC0000000, 0x00010000, True, True, False, True),
        ("ATA_IDE_MMIO",  0xD4000000, 0x00001000, True, True, False, True),
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
