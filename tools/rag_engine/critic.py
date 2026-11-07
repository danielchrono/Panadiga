"""Dual-layer critic: Corrective RAG (CRAG) retrieval critic & Self-RAG fact auditor.
"""

from __future__ import annotations

import re
from typing import List, Set, Tuple

from .config import (
    ABSTENTION_SIMILARITY_THRESHOLD,
    MIN_GROUNDEDNESS_RATIO,
    MIN_RELEVANCE_SCORE,
)
from .guardrails import QueryGuard
from .indexer import STOPWORDS
from .models import Chunk, CriticReport, EntityType, Query, RetrievalResult


class RetrievalCritic:
    """Corrective RAG (CRAG) pre-generation evaluator and noise filter."""

    def __init__(self, threshold: float = ABSTENTION_SIMILARITY_THRESHOLD) -> None:
        self.threshold = threshold

    def evaluate_retrieval(
        self, query: Query, results: List[RetrievalResult]
    ) -> Tuple[bool, List[RetrievalResult], str]:
        """Assesses retrieval quality; triggers epistemic abstention if evidence is insufficient."""
        if not results:
            return (
                False,
                [],
                "No relevant documents found in technical corpus. In embedded reverse engineering, answering without verified documentation is prohibited.",
            )

        top_score = results[0].score

        # If top score is below abstention threshold
        if top_score < self.threshold:
            return (
                False,
                [],
                f"Top document similarity score ({top_score:.4f}) is below confidence threshold ({self.threshold:.4f}). Abstaining to prevent physical hardware damage.",
            )

        # Check content term coverage of the top chunk
        query_words = [
            w for w in re.findall(r"\b[A-Za-z0-9_#/]+\b", query.sanitized_text.lower())
            if w not in STOPWORDS and len(w) > 2
        ]
        if query_words:
            top_matched = set(results[0].matched_tokens)
            coverage = len([w for w in query_words if w in top_matched]) / float(len(query_words))

            has_entity_grounding = any(
                e.name.upper() in results[0].chunk.text.upper()
                for e in query.extracted_entities
                if e.entity_type in (EntityType.HEX_ADDRESS, EntityType.ERROR_CODE, EntityType.PIN)
            )

            if coverage < 0.45 and not has_entity_grounding:
                return (
                    False,
                    [],
                    f"Low technical term coverage ({coverage:.1%}). Core query concepts not found in technical corpus.",
                )

        # Filter noise chunks
        filtered = [r for r in results if r.score >= MIN_RELEVANCE_SCORE]
        if not filtered:
            return (
                False,
                [],
                "All retrieved chunks fell below the minimum relevance filter.",
            )

        return True, filtered, "Retrieval grounded and sufficient."


class FactCritic:
    """Self-RAG post-generation auditor verifying factual grounding against sources."""

    # Non-capturing group for decimals so re.findall returns full voltage string
    VOLTAGE_REGEX = re.compile(r"\b\d+(?:\.\d+)?\s*(?:V|volts?)\b", re.IGNORECASE)
    HEX_REGEX = re.compile(r"\b0x[0-9A-Fa-f]+\b")

    def __init__(self) -> None:
        self.guard = QueryGuard()

    @staticmethod
    def _extract_hex_integers(text: str) -> Set[int]:
        """Extracts hex addresses and converts them to canonical integers.
        
        Prevents format disparity exploits (e.g. 0x007F0000 vs 0x7F0000) and
        stops substring false positives (e.g. 0x7F00 vs 0x7F0000).
        """
        matches = FactCritic.HEX_REGEX.findall(text)
        ints: Set[int] = set()
        for m in matches:
            try:
                ints.add(int(m, 16))
            except ValueError:
                pass
        return ints

    @staticmethod
    def _extract_voltages(text: str) -> Set[float]:
        """Extracts voltage numbers as floating point values."""
        matches = FactCritic.VOLTAGE_REGEX.findall(text)
        volts: Set[float] = set()
        for m in matches:
            num = re.search(r"\d+(?:\.\d+)?", m)
            if num:
                try:
                    volts.add(float(num.group(0)))
                except ValueError:
                    pass
        return volts

    def audit(self, generated_text: str, source_chunks: List[Chunk]) -> CriticReport:
        """Audits generated technical entities against retrieved source chunks."""
        combined_source_text = "\n".join(c.text for c in source_chunks)

        # 1. Audit Hex Addresses via Canonical Integers
        gen_hex_ints = self._extract_hex_integers(generated_text)
        src_hex_ints = self._extract_hex_integers(combined_source_text)

        verified_entities: List[str] = []
        ungrounded_entities: List[str] = []

        for h_int in gen_hex_ints:
            h_str = hex(h_int).upper()
            if h_int in src_hex_ints:
                verified_entities.append(f"HEX:{h_str}")
            else:
                ungrounded_entities.append(f"HEX:{h_str}")

        # 2. Audit Voltages
        gen_volts = self._extract_voltages(generated_text)
        src_volts = self._extract_voltages(combined_source_text)

        for v in gen_volts:
            v_str = f"{v}V"
            if v in src_volts:
                verified_entities.append(f"VOLTAGE:{v_str}")
            else:
                ungrounded_entities.append(f"VOLTAGE:{v_str}")

        # 3. Audit Components & Pins
        upper_gen = generated_text.upper()
        upper_src = combined_source_text.upper()
        gen_words = set(re.findall(r"\b[A-Z0-9_#/]+(?<![.,:;?!])", upper_gen))

        for comp in self.guard.KNOWN_COMPONENTS:
            if comp in gen_words or comp in upper_gen:
                if comp in upper_src:
                    verified_entities.append(f"COMPONENT:{comp}")
                else:
                    ungrounded_entities.append(f"COMPONENT:{comp}")

        for pin in self.guard.KNOWN_PINS_AND_SIGNALS:
            if pin in gen_words:
                if pin in upper_src:
                    verified_entities.append(f"PIN:{pin}")
                else:
                    ungrounded_entities.append(f"PIN:{pin}")

        total_entities = len(verified_entities) + len(ungrounded_entities)
        if total_entities == 0:
            groundedness_score = 1.0
        else:
            groundedness_score = len(verified_entities) / float(total_entities)

        is_faithful = (
            groundedness_score >= MIN_GROUNDEDNESS_RATIO and len(ungrounded_entities) == 0
        )

        notes: List[str] = []
        if ungrounded_entities:
            notes.append(
                f"Generated text contains ungrounded technical entities: {', '.join(ungrounded_entities)}"
            )

        return CriticReport(
            is_faithful=is_faithful,
            groundedness_score=groundedness_score,
            verified_entities=verified_entities,
            ungrounded_entities=ungrounded_entities,
            notes=notes,
        )
