//! Main Zero-Error RAG Engine pipeline orchestrating all 6 architectural defense layers.

use std::time::Instant;

use super::config::DEFAULT_TOP_K;
use super::critic::{FactCritic, RetrievalCritic};
use super::generator::GroundedGenerator;
use super::guardrails::QueryGuard;
use super::indexer::HierarchicalIndexer;
use super::models::{RagAnswer, SafetyReport};
use super::retriever::HybridRetriever;
use super::safety::HardwareSafetyValidator;

pub struct ZeroErrorRagEngine {
    pub guard: QueryGuard,
    pub indexer: HierarchicalIndexer,
    pub retrieval_critic: RetrievalCritic,
    pub generator: GroundedGenerator,
    pub fact_critic: FactCritic,
    pub safety_validator: HardwareSafetyValidator,
}

impl Default for ZeroErrorRagEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroErrorRagEngine {
    pub fn new() -> Self {
        Self {
            guard: QueryGuard::new(),
            indexer: HierarchicalIndexer::new(),
            retrieval_critic: RetrievalCritic::default(),
            generator: GroundedGenerator::new(),
            fact_critic: FactCritic::new(),
            safety_validator: HardwareSafetyValidator::new(),
        }
    }

    pub fn initialize(&mut self) -> usize {
        self.indexer.build_index_from_directories(None)
    }

    pub fn query(&self, raw_query: &str) -> RagAnswer {
        let t0 = Instant::now();

        // ----------------------------------------------------------------------
        // Layer 1: QueryGuard (Input Validation & Injection Detection)
        // ----------------------------------------------------------------------
        let q = self.guard.process(raw_query);
        if q.is_injection {
            let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
            return RagAnswer {
                query: raw_query.to_string(),
                answer_text: "[SECURITY ALERT] Prompt injection or safety override attempt detected. Request rejected.".to_string(),
                confidence: 0.0,
                groundedness_score: 0.0,
                safety_report: Some(SafetyReport {
                    is_safe: false,
                    violations: vec![q.validation_error.unwrap_or_else(|| "Prompt injection detected.".to_string())],
                    critical_alerts: vec!["Adversarial input blocked at Layer 1.".to_string()],
                    warnings: Vec::new(),
                }),
                citations: Vec::new(),
                critic_report: None,
                retrieved_chunks_count: 0,
                execution_time_ms: elapsed,
                is_abstained: true,
                abstention_reason: Some("Prompt injection detected.".to_string()),
            };
        }

        // ----------------------------------------------------------------------
        // Pre-Execution Safety Shield on Query Itself
        // ----------------------------------------------------------------------
        let query_safety = self.safety_validator.validate(&q.sanitized_text, true);
        if !query_safety.is_safe {
            let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
            let violations_str = query_safety
                .violations
                .iter()
                .map(|v| format!("• {}", v))
                .collect::<Vec<String>>()
                .join("\n");

            return RagAnswer {
                query: raw_query.to_string(),
                answer_text: format!(
                    "[CRITICAL AIR-GAP INTERCEPTION] Request blocked by the hardware safety shield to prevent physical equipment damage:\n\n{}",
                    violations_str
                ),
                confidence: 0.0,
                groundedness_score: 0.0,
                safety_report: Some(query_safety),
                citations: Vec::new(),
                critic_report: None,
                retrieved_chunks_count: 0,
                execution_time_ms: elapsed,
                is_abstained: true,
                abstention_reason: Some("Physical hardware invariant violation detected in query.".to_string()),
            };
        }

        // ----------------------------------------------------------------------
        // Layer 2: Hybrid Retrieval (BM25 + TF-IDF Vector + Entity Boost)
        // ----------------------------------------------------------------------
        let retriever = HybridRetriever::new(&self.indexer);
        let retrieval_results = retriever.retrieve(&q, DEFAULT_TOP_K);

        // ----------------------------------------------------------------------
        // Layer 3: RetrievalCritic (Corrective RAG - CRAG)
        // ----------------------------------------------------------------------
        let (is_grounded, valid_results, reason) =
            self.retrieval_critic.evaluate_retrieval(&q, &retrieval_results);

        if !is_grounded {
            let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
            return RagAnswer {
                query: raw_query.to_string(),
                answer_text: "Information not verified in technical corpus. In proprietary embedded systems, ungrounded inferences cause physical hardware destruction.".to_string(),
                confidence: 0.0,
                groundedness_score: 0.0,
                safety_report: None,
                citations: Vec::new(),
                critic_report: None,
                retrieved_chunks_count: retrieval_results.len(),
                execution_time_ms: elapsed,
                is_abstained: true,
                abstention_reason: Some(reason),
            };
        }

        // ----------------------------------------------------------------------
        // Layer 4: Grounded Synthesis
        // ----------------------------------------------------------------------
        let (claims_body, citations) = self.generator.synthesize_claims(&valid_results);
        let mut answer_text = self.generator.wrap_grounded_answer(&q, &claims_body);

        // ----------------------------------------------------------------------
        // Layer 5: FactCritic (Self-RAG Post-Generation Entity Grounding)
        // Audits only the synthesized factual claims (excluding boilerplate header/footer)
        // ----------------------------------------------------------------------
        let source_chunks: Vec<_> = valid_results.iter().map(|r| r.chunk.clone()).collect();
        let critic_report = self.fact_critic.audit(&claims_body, &source_chunks);

        // ----------------------------------------------------------------------
        // Layer 6: Hardware Safety Shield (Deterministic Post-Generation Veto)
        // ----------------------------------------------------------------------
        let post_safety = self.safety_validator.validate(&claims_body, false);

        let mut is_abstained = false;
        let mut abstention_reason = None;

        if !critic_report.is_faithful {
            is_abstained = true;
            abstention_reason = Some(format!(
                "Self-RAG FactCritic detected ungrounded technical entities: {}",
                critic_report.ungrounded_entities.join(", ")
            ));

            let ungrounded_list = critic_report
                .ungrounded_entities
                .iter()
                .map(|e| format!("• {}", e))
                .collect::<Vec<String>>()
                .join("\n");

            answer_text = format!(
                "[EPISTEMIC ABSTENTION - UNGROUNDED ENTITY DETECTED]\n\n\
                The response was intercepted because the following technical entities were not grounded in the verified corpus:\n\
                {}\n\n\
                In embedded hardware reverse engineering, answering with unverified parameters is strictly prohibited to prevent equipment damage.",
                ungrounded_list
            );
        }

        if !post_safety.is_safe {
            is_abstained = true;
            abstention_reason = Some("Generated recommendations violated physical hardware safety invariants.".to_string());
            let violations_str = post_safety
                .violations
                .iter()
                .map(|v| format!("• {}", v))
                .collect::<Vec<String>>()
                .join("\n");

            answer_text = format!(
                "[CRITICAL AIR-GAP INTERCEPTION] Generated recommendations were blocked due to safety violations:\n\n{}",
                violations_str
            );
        }

        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        let confidence = if critic_report.is_faithful && post_safety.is_safe && !is_abstained {
            1.0
        } else {
            0.0
        };

        RagAnswer {
            query: raw_query.to_string(),
            answer_text,
            citations,
            confidence,
            groundedness_score: critic_report.groundedness_score,
            critic_report: Some(critic_report),
            safety_report: Some(post_safety),
            retrieved_chunks_count: valid_results.len(),
            execution_time_ms: elapsed,
            is_abstained,
            abstention_reason,
        }
    }
}
