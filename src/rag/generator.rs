//! Grounded synthesis generator enforcing zero-extrapolation and mandatory citations.

use std::collections::HashSet;

use super::config::MAX_CHUNKS_PER_PROMPT;
use super::models::{Citation, Query, RetrievalResult};

pub struct GroundedGenerator;

impl Default for GroundedGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl GroundedGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn build_grounded_prompt(&self, query: &Query, results: &[RetrievalResult]) -> String {
        let mut context_blocks = Vec::new();
        for (idx, res) in results.iter().take(MAX_CHUNKS_PER_PROMPT).enumerate() {
            let c = &res.chunk;
            let m = &c.metadata;
            let label = format!(
                "[Doc {}: {} > {} (Lines {}-{})]",
                idx + 1,
                m.doc_title,
                m.section_title,
                m.line_start,
                m.line_end
            );
            context_blocks.push(format!("{}\n{}\n", label, c.text));
        }

        let joined_context = context_blocks.join("\n---\n");

        format!(
            "You are the Lead Embedded Systems and Reverse Engineering Specialist for the Panasonic DIGA DMR-EH55.\n\
            MANDATORY RULES:\n\
            1. Answer ONLY based on the verified technical context below. Do not extrapolate, guess, or infer from standard PC conventions.\n\
            2. If the answer is not in the context, explicitly state: 'Information not verified in technical corpus.'\n\
            3. Every factual claim must include a bracketed citation in the form [DocTitle, SectionTitle].\n\
            4. NEVER recommend voltages above 3.3V, flash operations in SA0..SA133, or JTAG TDI/TDO on CN55001.\n\n\
            TECHNICAL CONTEXT:\n{}\n\n\
            ENGINEER QUERY: {}\n\
            GROUNDED ANSWER:",
            joined_context, query.sanitized_text
        )
    }

    pub fn synthesize_claims(
        &self,
        results: &[RetrievalResult],
    ) -> (String, Vec<Citation>) {
        if results.is_empty() {
            return (
                "Information not verified in technical corpus. In proprietary embedded systems, ungrounded inferences cause physical hardware destruction.".to_string(),
                Vec::new(),
            );
        }

        let mut citations = Vec::new();
        let mut extracted_facts = Vec::new();
        let mut seen_sections = HashSet::new();

        for res in results.iter().take(MAX_CHUNKS_PER_PROMPT) {
            let chunk = &res.chunk;
            let meta = &chunk.metadata;
            let sec_key = (meta.doc_title.clone(), meta.section_title.clone());
            if seen_sections.contains(&sec_key) {
                continue;
            }
            seen_sections.insert(sec_key);

            let lines: Vec<&str> = chunk
                .text
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();

            if lines.is_empty() {
                continue;
            }

            let excerpt = lines.iter().take(3).copied().collect::<Vec<&str>>().join(" ");
            let citation_label = format!("[{}, {}]", meta.doc_title, meta.section_title);

            extracted_facts.push(format!(
                "• According to verified documentation {}: {}",
                citation_label, excerpt
            ));

            let snippet = if excerpt.chars().count() > 150 {
                let truncated: String = excerpt.chars().take(150).collect();
                format!("{}...", truncated)
            } else {
                excerpt
            };

            citations.push(Citation {
                doc_title: meta.doc_title.clone(),
                file_path: meta.file_path.clone(),
                section_title: meta.section_title.clone(),
                line_range: format!("L{}-L{}", meta.line_start, meta.line_end),
                excerpt: snippet,
            });
        }

        let body = extracted_facts.join("\n\n");
        (body, citations)
    }

    pub fn wrap_grounded_answer(&self, query: &Query, body: &str) -> String {
        let header = format!("Verified technical diagnosis for '{}':\n\n", query.sanitized_text);
        let footer = "\n\n[Hardware Safety Note: All operations must comply with 3.3V maximum nominal tolerance and preserve flash IPL sectors SA0..SA133].";
        format!("{}{}{}", header, body, footer)
    }

    pub fn synthesize_deterministic(
        &self,
        query: &Query,
        results: &[RetrievalResult],
    ) -> (String, Vec<Citation>) {
        if results.is_empty() {
            return (
                "Information not verified in technical corpus. In proprietary embedded systems, ungrounded inferences cause physical hardware destruction.".to_string(),
                Vec::new(),
            );
        }
        let (body, citations) = self.synthesize_claims(results);
        let full_answer = self.wrap_grounded_answer(query, &body);
        (full_answer, citations)
    }
}
