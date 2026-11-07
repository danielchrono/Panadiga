"""Panasonic DIGA DMR-EH55 Zero-Error RAG Engine.

Mission-critical hardware safety shield and grounded retrieval-augmented generation.
"""

from .config import (
    FORBIDDEN_FLASH_WRITE_END,
    FORBIDDEN_FLASH_WRITE_START,
    FORBIDDEN_JTAG_PINS,
    MAX_SAFE_VOLTAGE_VOLTS,
    SAFE_NVRAM_BASE_OFFSET,
    SAFE_NVRAM_END_OFFSET,
)
from .critic import FactCritic, RetrievalCritic
from .generator import GroundedGenerator
from .guardrails import QueryGuard
from .indexer import HierarchicalIndexer
from .models import (
    Chunk,
    ChunkMetadata,
    Citation,
    CriticReport,
    Entity,
    EntityType,
    Query,
    RagAnswer,
    RetrievalResult,
    SafetyReport,
    SubsystemType,
)
from .pipeline import ZeroErrorRagEngine
from .retriever import HybridRetriever
from .safety_invariants import HardwareSafetyValidator

__all__ = [
    "ZeroErrorRagEngine",
    "HardwareSafetyValidator",
    "HierarchicalIndexer",
    "HybridRetriever",
    "RetrievalCritic",
    "FactCritic",
    "QueryGuard",
    "GroundedGenerator",
    "RagAnswer",
    "Query",
    "Chunk",
    "ChunkMetadata",
    "Citation",
    "CriticReport",
    "SafetyReport",
    "Entity",
    "EntityType",
    "SubsystemType",
    "RetrievalResult",
    "MAX_SAFE_VOLTAGE_VOLTS",
    "FORBIDDEN_FLASH_WRITE_START",
    "FORBIDDEN_FLASH_WRITE_END",
    "SAFE_NVRAM_BASE_OFFSET",
    "SAFE_NVRAM_END_OFFSET",
    "FORBIDDEN_JTAG_PINS",
]
