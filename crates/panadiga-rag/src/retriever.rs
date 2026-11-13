//! Hybrid retriever combining BM25, TF-IDF vector ranking, and entity-specific boosting.

use std::collections::HashSet;

use super::config::{DEFAULT_TOP_K, RRF_K};
use super::indexer::{HierarchicalIndexer, technical_tokenize};
use super::models::{Query, RetrievalResult};

pub struct HybridRetriever<'a> {
    pub indexer: &'a HierarchicalIndexer,
}

fn dense_ranks(scores: &[f64]) -> Vec<usize> {
    let mut ranked: Vec<usize> = (0..scores.len()).collect();
    ranked.sort_unstable_by(|&a, &b| {
        scores[b]
            .partial_cmp(&scores[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut ranks = vec![usize::MAX; scores.len()];
    for (rank, &doc_idx) in ranked.iter().enumerate() {
        ranks[doc_idx] = rank;
    }
    ranks
}

impl<'a> HybridRetriever<'a> {
    pub fn new(indexer: &'a HierarchicalIndexer) -> Self {
        Self { indexer }
    }

    pub fn retrieve(&self, query: &Query, top_k: usize) -> Vec<RetrievalResult> {
        let chunks = &self.indexer.chunks;
        if chunks.is_empty() {
            return Vec::new();
        }

        let tokens = technical_tokenize(&query.sanitized_text, true);
        if tokens.is_empty() {
            return Vec::new();
        }

        let (bm25_scores, vector_scores) = rayon::join(
            || self.indexer.bm25.score(&tokens),
            || self.indexer.vector.score(&tokens),
        );

        let (bm25_ranks, vector_ranks) =
            rayon::join(|| dense_ranks(&bm25_scores), || dense_ranks(&vector_scores));

        let query_entity_names: HashSet<String> = query
            .extracted_entities
            .iter()
            .map(|e| e.name.to_uppercase())
            .collect();

        let max_rrf_normalizer = 2.0 / (RRF_K as f64 + 1.0);
        let mut results = Vec::new();

        for (idx, chunk) in chunks.iter().enumerate() {
            let lex = bm25_scores[idx];
            let sem = vector_scores[idx];

            if lex <= 0.0 && sem <= 0.0 {
                continue;
            }

            let r_bm = bm25_ranks[idx];
            let r_vec = vector_ranks[idx];

            let bm_comp = if lex > 0.0 {
                1.0 / (RRF_K as f64 + r_bm as f64 + 1.0)
            } else {
                0.0
            };

            let vec_comp = if sem > 0.0 {
                1.0 / (RRF_K as f64 + r_vec as f64 + 1.0)
            } else {
                0.0
            };

            let rrf_score = (bm_comp + vec_comp) / max_rrf_normalizer;

            // Exact entity boost (+0.10 for direct hardware entity match)
            let mut boost = 0.0;
            if !query_entity_names.is_empty() {
                let chunk_upper = chunk.text.to_uppercase();
                let chunk_entities: HashSet<String> = chunk
                    .metadata
                    .extracted_entities
                    .iter()
                    .map(|e| e.name.to_uppercase())
                    .collect();

                for ent_name in &query_entity_names {
                    if chunk_entities.contains(ent_name) || chunk_upper.contains(ent_name) {
                        boost += 0.10;
                    }
                }
            }

            let final_score = (rrf_score + boost).min(1.0);
            let chunk_token_set: HashSet<&String> = chunk.tokens.iter().collect();
            let matched_toks: Vec<String> = tokens
                .iter()
                .filter(|t| chunk_token_set.contains(t))
                .cloned()
                .collect();

            results.push(RetrievalResult {
                chunk: chunk.clone(),
                score: final_score,
                lexical_score: lex,
                semantic_score: sem,
                rrf_score,
                matched_tokens: matched_toks,
            });
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let effective_k = if top_k == 0 { DEFAULT_TOP_K } else { top_k };
        results.truncate(effective_k);
        results
    }
}
