"""Configuration and hardware safety invariants for Panasonic DIGA DMR-EH55 RAG Engine.
"""

from pathlib import Path
from typing import Final, List, Set

# Base directory paths
BASE_DIR: Path = Path(__file__).resolve().parent.parent.parent
DOCS_DIR: Path = BASE_DIR / "docs"
MANUALS_DIR: Path = BASE_DIR / "manuals_and_schematics"

# ==============================================================================
# HARDWARE PHYSICAL SAFETY INVARIANTS (NON-NEGOTIABLE)
# ==============================================================================

# Maximum allowed DC voltage on digital PCB (VEP79138) test ports, CN55001, and Flash
MAX_SAFE_VOLTAGE_VOLTS: Final[float] = 3.35  # Absolute max is 3.3V nominal (+5% tolerance)

# Spansion S29JL064H Flash Memory Map (8MB / 64Mbit)
FLASH_TOTAL_SIZE_BYTES: Final[int] = 0x800000  # 8,388,608 bytes (8 MB)

# SA0..SA133: IPL (Initial Program Loader) & Root of Trust. NEVER TOUCH IN U99 REPAIR!
FORBIDDEN_FLASH_WRITE_START: Final[int] = 0x00000000
FORBIDDEN_FLASH_WRITE_END: Final[int] = 0x007EFFFF

# SA134..SA141: NVRAM / EEMUL (8 x 8KB microsectors = 64KB total). ONLY SAFE TARGET FOR U99 REPAIR!
SAFE_NVRAM_BASE_OFFSET: Final[int] = 0x007F0000
SAFE_NVRAM_END_OFFSET: Final[int] = 0x007FFFFF
SAFE_NVRAM_SIZE_BYTES: Final[int] = 0x00010000  # 64 KB

# Matsushita DWire32A Protocol vs Standard JTAG
# CN55001 has 6 pins: EXTRG (Pin 1), SCLK (Pin 2), GND (Pin 3), SDATA (Pin 4), nRESET (Pin 5), NC (Pin 6)
# It DOES NOT HAVE TDI, TDO, TMS!
FORBIDDEN_JTAG_PINS: Final[Set[str]] = {"TDI", "TDO", "TMS", "TRST", "TCK"}
VALID_DWIRE_PINS: Final[Set[str]] = {"SDATA", "SCLK", "EXTRG", "NRESET", "GND"}

# MEIHDFS Hard Drive Structure Invariants
MEIHDFS_LBA0_OFFSET: Final[int] = 0x00000000
MEIHDFS_MAGIC_BYTES: Final[str] = "SH"
MEIHDFS_MIRROR_LBA: Final[int] = 57344  # 0x01C00000 (28 MB)
MEIHDFS_JOURNAL_START_LBA: Final[int] = 16384  # 0x00800000 (8 MB)
MEIHDFS_PART1_LBA: Final[int] = 65536  # 0x02000000 (32 MB)
MEIHDFS_PART2_LBA: Final[int] = 131072  # 0x04000000 (64 MB)
MEIHDFS_MIN_BLOCK_BYTES: Final[int] = 512

# ==============================================================================
# RAG RETRIEVAL & CRITIC HYPERPARAMETERS
# ==============================================================================

# BM25 Parameters
BM25_K1: Final[float] = 1.5
BM25_B: Final[float] = 0.75

# Reciprocal Rank Fusion Constant
RRF_K: Final[int] = 60

# Minimum relevance score for a chunk to be accepted (CRAG filter)
MIN_RELEVANCE_SCORE: Final[float] = 0.08

# Top-K chunks fed to the critic and generator
DEFAULT_TOP_K: Final[int] = 5
MAX_CHUNKS_PER_PROMPT: Final[int] = 8

# Chunking Configuration
DEFAULT_CHUNK_SIZE_WORDS: Final[int] = 300
DEFAULT_CHUNK_OVERLAP_WORDS: Final[int] = 50

# Epistemic Abstention Threshold
ABSTENTION_SIMILARITY_THRESHOLD: Final[float] = 0.12

# Minimum Groundedness Ratio required to release an answer without warnings
MIN_GROUNDEDNESS_RATIO: Final[float] = 0.95
