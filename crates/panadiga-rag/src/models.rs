//! Data models for the Panadiga Zero-Error RAG Engine.
//!
//! Strongly typed data structures designed specifically for mission-critical
//! embedded systems and hardware reverse engineering where hallucinations can
//! cause physical hardware destruction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubsystemType {
    Hardware,
    Firmware,
    Storage,
    Crypto,
    ReverseEngineering,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    HexAddress,
    Component,
    Pin,
    Voltage,
    ErrorCode,
    LbaSector,
    MagicSignature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub raw_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub doc_title: String,
    pub file_path: String,
    pub section_title: String,
    pub hierarchy: Vec<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub subsystem: SubsystemType,
    pub extracted_entities: Vec<Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub text: String,
    pub metadata: ChunkMetadata,
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub raw_text: String,
    pub sanitized_text: String,
    pub intent: String,
    pub extracted_entities: Vec<Entity>,
    pub is_high_risk: bool,
    pub is_injection: bool,
    pub validation_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub chunk: Chunk,
    pub score: f64,
    pub lexical_score: f64,
    pub semantic_score: f64,
    pub rrf_score: f64,
    pub matched_tokens: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CriticReport {
    pub is_faithful: bool,
    pub groundedness_score: f64,
    pub verified_entities: Vec<String>,
    pub ungrounded_entities: Vec<String>,
    pub contradicted_facts: Vec<String>,
    pub confidence: f64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafetyReport {
    pub is_safe: bool,
    pub violations: Vec<String>,
    pub critical_alerts: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub doc_title: String,
    pub file_path: String,
    pub section_title: String,
    pub line_range: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagAnswer {
    pub query: String,
    pub answer_text: String,
    pub citations: Vec<Citation>,
    pub confidence: f64,
    pub groundedness_score: f64,
    pub critic_report: Option<CriticReport>,
    pub safety_report: Option<SafetyReport>,
    pub retrieved_chunks_count: usize,
    pub execution_time_ms: f64,
    pub is_abstained: bool,
    pub abstention_reason: Option<String>,
}

/// DTO formatted to directly fulfill the Tauri desktop frontend `RagQueryResult` interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQueryResultDto {
    pub query: String,
    pub answer: String,
    pub confidence: f64,
    pub groundedness_score: f64,
    pub is_abstained: bool,
    pub abstention_reason: Option<String>,
    pub citations: Vec<RagCitationDto>,
    pub safety: RagSafetyDto,
    pub critic: RagCriticDto,
    pub retrieved_chunks_count: usize,
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagCitationDto {
    pub doc: String,
    pub section: String,
    pub lines: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSafetyDto {
    pub is_safe: bool,
    pub violations: Vec<String>,
    pub alerts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagCriticDto {
    pub is_faithful: bool,
    pub ungrounded_entities: Vec<String>,
}

impl RagAnswer {
    pub fn to_query_result(&self) -> RagQueryResultDto {
        RagQueryResultDto {
            query: self.query.clone(),
            answer: self.answer_text.clone(),
            confidence: (self.confidence * 10000.0).round() / 10000.0,
            groundedness_score: (self.groundedness_score * 10000.0).round() / 10000.0,
            is_abstained: self.is_abstained,
            abstention_reason: self.abstention_reason.clone(),
            citations: self
                .citations
                .iter()
                .map(|c| RagCitationDto {
                    doc: c.doc_title.clone(),
                    section: c.section_title.clone(),
                    lines: c.line_range.clone(),
                    excerpt: c.excerpt.clone(),
                })
                .collect(),
            safety: RagSafetyDto {
                is_safe: self
                    .safety_report
                    .as_ref()
                    .map(|s| s.is_safe)
                    .unwrap_or(true),
                violations: self
                    .safety_report
                    .as_ref()
                    .map(|s| s.violations.clone())
                    .unwrap_or_default(),
                alerts: self
                    .safety_report
                    .as_ref()
                    .map(|s| s.critical_alerts.clone())
                    .unwrap_or_default(),
            },
            critic: RagCriticDto {
                is_faithful: self
                    .critic_report
                    .as_ref()
                    .map(|c| c.is_faithful)
                    .unwrap_or(true),
                ungrounded_entities: self
                    .critic_report
                    .as_ref()
                    .map(|c| c.ungrounded_entities.clone())
                    .unwrap_or_default(),
            },
            retrieved_chunks_count: self.retrieved_chunks_count,
            execution_time_ms: (self.execution_time_ms * 100.0).round() / 100.0,
        }
    }
}
