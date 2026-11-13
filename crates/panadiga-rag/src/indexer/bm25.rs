//! Okapi BM25 lexical scoring index over tokenized technical chunks.

use std::collections::HashMap;

use super::super::config::{BM25_B, BM25_K1};

#[derive(Debug, Clone)]
pub struct BM25Index {
    pub k1: f64,
    pub b: f64,
    pub corpus_size: usize,
    pub avg_doc_len: f64,
    pub doc_lens: Vec<usize>,
    pub term_freqs: Vec<HashMap<String, usize>>,
    pub doc_freqs: HashMap<String, usize>,
    pub idf: HashMap<String, f64>,
}

impl Default for BM25Index {
    fn default() -> Self {
        Self::new(BM25_K1, BM25_B)
    }
}

impl BM25Index {
    pub fn new(k1: f64, b: f64) -> Self {
        Self {
            k1,
            b,
            corpus_size: 0,
            avg_doc_len: 0.0,
            doc_lens: Vec::new(),
            term_freqs: Vec::new(),
            doc_freqs: HashMap::new(),
            idf: HashMap::new(),
        }
    }

    pub fn fit(&mut self, tokenized_docs: &[Vec<String>]) {
        self.corpus_size = tokenized_docs.len();
        if self.corpus_size == 0 {
            return;
        }

        let mut total_len = 0;
        self.doc_lens.clear();
        self.term_freqs.clear();
        self.doc_freqs.clear();

        for doc in tokenized_docs {
            let d_len = doc.len();
            self.doc_lens.push(d_len);
            total_len += d_len;

            let mut counts = HashMap::new();
            for term in doc {
                *counts.entry(term.clone()).or_insert(0) += 1;
            }

            for term in counts.keys() {
                *self.doc_freqs.entry(term.clone()).or_insert(0) += 1;
            }
            self.term_freqs.push(counts);
        }

        self.avg_doc_len = (total_len as f64 / self.corpus_size as f64).max(1.0);

        self.idf.clear();
        for (term, freq) in &self.doc_freqs {
            let n = self.corpus_size as f64;
            let f = *freq as f64;
            let val = ((n - f + 0.5) / (f + 0.5) + 1.0).ln();
            self.idf.insert(term.clone(), val.max(0.01));
        }
    }

    pub fn score(&self, query_tokens: &[String]) -> Vec<f64> {
        if self.corpus_size == 0 || query_tokens.is_empty() {
            return vec![0.0; self.corpus_size];
        }

        let mut scores = vec![0.0; self.corpus_size];
        for term in query_tokens {
            if let Some(&idf_val) = self.idf.get(term) {
                for (idx, score) in scores.iter_mut().enumerate().take(self.corpus_size) {
                    let tf = *self.term_freqs[idx].get(term).unwrap_or(&0) as f64;
                    if tf == 0.0 {
                        continue;
                    }
                    let d_len = self.doc_lens[idx] as f64;
                    let denom = tf + self.k1 * (1.0 - self.b + self.b * (d_len / self.avg_doc_len));
                    let term_score = idf_val * ((tf * (self.k1 + 1.0)) / denom);
                    *score += term_score;
                }
            }
        }

        scores
    }
}
