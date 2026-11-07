"""Grounded synthesis generator enforcing zero-extrapolation and mandatory citations.
"""

from __future__ import annotations

from typing import List, Tuple

from .config import MAX_CHUNKS_PER_PROMPT
from .models import Citation, Query, RetrievalResult


class GroundedGenerator:
    """Produces technical responses strictly grounded in retrieved source chunks."""

    def __init__(self) -> None:
        pass

    def build_grounded_prompt(self, query: Query, results: List[RetrievalResult]) -> str:
        """Constructs a hermetic prompt requiring bracketed citations and forbidding extrapolation."""
        context_blocks: List[str] = []
        for idx, res in enumerate(results[:MAX_CHUNKS_PER_PROMPT], start=1):
            c = res.chunk
            m = c.metadata
            label = f"[Doc {idx}: {m.doc_title} > {m.section_title} (Lines {m.line_start}-{m.line_end})]"
            context_blocks.append(f"{label}\n{c.text}\n")

        joined_context = "\n---\n".join(context_blocks)

        prompt = (
            "You are the Lead Embedded Systems and Reverse Engineering Specialist for the Panasonic DIGA DMR-EH55.\n"
            "MANDATORY RULES:\n"
            "1. Answer ONLY based on the verified technical context below. Do not extrapolate, guess, or infer from standard PC conventions.\n"
            "2. If the answer is not in the context, explicitly state: 'Information not verified in technical corpus.'\n"
            "3. Every factual claim must include a bracketed citation in the form [DocTitle, SectionTitle].\n"
            "4. NEVER recommend voltages above 3.3V, flash operations in SA0..SA133, or JTAG TDI/TDO on CN55001.\n\n"
            f"TECHNICAL CONTEXT:\n{joined_context}\n\n"
            f"ENGINEER QUERY: {query.sanitized_text}\n"
            "GROUNDED ANSWER:"
        )
        return prompt

    SAFETY_FOOTER = (
        "\n\n[Hardware Safety Note: All operations must comply with 3.3V maximum nominal tolerance "
        "and preserve flash IPL sectors SA0..SA133]."
    )

    @classmethod
    def strip_boilerplate(cls, text: str, query: Query) -> str:
        """Removes the deterministic header and safety footer prior to entity/safety auditing."""
        header = f"Verified technical diagnosis for '{query.sanitized_text}':\n\n"
        cleaned = text
        if cleaned.startswith(header):
            cleaned = cleaned[len(header):]
        if cleaned.endswith(cls.SAFETY_FOOTER):
            cleaned = cleaned[:-len(cls.SAFETY_FOOTER)]
        return cleaned

    def synthesize_deterministic(
        self, query: Query, results: List[RetrievalResult]
    ) -> Tuple[str, List[Citation]]:
        """Synthesizes a deterministic factual response grounded directly in retrieved chunks."""
        if not results:
            return (
                "Information not verified in technical corpus. In proprietary embedded systems, ungrounded inferences cause physical hardware destruction.",
                [],
            )

        citations: List[Citation] = []
        extracted_facts: List[str] = []

        seen_sections = set()
        for res in results[:MAX_CHUNKS_PER_PROMPT]:
            chunk = res.chunk
            meta = chunk.metadata
            sec_key = (meta.doc_title, meta.section_title)
            if sec_key in seen_sections:
                continue
            seen_sections.add(sec_key)

            # Extract first relevant sentences
            lines = [l.strip() for l in chunk.text.splitlines() if l.strip() and not l.strip().startswith("#")]
            if not lines:
                continue

            excerpt = " ".join(lines[:3])
            citation_label = f"[{meta.doc_title}, {meta.section_title}]"

            extracted_facts.append(f"• According to verified documentation {citation_label}: {excerpt}")
            citations.append(
                Citation(
                    doc_title=meta.doc_title,
                    file_path=meta.file_path,
                    section_title=meta.section_title,
                    line_range=f"L{meta.line_start}-L{meta.line_end}",
                    excerpt=excerpt[:150] + "..." if len(excerpt) > 150 else excerpt,
                )
            )

        header = f"Verified technical diagnosis for '{query.sanitized_text}':\n\n"
        body = "\n\n".join(extracted_facts)
        return header + body + self.SAFETY_FOOTER, citations
