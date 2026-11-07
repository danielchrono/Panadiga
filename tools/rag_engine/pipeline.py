"""Main Zero-Error RAG Engine pipeline orchestrating all 6 architectural defense layers.
"""

from __future__ import annotations

import time
from typing import List, Optional

from .critic import FactCritic, RetrievalCritic
from .generator import GroundedGenerator
from .guardrails import QueryGuard
from .indexer import HierarchicalIndexer
from .models import RagAnswer, SafetyReport
from .retriever import HybridRetriever
from .safety_invariants import HardwareSafetyValidator


class ZeroErrorRagEngine:
    """Enterprise RAG pipeline with dual-layer critics and hardware safety shield."""

    def __init__(self, indexer: Optional[HierarchicalIndexer] = None) -> None:
        self.guard = QueryGuard()
        self.indexer = indexer or HierarchicalIndexer()
        self.retriever = HybridRetriever(self.indexer)
        self.retrieval_critic = RetrievalCritic()
        self.generator = GroundedGenerator()
        self.fact_critic = FactCritic()
        self.safety_validator = HardwareSafetyValidator()

    def initialize(self) -> int:
        """Indexes default documentation and schematics."""
        return self.indexer.build_index_from_directories()

    def query(self, raw_query: str) -> RagAnswer:
        """Executes the full 6-layer defense pipeline."""
        t0 = time.perf_counter()

        # ----------------------------------------------------------------------
        # Layer 1: QueryGuard (Input Validation & Injection Detection)
        # ----------------------------------------------------------------------
        q = self.guard.process(raw_query)
        if q.is_injection:
            elapsed = (time.perf_counter() - t0) * 1000.0
            return RagAnswer(
                query=raw_query,
                answer_text="[SECURITY ALERT] Prompt injection or safety override attempt detected. Request rejected.",
                confidence=0.0,
                groundedness_score=0.0,
                safety_report=SafetyReport(
                    is_safe=False,
                    violations=[q.validation_error or "Prompt injection detected."],
                    critical_alerts=["Adversarial input blocked at Layer 1."],
                ),
                is_abstained=True,
                abstention_reason="Prompt injection detected.",
                execution_time_ms=elapsed,
            )

        # ----------------------------------------------------------------------
        # Pre-Execution Safety Shield on Query Itself
        # ----------------------------------------------------------------------
        query_safety = self.safety_validator.validate(q.sanitized_text, is_query=True)
        if not query_safety.is_safe:
            elapsed = (time.perf_counter() - t0) * 1000.0
            return RagAnswer(
                query=raw_query,
                answer_text=(
                    "[CRITICAL AIR-GAP INTERCEPTION] Request blocked by the hardware safety shield to prevent physical equipment damage:\n\n"
                    + "\n".join(f"• {v}" for v in query_safety.violations)
                ),
                confidence=0.0,
                groundedness_score=0.0,
                safety_report=query_safety,
                is_abstained=True,
                abstention_reason="Physical hardware invariant violation detected in query.",
                execution_time_ms=elapsed,
            )

        # ----------------------------------------------------------------------
        # Layer 2: Hybrid Retrieval (BM25 + TF-IDF Vector + Entity Boost)
        # ----------------------------------------------------------------------
        retrieval_results = self.retriever.retrieve(q)

        # ----------------------------------------------------------------------
        # Layer 3: RetrievalCritic (Corrective RAG - CRAG)
        # ----------------------------------------------------------------------
        is_grounded, valid_results, reason = self.retrieval_critic.evaluate_retrieval(
            q, retrieval_results
        )

        if not is_grounded:
            elapsed = (time.perf_counter() - t0) * 1000.0
            return RagAnswer(
                query=raw_query,
                answer_text=(
                    "Information not verified in technical corpus. In proprietary embedded systems, "
                    "ungrounded inferences cause physical hardware destruction."
                ),
                confidence=0.0,
                groundedness_score=0.0,
                retrieved_chunks_count=len(retrieval_results),
                is_abstained=True,
                abstention_reason=reason,
                execution_time_ms=elapsed,
            )

        # ----------------------------------------------------------------------
        # Layer 4: Grounded Synthesis
        # ----------------------------------------------------------------------
        answer_text, citations = self.generator.synthesize_deterministic(q, valid_results)

        # ----------------------------------------------------------------------
        # Layer 5: FactCritic (Self-RAG Post-Generation Entity Grounding)
        # ----------------------------------------------------------------------
        critic_report = self.fact_critic.audit(
            answer_text, [r.chunk for r in valid_results]
        )

        # ----------------------------------------------------------------------
        # Layer 6: Hardware Safety Shield (Deterministic Post-Generation Veto)
        # ----------------------------------------------------------------------
        post_safety = self.safety_validator.validate(answer_text, is_query=False)

        is_abstained = False
        abstention_reason = None

        # OPUS-08 FIX: When FactCritic detects hallucination, force epistemic abstention
        if not critic_report.is_faithful:
            is_abstained = True
            abstention_reason = (
                f"Self-RAG FactCritic detected ungrounded technical entities: "
                f"{', '.join(critic_report.ungrounded_entities)}"
            )
            answer_text = (
                "[EPISTEMIC ABSTENTION - UNGROUNDED ENTITY DETECTED]\n\n"
                "The response was intercepted because the following technical entities were not grounded in the verified corpus:\n"
                + "\n".join(f"• {e}" for e in critic_report.ungrounded_entities)
                + "\n\nIn embedded hardware reverse engineering, answering with unverified parameters is strictly prohibited to prevent equipment damage."
            )

        if not post_safety.is_safe:
            is_abstained = True
            abstention_reason = "Generated recommendations violated physical hardware safety invariants."
            answer_text = (
                "[CRITICAL AIR-GAP INTERCEPTION] Generated recommendations were blocked due to safety violations:\n\n"
                + "\n".join(f"• {v}" for v in post_safety.violations)
            )

        elapsed = (time.perf_counter() - t0) * 1000.0
        return RagAnswer(
            query=raw_query,
            answer_text=answer_text,
            citations=citations,
            confidence=1.0 if (critic_report.is_faithful and post_safety.is_safe and not is_abstained) else 0.0,
            groundedness_score=critic_report.groundedness_score,
            critic_report=critic_report,
            safety_report=post_safety,
            retrieved_chunks_count=len(valid_results),
            execution_time_ms=elapsed,
            is_abstained=is_abstained,
            abstention_reason=abstention_reason,
        )
