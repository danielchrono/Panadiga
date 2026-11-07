//! Hierarchical markdown chunker and hybrid indexer (BM25 + TF-IDF Vector Index).

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use super::config::{BM25_B, BM25_K1};
use super::guardrails::QueryGuard;
use super::models::{Chunk, ChunkMetadata, SubsystemType};

pub const STOPWORDS: &[&str] = &[
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
    "da", "de", "do", "das", "dos", "em", "no", "na", "nos", "nas", "para", "por", "com", "um", "uma",
];

static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b0x[0-9A-Fa-f]+\b|[A-Za-z0-9_#/]+").unwrap()
});

static STOPWORDS_SET: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    STOPWORDS.iter().copied().collect()
});

/// Tokenizes text preserving hexadecimal values, IC numbers, pins, and error codes.
pub fn technical_tokenize(text: &str, filter_stopwords: bool) -> Vec<String> {
    let mut tokens = Vec::new();
    for m in TOKEN_REGEX.find_iter(text) {
        let tok = m.as_str().trim_matches(|c: char| {
            c == '.' || c == ',' || c == ';' || c == ':' || c == '(' || c == ')'
                || c == '[' || c == ']' || c == '{' || c == '}' || c == '\'' || c == '"'
        });
        if tok.is_empty() {
            continue;
        }

        if tok.starts_with("0x") || tok.starts_with("0X") {
            tokens.push(tok.to_uppercase());
        } else {
            let lower = tok.to_lowercase();
            if filter_stopwords && STOPWORDS_SET.contains(lower.as_str()) {
                continue;
            }
            tokens.push(lower);
        }
    }
    tokens
}

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

        self.avg_doc_len = total_len as f64 / self.corpus_size as f64;

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
                for idx in 0..self.corpus_size {
                    let tf = *self.term_freqs[idx].get(term).unwrap_or(&0) as f64;
                    if tf == 0.0 {
                        continue;
                    }
                    let d_len = self.doc_lens[idx] as f64;
                    let denom = tf + self.k1 * (1.0 - self.b + self.b * (d_len / self.avg_doc_len));
                    let term_score = idf_val * ((tf * (self.k1 + 1.0)) / denom);
                    scores[idx] += term_score;
                }
            }
        }

        scores
    }
}

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
                    let weight = (1.0 + (count as f64).ln()) * self.idf.get(term).copied().unwrap_or(1.0);
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

        let q_norm = if q_norm_sq > 0.0 { q_norm_sq.sqrt() } else { 1.0 };
        if q_norm == 0.0 {
            return vec![0.0; corpus_size];
        }

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

pub struct HierarchicalIndexer {
    pub guard: QueryGuard,
    pub chunks: Vec<Chunk>,
    pub bm25: BM25Index,
    pub vector: VectorIndex,
}

impl Default for HierarchicalIndexer {
    fn default() -> Self {
        Self::new()
    }
}

fn flush_chunk(
    chunks: &mut Vec<Chunk>,
    guard: &QueryGuard,
    file_path: &Path,
    doc_title: &str,
    subsystem: SubsystemType,
    current_section: &str,
    current_hierarchy: &[String],
    lines_buf: &mut Vec<(usize, &str)>,
    end_line: usize,
) {
    if lines_buf.is_empty() {
        return;
    }
    let text_block: String = lines_buf
        .iter()
        .map(|(_, l)| *l)
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string();

    if text_block.len() >= 30 {
        let start_line = lines_buf[0].0;
        let stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
        let cid = format!("{}::L{}-L{}", stem, start_line, end_line);

        let meta_query = guard.process(&text_block);

        let meta = ChunkMetadata {
            doc_title: doc_title.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            section_title: current_section.to_string(),
            hierarchy: current_hierarchy.to_vec(),
            line_start: start_line,
            line_end: end_line,
            subsystem,
            extracted_entities: meta_query.extracted_entities,
        };

        let tokens = technical_tokenize(&text_block, true);
        chunks.push(Chunk {
            id: cid,
            text: text_block,
            metadata: meta,
            tokens,
        });
    }
    lines_buf.clear();
}

impl HierarchicalIndexer {
    pub fn new() -> Self {
        Self {
            guard: QueryGuard::new(),
            chunks: Vec::new(),
            bm25: BM25Index::default(),
            vector: VectorIndex::default(),
        }
    }

    pub fn determine_subsystem(&self, file_path: &Path, content: &str) -> SubsystemType {
        let filename_lower = file_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_lowercase();
        let content_lower = content.to_lowercase();

        if filename_lower.contains("dwire") || filename_lower.contains("hardware") || filename_lower.contains("esquema") {
            SubsystemType::Hardware
        } else if filename_lower.contains("meihdfs") || filename_lower.contains("hdd") || filename_lower.contains("alocacao") {
            SubsystemType::Storage
        } else if filename_lower.contains("frm") || filename_lower.contains("crypto") || filename_lower.contains("aes") {
            SubsystemType::Crypto
        } else if filename_lower.contains("ghidra") || filename_lower.contains("gdb") || filename_lower.contains("reverse") {
            SubsystemType::ReverseEngineering
        } else if content_lower.contains("boot") || content_lower.contains("ipl") || content_lower.contains("flash") {
            SubsystemType::Firmware
        } else {
            SubsystemType::General
        }
    }

    pub fn chunk_markdown_file(&self, file_path: &Path) -> Vec<Chunk> {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let doc_title = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Document")
            .replace('_', " ");

        let mut current_hierarchy = vec![doc_title.clone()];
        let mut current_section = "Introduction".to_string();
        let mut current_lines: Vec<(usize, &str)> = Vec::new();

        let subsystem = self.determine_subsystem(file_path, &content);

        for (line_idx_zero, &line) in lines.iter().enumerate() {
            let line_idx = line_idx_zero + 1;
            let stripped = line.trim();

            if stripped.starts_with('#') {
                if !current_lines.is_empty() {
                    flush_chunk(
                        &mut chunks,
                        &self.guard,
                        file_path,
                        &doc_title,
                        subsystem,
                        &current_section,
                        &current_hierarchy,
                        &mut current_lines,
                        line_idx - 1,
                    );
                }

                let level = stripped.chars().take_while(|&c| c == '#').count();
                let heading_text = stripped.trim_start_matches('#').trim().to_string();
                current_section = heading_text.clone();

                if level == 1 {
                    current_hierarchy = vec![heading_text];
                } else if level == 2 {
                    let first = current_hierarchy.first().cloned().unwrap_or_else(|| doc_title.clone());
                    current_hierarchy = vec![first, heading_text];
                } else if current_hierarchy.len() >= 2 {
                    current_hierarchy = vec![current_hierarchy[0].clone(), current_hierarchy[1].clone(), heading_text];
                } else {
                    current_hierarchy.push(heading_text);
                }

                current_lines.push((line_idx, line));
            } else {
                current_lines.push((line_idx, line));
                if current_lines.len() >= 80 {
                    flush_chunk(
                        &mut chunks,
                        &self.guard,
                        file_path,
                        &doc_title,
                        subsystem,
                        &current_section,
                        &current_hierarchy,
                        &mut current_lines,
                        line_idx,
                    );
                }
            }
        }

        if !current_lines.is_empty() {
            flush_chunk(
                &mut chunks,
                &self.guard,
                file_path,
                &doc_title,
                subsystem,
                &current_section,
                &current_hierarchy,
                &mut current_lines,
                lines.len(),
            );
        }

        chunks
    }

    pub fn build_index_from_directories(&mut self, directories: Option<&[PathBuf]>) -> usize {
        let default_dirs = [
            PathBuf::from("docs"),
            PathBuf::from("manuals_and_schematics"),
            PathBuf::from("../../docs"),
            PathBuf::from("../../manuals_and_schematics"),
            PathBuf::from("../../../docs"),
            PathBuf::from("../../../manuals_and_schematics"),
        ];

        let dirs_to_scan = directories.unwrap_or(&default_dirs);
        let mut all_chunks = Vec::new();
        let mut seen_canonical = HashSet::new();

        for d in dirs_to_scan {
            if !d.exists() {
                continue;
            }
            if let Ok(canon) = fs::canonicalize(d) {
                if !seen_canonical.insert(canon) {
                    continue;
                }
            }
            self.scan_recursive(d, &mut all_chunks);
        }

        self.chunks = all_chunks;
        self.reindex_models();
        self.chunks.len()
    }

    fn reindex_models(&mut self) {
        let tokenized: Vec<Vec<String>> = self.chunks.iter().map(|c| c.tokens.clone()).collect();
        self.bm25.fit(&tokenized);
        self.vector.fit(&tokenized);
    }

    fn scan_recursive(&self, dir: &Path, acc: &mut Vec<Chunk>) {
        if let Ok(entries) = fs::read_dir(dir) {
            let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();

            for path in paths {
                if path.is_dir() {
                    self.scan_recursive(&path, acc);
                } else if path.is_file() {
                    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if (fname.ends_with(".md") || fname.ends_with(".txt"))
                        && !fname.starts_with("AI_GUIDELINES")
                        && !fname.starts_with("AUDIT_DOSSIER")
                    {
                        acc.extend(self.chunk_markdown_file(&path));
                    }
                }
            }
        }
    }

    pub fn add_custom_chunks(&mut self, chunks: Vec<Chunk>) {
        self.chunks.extend(chunks);
        self.reindex_models();
    }
}
