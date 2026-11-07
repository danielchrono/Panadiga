"""Hybrid retriever combining BM25, TF-IDF vector ranking, and entity-specific boosting.
"""

from __future__ import annotations

from typing import List

from .config import DEFAULT_TOP_K, RRF_K
from .indexer import HierarchicalIndexer, technical_tokenize
from .models import Query, RetrievalResult


class HybridRetriever:
    """Performs reciprocal rank fusion across lexical, semantic, and domain entity dimensions."""

    def __init__(self, indexer: HierarchicalIndexer) -> None:
        self.indexer = indexer

    def retrieve(self, query: Query, top_k: int = DEFAULT_TOP_K) -> List[RetrievalResult]:
        chunks = self.indexer.chunks
        if not chunks:
            return []

        tokens = technical_tokenize(query.sanitized_text)
        if not tokens:
            return []

        bm25_scores = self.indexer.bm25.score(tokens)
        vector_scores = self.indexer.vector.score(tokens)

        # Rank indices descending
        bm25_ranked = sorted(range(len(chunks)), key=lambda i: bm25_scores[i], reverse=True)
        vector_ranked = sorted(range(len(chunks)), key=lambda i: vector_scores[i], reverse=True)

        bm25_ranks = {doc_idx: rank for rank, doc_idx in enumerate(bm25_ranked)}
        vector_ranks = {doc_idx: rank for rank, doc_idx in enumerate(vector_ranked)}

        results: List[RetrievalResult] = []

        # Exact entity matches to boost
        query_entity_names = {e.name.upper() for e in query.extracted_entities}
        max_rrf_normalizer = 2.0 / (RRF_K + 1)

        for idx, chunk in enumerate(chunks):
            lex = bm25_scores[idx]
            sem = vector_scores[idx]

            if lex <= 0.0 and sem <= 0.0:
                continue

            r_bm = bm25_ranks[idx]
            r_vec = vector_ranks[idx]

            bm_comp = (1.0 / (RRF_K + r_bm + 1)) if lex > 0.0 else 0.0
            vec_comp = (1.0 / (RRF_K + r_vec + 1)) if sem > 0.0 else 0.0
            rrf_score = (bm_comp + vec_comp) / max_rrf_normalizer

            # Exact entity boost (+0.10 for direct hardware entity match)
            boost = 0.0
            chunk_upper = chunk.text.upper()
            chunk_entities = {e.name.upper() for e in chunk.metadata.extracted_entities}

            for ent_name in query_entity_names:
                if ent_name in chunk_entities or ent_name in chunk_upper:
                    boost += 0.10

            final_score = rrf_score + boost
            matched_toks = [t for t in tokens if t in chunk.tokens]

            results.append(
                RetrievalResult(
                    chunk=chunk,
                    score=final_score,
                    lexical_score=lex,
                    semantic_score=sem,
                    rrf_score=rrf_score,
                    matched_tokens=matched_toks,
                )
            )

        results.sort(key=lambda r: r.score, reverse=True)
        return results[:top_k]
