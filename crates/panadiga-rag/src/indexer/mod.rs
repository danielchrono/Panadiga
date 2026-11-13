//! Hierarchical multi-modal corpus indexer (Markdown docs, Service Manual PDFs, Source Code & Ghidra).

pub mod bm25;
pub mod code;
pub mod markdown;
pub mod pdf;
pub mod tokenizer;
pub mod vector;

pub use bm25::BM25Index;
pub use tokenizer::{STOPWORDS, technical_tokenize};
pub use vector::VectorIndex;

use panadiga_core::constants::resolve_workspace_root;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::guardrails::QueryGuard;
use super::models::{Chunk, ChunkMetadata, SubsystemType};

enum IndexedBucket {
    Markdown(Vec<Chunk>),
    Pdf(Vec<Chunk>),
    Code(Vec<Chunk>),
    Ghidra(Vec<Chunk>),
    Skipped,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CorpusIndexStats {
    pub total_chunks: usize,
    pub markdown_chunks: usize,
    pub pdf_chunks: usize,
    pub pdf_files_indexed: usize,
    pub code_chunks: usize,
    pub ghidra_chunks: usize,
    pub vocab_size: usize,
    pub corpus_root: String,
}

pub struct HierarchicalIndexer {
    pub guard: QueryGuard,
    pub chunks: Vec<Chunk>,
    pub bm25: BM25Index,
    pub vector: VectorIndex,
    pub stats: CorpusIndexStats,
}

impl Default for HierarchicalIndexer {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchicalIndexer {
    pub fn new() -> Self {
        Self {
            guard: QueryGuard::new(),
            chunks: Vec::new(),
            bm25: BM25Index::default(),
            vector: VectorIndex::default(),
            stats: CorpusIndexStats::default(),
        }
    }

    pub fn corpus_stats(&self) -> CorpusIndexStats {
        let mut stats = self.stats.clone();
        stats.total_chunks = self.chunks.len();
        stats.vocab_size = self.vector.vocab.len();
        stats
    }

    pub fn determine_subsystem(&self, file_path: &Path, content: &str) -> SubsystemType {
        let path_lower = file_path.to_string_lossy().to_lowercase();
        let filename_lower = file_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_lowercase();
        let content_lower = content.to_lowercase();

        if filename_lower.contains("p7501")
            || filename_lower.contains("hardware")
            || filename_lower.contains("esquema")
            || filename_lower.contains("schematic")
            || path_lower.contains("pico-p7501")
        {
            SubsystemType::Hardware
        } else if filename_lower.contains("meihdfs")
            || filename_lower.contains("hdd")
            || filename_lower.contains("alocacao")
        {
            SubsystemType::Storage
        } else if filename_lower.contains("frm")
            || filename_lower.contains("crypto")
            || filename_lower.contains("aes")
        {
            SubsystemType::Crypto
        } else if filename_lower.contains("ghidra")
            || filename_lower.contains("gdb")
            || filename_lower.contains("reverse")
            || path_lower.contains("ghidra")
        {
            SubsystemType::ReverseEngineering
        } else if content_lower.contains("boot")
            || content_lower.contains("ipl")
            || content_lower.contains("flash")
            || filename_lower.contains("nvram")
        {
            SubsystemType::Firmware
        } else {
            SubsystemType::General
        }
    }

    pub fn chunk_markdown_file(&self, file_path: &Path) -> Vec<Chunk> {
        let preview = fs::read_to_string(file_path).unwrap_or_default();
        let subsystem = self.determine_subsystem(file_path, &preview);
        markdown::chunk_markdown_file(&self.guard, subsystem, file_path)
    }

    pub fn build_index_from_directories(&mut self, directories: Option<&[PathBuf]>) -> usize {
        let root = resolve_workspace_root();
        self.stats = CorpusIndexStats {
            corpus_root: root.to_string_lossy().to_string(),
            ..Default::default()
        };

        let mut seen_canonical = HashSet::new();
        let mut candidate_files = Vec::new();

        if let Some(custom_dirs) = directories {
            for d in custom_dirs {
                Self::collect_entry(d, &mut seen_canonical, &mut candidate_files);
            }
        } else {
            let default_entries = [
                root.join("docs"),
                root.join("manuals_and_schematics"),
                root.join("firmware").join("pico-p7501"),
                root.join("tools").join("ghidra"),
                root.join("crates"),
                root.join("apps")
                    .join("desktop")
                    .join("src-tauri")
                    .join("src"),
                root.join("apps").join("desktop").join("src"),
                root.join("scripts").join("re"),
                root.join("scripts").join("bench"),
                root.join("AGENTS.md"),
                root.join("README.md"),
            ];
            for entry in &default_entries {
                Self::collect_entry(entry, &mut seen_canonical, &mut candidate_files);
            }
        }

        let per_file: Vec<IndexedBucket> = candidate_files
            .par_iter()
            .map(|path| self.index_single_file(path))
            .collect();

        let mut all_chunks = Vec::new();
        for bucket in per_file {
            match bucket {
                IndexedBucket::Markdown(added) => {
                    self.stats.markdown_chunks += added.len();
                    all_chunks.extend(added);
                }
                IndexedBucket::Pdf(added) => {
                    if !added.is_empty() {
                        self.stats.pdf_files_indexed += 1;
                        self.stats.pdf_chunks += added.len();
                        all_chunks.extend(added);
                    }
                }
                IndexedBucket::Code(added) => {
                    self.stats.code_chunks += added.len();
                    all_chunks.extend(added);
                }
                IndexedBucket::Ghidra(added) => {
                    self.stats.ghidra_chunks += added.len();
                    all_chunks.extend(added);
                }
                IndexedBucket::Skipped => {}
            }
        }

        pdf::flush_disk_cache();
        self.chunks = all_chunks;
        self.reindex_models();
        self.chunks.len()
    }

    fn collect_entry(
        entry: &Path,
        seen_canonical: &mut HashSet<PathBuf>,
        files: &mut Vec<PathBuf>,
    ) {
        if !entry.exists() {
            return;
        }
        if let Ok(canon) = fs::canonicalize(entry)
            && !seen_canonical.insert(canon)
        {
            return;
        }
        if entry.is_file() {
            files.push(entry.to_path_buf());
        } else if entry.is_dir() {
            Self::collect_recursive(entry, files);
        }
    }

    fn index_single_file(&self, path: &Path) -> IndexedBucket {
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let path_lower = path.to_string_lossy().to_lowercase();

        if fname.starts_with("AI_GUIDELINES") || fname.starts_with("AUDIT_DOSSIER") {
            return IndexedBucket::Skipped;
        }

        if fname.ends_with(".md") || fname.ends_with(".txt") {
            IndexedBucket::Markdown(self.chunk_markdown_file(path))
        } else if fname == "visual_manifest.json" {
            IndexedBucket::Markdown(self.chunk_visual_manifest(path))
        } else if fname.to_lowercase().ends_with(".pdf") {
            let subsystem = self.determine_subsystem(path, "");
            IndexedBucket::Pdf(pdf::chunk_pdf_file(&self.guard, subsystem, path))
        } else if code::is_supported_source_file(path)
            && !path_lower.contains("\\tests\\")
            && !path_lower.contains("/tests/")
        {
            let subsystem = self.determine_subsystem(path, "");
            let added = code::chunk_source_file(&self.guard, subsystem, path);
            if path_lower.contains("ghidra") {
                IndexedBucket::Ghidra(added)
            } else {
                IndexedBucket::Code(added)
            }
        } else {
            IndexedBucket::Skipped
        }
    }

    fn reindex_models(&mut self) {
        let doc_refs: Vec<&[String]> = self.chunks.iter().map(|c| c.tokens.as_slice()).collect();
        let bm25 = &mut self.bm25;
        let vector = &mut self.vector;
        rayon::join(|| bm25.fit(&doc_refs), || vector.fit(&doc_refs));
        self.stats.total_chunks = self.chunks.len();
        self.stats.vocab_size = self.vector.vocab.len();
    }

    fn collect_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
        let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if matches!(
            dir_name,
            "target"
                | "node_modules"
                | "dist"
                | ".git"
                | "tests"
                | "test"
                | "mocks"
                | "fixtures"
                | "e2e"
                | "__pycache__"
        ) {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();

            for path in paths {
                if path.is_dir() {
                    Self::collect_recursive(&path, files);
                } else if path.is_file() {
                    files.push(path);
                }
            }
        }
    }

    /// Indexes curated DMR-EH55 schematic & PCB PNG metadata from `docs/assets/visual_manifest.json`.
    fn chunk_visual_manifest(&self, path: &Path) -> Vec<Chunk> {
        #[derive(Deserialize)]
        struct VisualEntry {
            id: String,
            filename: String,
            repo_path: String,
            #[serde(default)]
            aliases: Vec<String>,
            #[serde(default)]
            manual_edition: String,
            page_number: Option<u32>,
            #[serde(default)]
            category: String,
            #[serde(default)]
            tags: Vec<String>,
            #[serde(default)]
            caption: String,
        }
        #[derive(Deserialize)]
        struct VisualManifest {
            #[serde(default)]
            images: Vec<VisualEntry>,
        }

        let Ok(raw) = fs::read_to_string(path) else {
            return Vec::new();
        };
        let Ok(manifest) = serde_json::from_str::<VisualManifest>(&raw) else {
            return Vec::new();
        };
        let path_str = path.to_string_lossy().to_string();
        let mut chunks = Vec::with_capacity(manifest.images.len());
        for (idx, item) in manifest.images.into_iter().enumerate() {
            let page_str = item
                .page_number
                .map(|p| p.to_string())
                .unwrap_or_else(|| "N/A".to_string());
            let aliases = if item.aliases.is_empty() {
                String::new()
            } else {
                format!(" Aliases: {}.", item.aliases.join(", "))
            };
            let tags = if item.tags.is_empty() {
                String::new()
            } else {
                format!(" Tags: {}.", item.tags.join(", "))
            };
            let text = format!(
                "[Visual Asset Catalog] {} ({}) — {} p.{}, Categoria: {}.{} {} {} Path: {}",
                item.filename,
                item.id,
                item.manual_edition,
                page_str,
                item.category,
                aliases,
                tags,
                item.caption,
                item.repo_path
            );
            let tokens = technical_tokenize(&text, true);
            let extracted_entities = self.guard.process(&text).extracted_entities;
            chunks.push(Chunk {
                id: format!("visual_manifest::{}", item.id),
                text,
                tokens,
                metadata: ChunkMetadata {
                    doc_title: "Visual Asset Catalog (DMR-EH55 Service Manuals)".to_string(),
                    file_path: path_str.clone(),
                    section_title: format!("{} (p.{})", item.filename, page_str),
                    hierarchy: vec!["Visual Catalog".to_string(), item.category, item.filename],
                    line_start: idx + 1,
                    line_end: idx + 1,
                    subsystem: SubsystemType::Hardware,
                    extracted_entities,
                },
            });
        }
        chunks
    }

    pub fn add_custom_chunks(&mut self, chunks: Vec<Chunk>) {
        self.chunks.extend(chunks);
        self.reindex_models();
    }
}
