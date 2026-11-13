# Ghidra Script: Panasonic DMR-EH55 Connect GDB Debugger to Panadiga Bridge
# @category Forensics.Panasonic
# @author Panadiga Engineering Team
#
# Configures Ghidra's Debugger tool to connect directly to the Panadiga
# GDB Remote Serial Protocol (RSP) bridge running on localhost:3333.
#
# Prerequisites:
# 1. Start the GDB bridge in Panadiga Desktop (or via Panadiga CLI).
# 2. Open Ghidra Debugger perspective (Window -> Debugger).
# 3. In the Debugger Targets window, click the plug/connector icon:
#    - Model: "gdb" (GNU Debugger)
#    - GDB launch command: "gdb-multiarch"
#    - Use script or enter GDB command: "target remote 127.0.0.1:3333"

import sys

def print_connection_instructions():
    print("=" * 70)
    print("  PANADIGA SUITE - GHIDRA DEBUGGER AM34 / P7501 CONNECTION")
    print("=" * 70)
    print("")
    print("1. Bridge Status Check:")
    print("   Ensure the Panadiga GDB Bridge is active on 127.0.0.1:3333")
    print("   (Either click 'Start GDB Server' in Panadiga Desktop, or")
    print("    execute: cargo run -- gdb-config)")
    print("")
    print("2. In Ghidra Debugger Window:")
    print("   - Open 'Debugger' perspective (Window -> Debugger)")
    print("   - In 'Objects / Targets' panel, click the 'Connect' icon")
    print("   - Select Connector: 'GNU gdb local IN-VM' or 'gdb via SSH/local'")
    print("   - Set 'GDB launch command': gdb-multiarch")
    print("")
    print("3. Commands to execute in Ghidra Debugger GDB Console:")
    print("   set architecture mn10300")
    print("   set endian little")
    print("   target remote 127.0.0.1:3333")
    print("   display/i $pc")
    print("")
    print("4. Live Target Sync:")
    print("   Once connected, Ghidra's Dynamic Listing will trace live")
    print("   execution on the Panasonic DMR-EH55 motherboard in real-time!")
    print("=" * 70)

if __name__ == "__main__":
    print_connection_instructions()
