//! Okapi BM25 lexical scoring index over tokenized technical chunks.

use rayon::prelude::*;
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

    pub fn fit<T: AsRef<[String]> + Sync>(&mut self, tokenized_docs: &[T]) {
        self.corpus_size = tokenized_docs.len();
        self.doc_lens.clear();
        self.term_freqs.clear();
        self.doc_freqs.clear();
        self.idf.clear();

        if self.corpus_size == 0 {
            return;
        }

        let per_doc: Vec<(usize, HashMap<String, usize>)> = tokenized_docs
            .par_iter()
            .map(|entry| {
                let doc = entry.as_ref();
                let mut counts = HashMap::with_capacity(doc.len());
                for term in doc {
                    *counts.entry(term.clone()).or_insert(0) += 1;
                }
                (doc.len(), counts)
            })
            .collect();

        let mut total_len = 0usize;
        self.doc_lens.reserve(self.corpus_size);
        self.term_freqs.reserve(self.corpus_size);

        for (d_len, counts) in per_doc {
            total_len += d_len;
            self.doc_lens.push(d_len);
            for term in counts.keys() {
                *self.doc_freqs.entry(term.clone()).or_insert(0) += 1;
            }
            self.term_freqs.push(counts);
        }

        self.avg_doc_len = (total_len as f64 / self.corpus_size as f64).max(1.0);

        let n = self.corpus_size as f64;
        self.idf.reserve(self.doc_freqs.len());
        for (term, freq) in &self.doc_freqs {
            let f = *freq as f64;
            let val = ((n - f + 0.5) / (f + 0.5) + 1.0).ln();
            self.idf.insert(term.clone(), val.max(0.01));
        }
    }

    pub fn score(&self, query_tokens: &[String]) -> Vec<f64> {
        if self.corpus_size == 0 || query_tokens.is_empty() {
            return vec![0.0; self.corpus_size];
        }

        let query_idf: Vec<(&String, f64)> = query_tokens
            .iter()
            .filter_map(|t| self.idf.get(t).map(|&idf_val| (t, idf_val)))
            .collect();
        if query_idf.is_empty() {
            return vec![0.0; self.corpus_size];
        }

        let score_doc = |idx: usize| -> f64 {
            let tf_map = &self.term_freqs[idx];
            let d_len = self.doc_lens[idx] as f64;
            let base_norm = self.k1 * (1.0 - self.b + self.b * (d_len / self.avg_doc_len));
            let mut doc_score = 0.0;
            for &(term, idf_val) in &query_idf {
                let tf = *tf_map.get(term).unwrap_or(&0) as f64;
                if tf > 0.0 {
                    doc_score += idf_val * ((tf * (self.k1 + 1.0)) / (tf + base_norm));
                }
            }
            doc_score
        };

        if self.corpus_size >= 128 {
            (0..self.corpus_size)
                .into_par_iter()
                .map(score_doc)
                .collect()
        } else {
            (0..self.corpus_size).map(score_doc).collect()
        }
    }
}
