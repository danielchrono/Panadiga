"""Hierarchical markdown chunker and hybrid indexer (BM25 + TF-IDF Vector Index).
"""

from __future__ import annotations

import math
import re
from collections import Counter, defaultdict
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

from .config import (
    BM25_B,
    BM25_K1,
    DEFAULT_CHUNK_OVERLAP_WORDS,
    DEFAULT_CHUNK_SIZE_WORDS,
    DOCS_DIR,
    MANUALS_DIR,
)
from .guardrails import QueryGuard
from .models import Chunk, ChunkMetadata, SubsystemType


# Stopwords for filtering noise tokens in technical queries and indexing
STOPWORDS: Set[str] = {
    "a", "about", "above", "after", "again", "against", "all", "am", "an", "and", "any", "are",
    "as", "at", "be", "because", "been", "before", "being", "below", "between", "both", "but", "by",
    "can", "cannot", "could", "did", "do", "does", "doing", "down", "during", "each", "few", "for",
    "from", "further", "had", "has", "have", "having", "he", "her", "here", "hers", "him", "his",
    "how", "i", "if", "in", "into", "is", "it", "its", "itself", "me", "more", "most", "my", "myself",
    "no", "nor", "not", "of", "off", "on", "once", "only", "or", "other", "our", "ours", "out", "over",
    "own", "same", "she", "should", "so", "some", "such", "than", "that", "the", "their", "theirs",
    "them", "then", "there", "these", "they", "this", "those", "through", "to", "too", "under",
    "until", "up", "very", "was", "we", "were", "what", "when", "where", "which", "while", "who",
    "whom", "why", "with", "would", "you", "your", "yours",
    "da", "de", "do", "das", "dos", "em", "no", "na", "nos", "nas", "para", "por", "com", "um", "uma"
}


def technical_tokenize(text: str, filter_stopwords: bool = True) -> List[str]:
    """Tokenizes text preserving hex values, IC numbers, pins, and error codes."""
    raw_tokens = re.findall(r"\b0x[0-9A-Fa-f]+\b|[A-Za-z0-9_#/]+(?<![.,:;?!])", text)
    tokens: List[str] = []
    for tok in raw_tokens:
        tok_clean = tok.strip(".,;:()[]{}'\"")
        if not tok_clean:
            continue
        # Preserve hex as canonical uppercase
        if tok_clean.startswith("0x") or tok_clean.startswith("0X"):
            tokens.append(tok_clean.upper())
        else:
            lower_tok = tok_clean.lower()
            if filter_stopwords and lower_tok in STOPWORDS:
                continue
            tokens.append(lower_tok)
    return tokens


class BM25Index:
    """BM25 Okapi inverted index with exact numeric & identifier retention."""

    def __init__(self, k1: float = BM25_K1, b: float = BM25_B) -> None:
        self.k1 = k1
        self.b = b
        self.corpus_size = 0
        self.avg_doc_len = 0.0
        self.doc_lens: List[int] = []
        self.doc_freqs: Dict[str, int] = defaultdict(int)
        self.term_freqs: List[Dict[str, int]] = []
        self.idf: Dict[str, float] = {}

    def fit(self, tokenized_docs: List[List[str]]) -> None:
        self.corpus_size = len(tokenized_docs)
        if self.corpus_size == 0:
            return

        total_len = 0
        self.doc_lens = []
        self.term_freqs = []
        self.doc_freqs.clear()

        for doc in tokenized_docs:
            d_len = len(doc)
            self.doc_lens.append(d_len)
            total_len += d_len

            counts = Counter(doc)
            self.term_freqs.append(counts)
            for term in counts.keys():
                self.doc_freqs[term] += 1

        self.avg_doc_len = total_len / float(self.corpus_size)

        # Precompute IDF (Okapi)
        self.idf.clear()
        for term, freq in self.doc_freqs.items():
            # Standard BM25 IDF with smoothing
            val = math.log((self.corpus_size - freq + 0.5) / (freq + 0.5) + 1.0)
            self.idf[term] = max(val, 0.01)

    def score(self, query_tokens: List[str]) -> List[float]:
        if self.corpus_size == 0 or not query_tokens:
            return [0.0] * self.corpus_size

        scores = [0.0] * self.corpus_size
        for term in query_tokens:
            if term not in self.idf:
                continue
            idf_val = self.idf[term]

            for idx in range(self.corpus_size):
                tf = self.term_freqs[idx].get(term, 0)
                if tf == 0:
                    continue
                d_len = self.doc_lens[idx]
                denom = tf + self.k1 * (1.0 - self.b + self.b * (d_len / self.avg_doc_len))
                term_score = idf_val * ((tf * (self.k1 + 1.0)) / denom)
                scores[idx] += term_score

        return scores


class VectorIndex:
    """Normalized sublinear TF-IDF vector index for semantic retrieval."""

    def __init__(self) -> None:
        self.vocab: Dict[str, int] = {}
        self.idf: Dict[str, float] = {}
        self.doc_vectors: List[Dict[int, float]] = []
        self.doc_norms: List[float] = []

    def fit(self, tokenized_docs: List[List[str]]) -> None:
        corpus_size = len(tokenized_docs)
        if corpus_size == 0:
            return

        doc_freqs: Dict[str, int] = defaultdict(int)
        for doc in tokenized_docs:
            unique_terms = set(doc)
            for term in unique_terms:
                doc_freqs[term] += 1

        # Build vocabulary
        self.vocab = {term: idx for idx, term in enumerate(sorted(doc_freqs.keys()))}
        self.idf = {
            term: math.log(1.0 + (corpus_size / (freq + 1.0)))
            for term, freq in doc_freqs.items()
        }

        self.doc_vectors = []
        self.doc_norms = []

        for doc in tokenized_docs:
            counts = Counter(doc)
            vec: Dict[int, float] = {}
            norm_sq = 0.0

            for term, count in counts.items():
                if term in self.vocab:
                    term_id = self.vocab[term]
                    weight = (1.0 + math.log(count)) * self.idf[term]
                    vec[term_id] = weight
                    norm_sq += weight * weight

            norm = math.sqrt(norm_sq) if norm_sq > 0.0 else 1.0
            self.doc_vectors.append(vec)
            self.doc_norms.append(norm)

    def score(self, query_tokens: List[str]) -> List[float]:
        corpus_size = len(self.doc_vectors)
        if corpus_size == 0 or not query_tokens:
            return [0.0] * corpus_size

        q_counts = Counter(query_tokens)
        q_vec: Dict[int, float] = {}
        q_norm_sq = 0.0

        for term, count in q_counts.items():
            if term in self.vocab:
                term_id = self.vocab[term]
                w = (1.0 + math.log(count)) * self.idf[term]
                q_vec[term_id] = w
                q_norm_sq += w * w

        q_norm = math.sqrt(q_norm_sq) if q_norm_sq > 0.0 else 1.0
        if q_norm == 0.0:
            return [0.0] * corpus_size

        scores = [0.0] * corpus_size
        for idx, d_vec in enumerate(self.doc_vectors):
            dot_product = sum(w * d_vec.get(t_id, 0.0) for t_id, w in q_vec.items())
            sim = dot_product / (q_norm * self.doc_norms[idx])
            scores[idx] = max(0.0, sim)

        return scores


class HierarchicalIndexer:
    """Parses engineering documents into hierarchical structured chunks and builds indexes."""

    def __init__(self) -> None:
        self.guard = QueryGuard()
        self.chunks: List[Chunk] = []
        self.bm25 = BM25Index()
        self.vector = VectorIndex()

    def determine_subsystem(self, file_path: str, content: str) -> SubsystemType:
        fp_lower = file_path.lower()
        content_lower = content.lower()
        if "dwire" in fp_lower or "hardware" in fp_lower or "esquema" in fp_lower:
            return SubsystemType.HARDWARE
        if "meihdfs" in fp_lower or "hdd" in fp_lower or "alocacao" in fp_lower:
            return SubsystemType.STORAGE
        if "frm" in fp_lower or "crypto" in fp_lower or "aes" in fp_lower:
            return SubsystemType.CRYPTO
        if "ghidra" in fp_lower or "gdb" in fp_lower or "reverse" in fp_lower:
            return SubsystemType.REVERSE_ENGINEERING
        if "boot" in content_lower or "ipl" in content_lower or "flash" in content_lower:
            return SubsystemType.FIRMWARE
        return SubsystemType.GENERAL

    def chunk_markdown_file(self, file_path: Path) -> List[Chunk]:
        """Parses a markdown file preserving section headers and line ranges."""
        try:
            content = file_path.read_text(encoding="utf-8", errors="replace")
        except Exception:
            return []

        lines = content.splitlines()
        chunks: List[Chunk] = []
        doc_title = file_path.stem.replace("_", " ").title()

        current_hierarchy: List[str] = [doc_title]
        current_section = "Introduction"
        current_lines: List[Tuple[int, str]] = []

        subsystem = self.determine_subsystem(str(file_path), content)

        def flush_chunk(end_line: int) -> None:
            nonlocal current_lines, current_section, current_hierarchy
            if not current_lines:
                return

            text_block = "\n".join(l[1] for l in current_lines).strip()
            if len(text_block) < 30:
                current_lines = []
                return

            start_line = current_lines[0][0]
            cid = f"{file_path.stem}::L{start_line}-L{end_line}"

            meta_query = self.guard.process(text_block)

            meta = ChunkMetadata(
                doc_title=doc_title,
                file_path=str(file_path),
                section_title=current_section,
                hierarchy=list(current_hierarchy),
                line_start=start_line,
                line_end=end_line,
                subsystem=subsystem,
                extracted_entities=meta_query.extracted_entities,
            )

            tokens = technical_tokenize(text_block)
            chunks.append(Chunk(id=cid, text=text_block, metadata=meta, tokens=tokens))
            current_lines = []

        for line_idx, line in enumerate(lines, start=1):
            stripped = line.strip()
            if stripped.startswith("#"):
                # Header encountered - flush previous chunk
                if current_lines:
                    flush_chunk(line_idx - 1)

                level = len(stripped) - len(stripped.lstrip("#"))
                heading_text = stripped.lstrip("#").strip()
                current_section = heading_text

                # Update hierarchy
                if level == 1:
                    current_hierarchy = [heading_text]
                elif level == 2:
                    current_hierarchy = [current_hierarchy[0] if current_hierarchy else doc_title, heading_text]
                else:
                    if len(current_hierarchy) >= 2:
                        current_hierarchy = [current_hierarchy[0], current_hierarchy[1], heading_text]
                    else:
                        current_hierarchy.append(heading_text)

                current_lines.append((line_idx, line))
            else:
                current_lines.append((line_idx, line))
                # If block gets large, flush chunk
                if len(current_lines) >= 80:
                    flush_chunk(line_idx)

        if current_lines:
            flush_chunk(len(lines))

        return chunks

    def build_index_from_directories(self, directories: Optional[List[Path]] = None) -> int:
        """Scans directories, chunks markdown and text files, and builds indexes.
        
        Excludes meta-documentation (AI guidelines and test dossiers) to prevent
        hallucination examples and audit text from polluting the engineering corpus.
        """
        if directories is None:
            directories = [DOCS_DIR, MANUALS_DIR]

        all_chunks: List[Chunk] = []
        for d in directories:
            if not d.exists():
                continue
            for p in d.rglob("*.md"):
                if p.is_file() and not p.name.startswith("AI_GUIDELINES") and not p.name.startswith("AUDIT_DOSSIER"):
                    all_chunks.extend(self.chunk_markdown_file(p))
            for p in d.rglob("*.txt"):
                if p.is_file():
                    all_chunks.extend(self.chunk_markdown_file(p))

        self.chunks = all_chunks
        tokenized_corpus = [c.tokens for c in self.chunks]

        self.bm25.fit(tokenized_corpus)
        self.vector.fit(tokenized_corpus)

        return len(self.chunks)

    def add_custom_chunks(self, chunks: List[Chunk]) -> None:
        """Adds custom chunks directly (e.g. for synthetic fixtures or testing)."""
        self.chunks.extend(chunks)
        tokenized_corpus = [c.tokens for c in self.chunks]
        self.bm25.fit(tokenized_corpus)
        self.vector.fit(tokenized_corpus)
