# Ghidra Script: Panasonic DMR-EH55 Symbol Importer (Backward-Compatibility Wrapper)
# @category Forensics.Panasonic
# @author Panadiga Engineering Team
import runpy
from pathlib import Path

_target = Path(__file__).parent / "ghidra_import_symbols_panadiga.py"
runpy.run_path(str(_target), run_name="__main__")
