"""Data models for the Zero-Error RAG Engine.

Designed specifically for mission-critical embedded systems and hardware
reverse engineering where hallucinations can cause physical hardware damage.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional


class SubsystemType(str, Enum):
    HARDWARE = "hardware"
    FIRMWARE = "firmware"
    STORAGE = "storage"
    CRYPTO = "crypto"
    REVERSE_ENGINEERING = "reverse_engineering"
    GENERAL = "general"


class EntityType(str, Enum):
    HEX_ADDRESS = "hex_address"
    COMPONENT = "component"
    PIN = "pin"
    VOLTAGE = "voltage"
    ERROR_CODE = "error_code"
    LBA_SECTOR = "lba_sector"
    MAGIC_SIGNATURE = "magic_signature"


@dataclass(frozen=True)
class Entity:
    name: str
    entity_type: EntityType
    raw_value: str


@dataclass
class ChunkMetadata:
    doc_title: str
    file_path: str
    section_title: str
    hierarchy: List[str] = field(default_factory=list)
    line_start: int = 0
    line_end: int = 0
    subsystem: SubsystemType = SubsystemType.GENERAL
    extracted_entities: List[Entity] = field(default_factory=list)

    def to_citation_label(self) -> str:
        sec = self.section_title or "General"
        return f"{self.doc_title} ({sec})"


@dataclass
class Chunk:
    id: str
    text: str
    metadata: ChunkMetadata
    tokens: List[str] = field(default_factory=list)

    @property
    def full_reference(self) -> str:
        return f"[{self.metadata.doc_title} > {self.metadata.section_title} | Lines {self.metadata.line_start}-{self.metadata.line_end}]"


@dataclass
class Query:
    raw_text: str
    sanitized_text: str
    intent: str = "general"
    extracted_entities: List[Entity] = field(default_factory=list)
    is_high_risk: bool = False
    is_injection: bool = False
    validation_error: Optional[str] = None


@dataclass
class RetrievalResult:
    chunk: Chunk
    score: float
    lexical_score: float = 0.0
    semantic_score: float = 0.0
    rrf_score: float = 0.0
    matched_tokens: List[str] = field(default_factory=list)


@dataclass
class CriticReport:
    is_faithful: bool
    groundedness_score: float
    verified_entities: List[str] = field(default_factory=list)
    ungrounded_entities: List[str] = field(default_factory=list)
    contradicted_facts: List[str] = field(default_factory=list)
    confidence: float = 1.0
    notes: List[str] = field(default_factory=list)


@dataclass
class SafetyReport:
    is_safe: bool
    violations: List[str] = field(default_factory=list)
    critical_alerts: List[str] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)


@dataclass
class Citation:
    doc_title: str
    file_path: str
    section_title: str
    line_range: str
    excerpt: str


@dataclass
class RagAnswer:
    query: str
    answer_text: str
    citations: List[Citation] = field(default_factory=list)
    confidence: float = 1.0
    groundedness_score: float = 1.0
    critic_report: Optional[CriticReport] = None
    safety_report: Optional[SafetyReport] = None
    retrieved_chunks_count: int = 0
    execution_time_ms: float = 0.0
    is_abstained: bool = False
    abstention_reason: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "query": self.query,
            "answer": self.answer_text,
            "confidence": round(self.confidence, 4),
            "groundedness_score": round(self.groundedness_score, 4),
            "is_abstained": self.is_abstained,
            "abstention_reason": self.abstention_reason,
            "citations": [
                {
                    "doc": c.doc_title,
                    "section": c.section_title,
                    "lines": c.line_range,
                    "excerpt": c.excerpt,
                }
                for c in self.citations
            ],
            "safety": {
                "is_safe": self.safety_report.is_safe if self.safety_report else True,
                "violations": self.safety_report.violations if self.safety_report else [],
                "alerts": self.safety_report.critical_alerts if self.safety_report else [],
            },
            "critic": {
                "is_faithful": self.critic_report.is_faithful if self.critic_report else True,
                "ungrounded_entities": self.critic_report.ungrounded_entities if self.critic_report else [],
            },
            "retrieved_chunks_count": self.retrieved_chunks_count,
            "execution_time_ms": round(self.execution_time_ms, 2),
        }
