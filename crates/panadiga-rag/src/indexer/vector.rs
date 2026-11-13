//! Sublinear TF-IDF cosine vector similarity index over technical corpus chunks.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct VectorIndex {
    pub vocab: HashMap<String, usize>,
    pub idf: HashMap<String, f64>,
    pub doc_vectors: Vec<HashMap<usize, f64>>,
    pub doc_norms: Vec<f64>,
}

impl VectorIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fit(&mut self, tokenized_docs: &[Vec<String>]) {
        let corpus_size = tokenized_docs.len();
        if corpus_size == 0 {
            return;
        }

        let mut doc_freqs: HashMap<String, usize> = HashMap::new();
        for doc in tokenized_docs {
            let unique: HashSet<&String> = doc.iter().collect();
            for term in unique {
                *doc_freqs.entry(term.clone()).or_insert(0) += 1;
            }
        }

        let mut terms: Vec<String> = doc_freqs.keys().cloned().collect();
        terms.sort();

        self.vocab = terms.into_iter().enumerate().map(|(i, t)| (t, i)).collect();
        self.idf = doc_freqs
            .iter()
            .map(|(t, &f)| {
                let idf_val = (1.0 + (corpus_size as f64 / (f as f64 + 1.0))).ln();
                (t.clone(), idf_val)
            })
            .collect();

        self.doc_vectors.clear();
        self.doc_norms.clear();

        for doc in tokenized_docs {
            let mut counts = HashMap::new();
            for t in doc {
                *counts.entry(t).or_insert(0) += 1;
            }

            let mut vec = HashMap::new();
            let mut norm_sq = 0.0;

            for (term, count) in counts {
                if let Some(&term_id) = self.vocab.get(term) {
                    let weight =
                        (1.0 + (count as f64).ln()) * self.idf.get(term).copied().unwrap_or(1.0);
                    vec.insert(term_id, weight);
                    norm_sq += weight * weight;
                }
            }

            let norm = if norm_sq > 0.0 { norm_sq.sqrt() } else { 1.0 };
            self.doc_vectors.push(vec);
            self.doc_norms.push(norm);
        }
    }

    pub fn score(&self, query_tokens: &[String]) -> Vec<f64> {
        let corpus_size = self.doc_vectors.len();
        if corpus_size == 0 || query_tokens.is_empty() {
            return vec![0.0; corpus_size];
        }

        let mut q_counts = HashMap::new();
        for t in query_tokens {
            *q_counts.entry(t).or_insert(0) += 1;
        }

        let mut q_vec = HashMap::new();
        let mut q_norm_sq = 0.0;

        for (term, count) in q_counts {
            if let Some(&term_id) = self.vocab.get(term) {
                let w = (1.0 + (count as f64).ln()) * self.idf.get(term).copied().unwrap_or(1.0);
                q_vec.insert(term_id, w);
                q_norm_sq += w * w;
            }
        }

        let q_norm = if q_norm_sq > 0.0 {
            q_norm_sq.sqrt()
        } else {
            1.0
        };

        let mut scores = vec![0.0; corpus_size];
        for (idx, d_vec) in self.doc_vectors.iter().enumerate() {
            let mut dot_product = 0.0;
            for (&t_id, &w) in &q_vec {
                if let Some(&d_w) = d_vec.get(&t_id) {
                    dot_product += w * d_w;
                }
            }
            let sim = dot_product / (q_norm * self.doc_norms[idx]);
            scores[idx] = sim.max(0.0);
        }

        scores
    }
}
