//! Dual-layer critic: Corrective RAG (CRAG) retrieval critic & Self-RAG fact auditor.

use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

use super::config::{ABSTENTION_SIMILARITY_THRESHOLD, MIN_GROUNDEDNESS_RATIO, MIN_RELEVANCE_SCORE};
use super::guardrails::{KNOWN_COMPONENTS, KNOWN_PINS_AND_SIGNALS};
use super::indexer::technical_tokenize;
use super::models::{Chunk, CriticReport, EntityType, Query, RetrievalResult};
use super::safety::extract_voltages_volts;

static HEX_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b0x[0-9A-Fa-f]+\b").unwrap());

static WORD_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b[A-Za-z0-9_#/]+").unwrap());

pub struct RetrievalCritic {
    pub threshold: f64,
}

impl Default for RetrievalCritic {
    fn default() -> Self {
        Self::new(ABSTENTION_SIMILARITY_THRESHOLD)
    }
}

impl RetrievalCritic {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    pub fn evaluate_retrieval(
        &self,
        query: &Query,
        results: &[RetrievalResult],
    ) -> (bool, Vec<RetrievalResult>, String) {
        if results.is_empty() {
            return (
                false,
                Vec::new(),
                "No relevant documents found in technical corpus. In embedded reverse engineering, answering without verified documentation is prohibited.".to_string(),
            );
        }

        let top_score = results[0].score;
        if top_score < self.threshold {
            return (
                false,
                Vec::new(),
                format!(
                    "Top document similarity score ({:.4}) is below confidence threshold ({:.4}). Abstaining to prevent physical hardware damage.",
                    top_score, self.threshold
                ),
            );
        }

        // Check content term coverage using canonical technical tokenizer
        let query_tokens: Vec<String> = technical_tokenize(&query.sanitized_text, true)
            .into_iter()
            .filter(|w| w.len() > 2)
            .collect();

        if !query_tokens.is_empty() {
            let top_matched: HashSet<&String> = results[0].matched_tokens.iter().collect();
            let matched_count = query_tokens
                .iter()
                .filter(|w| top_matched.contains(w))
                .count();
            let coverage = matched_count as f64 / query_tokens.len() as f64;

            let chunk_upper = results[0].chunk.text.to_uppercase();
            let has_entity_grounding = query.extracted_entities.iter().any(|e| {
                matches!(
                    e.entity_type,
                    EntityType::HexAddress
                        | EntityType::ErrorCode
                        | EntityType::Pin
                        | EntityType::Component
                        | EntityType::LbaSector
                ) && chunk_upper.contains(&e.name.to_uppercase())
            });

            if coverage < 0.45 && !has_entity_grounding {
                return (
                    false,
                    Vec::new(),
                    format!(
                        "Low technical term coverage ({:.1}%). Core query concepts not found in technical corpus.",
                        coverage * 100.0
                    ),
                );
            }
        }

        let filtered: Vec<RetrievalResult> = results
            .iter()
            .filter(|r| r.score >= MIN_RELEVANCE_SCORE)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return (
                false,
                Vec::new(),
                "All retrieved chunks fell below the minimum relevance filter.".to_string(),
            );
        }

        (
            true,
            filtered,
            "Retrieval grounded and sufficient.".to_string(),
        )
    }
}

pub struct FactCritic;

impl Default for FactCritic {
    fn default() -> Self {
        Self::new()
    }
}

impl FactCritic {
    pub fn new() -> Self {
        Self
    }

    fn extract_hex_integers(text: &str) -> HashSet<u64> {
        let mut set = HashSet::new();
        for m in HEX_REGEX.find_iter(text) {
            let raw = m.as_str().trim_start_matches("0x").trim_start_matches("0X");
            if let Ok(val) = u64::from_str_radix(raw, 16) {
                set.insert(val);
            }
        }
        set
    }

    fn extract_voltages(text: &str) -> HashSet<u64> {
        let mut set = HashSet::new();
        for (v, _, _, _) in extract_voltages_volts(text) {
            // Millivolt integer representation for exact float hashing
            let mv = (v * 1000.0).round() as u64;
            set.insert(mv);
        }
        set
    }

    pub fn audit(&self, generated_text: &str, source_chunks: &[Chunk]) -> CriticReport {
        let combined_source_text = source_chunks
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<&str>>()
            .join("\n");

        let mut verified_entities = Vec::new();
        let mut ungrounded_entities = Vec::new();

        // 1. Audit Hex Addresses via Canonical Integers
        let gen_hex_ints = Self::extract_hex_integers(generated_text);
        let src_hex_ints = Self::extract_hex_integers(&combined_source_text);

        for h_int in gen_hex_ints {
            let h_str = format!("0x{:X}", h_int);
            if src_hex_ints.contains(&h_int) {
                verified_entities.push(format!("HEX:{}", h_str));
            } else {
                ungrounded_entities.push(format!("HEX:{}", h_str));
            }
        }

        // 2. Audit Voltages
        let gen_volts = Self::extract_voltages(generated_text);
        let src_volts = Self::extract_voltages(&combined_source_text);

        for mv in gen_volts {
            let v_str = format!("{:.1}V", mv as f64 / 1000.0);
            if src_volts.contains(&mv) {
                verified_entities.push(format!("VOLTAGE:{}", v_str));
            } else {
                ungrounded_entities.push(format!("VOLTAGE:{}", v_str));
            }
        }

        // 3. Audit Components and Pins
        let upper_gen = generated_text.to_uppercase();
        let upper_src = combined_source_text.to_uppercase();
        let gen_words: HashSet<&str> = WORD_REGEX
            .find_iter(&upper_gen)
            .map(|m| m.as_str())
            .collect();

        for &comp in KNOWN_COMPONENTS {
            if gen_words.contains(comp) || upper_gen.contains(comp) {
                if upper_src.contains(comp) {
                    verified_entities.push(format!("COMPONENT:{}", comp));
                } else {
                    ungrounded_entities.push(format!("COMPONENT:{}", comp));
                }
            }
        }

        for &pin in KNOWN_PINS_AND_SIGNALS {
            if gen_words.contains(pin) {
                if upper_src.contains(pin) {
                    verified_entities.push(format!("PIN:{}", pin));
                } else {
                    ungrounded_entities.push(format!("PIN:{}", pin));
                }
            }
        }

        let total_entities = verified_entities.len() + ungrounded_entities.len();
        let groundedness_score = if total_entities == 0 {
            1.0
        } else {
            verified_entities.len() as f64 / total_entities as f64
        };

        let is_faithful =
            groundedness_score >= MIN_GROUNDEDNESS_RATIO && ungrounded_entities.is_empty();

        let mut notes = Vec::new();
        if !ungrounded_entities.is_empty() {
            notes.push(format!(
                "Generated text contains ungrounded technical entities: {}",
                ungrounded_entities.join(", ")
            ));
        }

        CriticReport {
            is_faithful,
            groundedness_score,
            verified_entities,
            ungrounded_entities,
            contradicted_facts: Vec::new(),
            confidence: if is_faithful { 1.0 } else { 0.0 },
            notes,
        }
    }
}
