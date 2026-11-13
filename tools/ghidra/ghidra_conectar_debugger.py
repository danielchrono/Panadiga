# Ghidra Script: Panasonic DMR-EH55 Connect GDB Debugger (Backward-Compatibility Wrapper)
# @category Forensics.Panasonic
# @author Panadiga Engineering Team
import runpy
from pathlib import Path

_target = Path(__file__).parent / "ghidra_connect_debugger.py"
runpy.run_path(str(_target), run_name="__main__")
