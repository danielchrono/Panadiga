//! CLI handlers for manuals lookup, native RAG engine, AM34 memory map, and artifact hashes.

use super::super::args::{ManualsArgs, MemoryMapArgs, RagActions, RagArgs, VerifyArtifactArgs};
use crate::constants::{format_memory_map_table, get_am34_memory_map_json};
use crate::manuals::search_manuals;
use anyhow::Result;
use sha2::{Digest, Sha256};

pub fn handle_manuals(args: ManualsArgs) -> Result<()> {
    let custom_terms = if args.terms.is_empty() {
        None
    } else {
        Some(args.terms)
    };
    search_manuals(&args.dir, custom_terms, args.filter.as_deref())?;
    Ok(())
}

pub fn handle_rag(args: RagArgs) -> Result<()> {
    match args.action {
        RagActions::Query { query, json } => {
            let ans = crate::rag::query(&query);
            if json {
                let dto = ans.to_query_result();
                log_println!("{}", serde_json::to_string_pretty(&dto)?);
            } else {
                log_println!(
                    "===================================================================="
                );
                log_println!("         PANADIGA ZERO-ERROR RAG ENGINE (NATIVE RUST)");
                log_println!(
                    "===================================================================="
                );
                log_println!("Query:              {}", query);
                log_println!("Groundedness Score: {:.2}%", ans.groundedness_score * 100.0);
                log_println!("Confidence:         {:.2}%", ans.confidence * 100.0);
                if ans.is_abstained {
                    log_println!("Status:             ABSTAINED");
                    if let Some(reason) = &ans.abstention_reason {
                        log_println!("Reason:             {}", reason);
                    }
                } else {
                    log_println!("Status:             ANSWERED");
                }
                if let Some(s) = &ans.safety_report {
                    log_println!(
                        "Safety Check:       {}",
                        if s.is_safe {
                            "PASS (SAFE)"
                        } else {
                            "FAIL (HAZARD DETECTED)"
                        }
                    );
                    for v in &s.violations {
                        log_println!("  [!] VIOLATION: {}", v);
                    }
                    for a in &s.critical_alerts {
                        log_println!("  [*] ALERT: {}", a);
                    }
                }
                if let Some(c) = &ans.critic_report {
                    log_println!(
                        "Critic Faithful:    {}",
                        if c.is_faithful { "YES" } else { "NO" }
                    );
                    if !c.ungrounded_entities.is_empty() {
                        log_println!("  [?] Ungrounded:   {:?}", c.ungrounded_entities);
                    }
                }
                log_println!(
                    "--------------------------------------------------------------------"
                );
                log_println!("ANSWER:\n{}", ans.answer_text);
                if !ans.citations.is_empty() {
                    log_println!(
                        "--------------------------------------------------------------------"
                    );
                    log_println!("CITATIONS:");
                    for (idx, cit) in ans.citations.iter().enumerate() {
                        log_println!(
                            "  [{}] {} | Section: {} | Lines: {}",
                            idx + 1,
                            cit.doc_title,
                            cit.section_title,
                            cit.line_range
                        );
                        log_println!("      Excerpt: \"{}\"", cit.excerpt);
                    }
                }
                log_println!(
                    "===================================================================="
                );
            }
        }
    }
    Ok(())
}

pub fn handle_memory_map(args: MemoryMapArgs) -> Result<()> {
    let data = if args.format.to_lowercase() == "json" {
        let regions_json = get_am34_memory_map_json();
        serde_json::to_string_pretty(&regions_json)?
    } else {
        format_memory_map_table()
    };

    if let Some(out_p) = args.output {
        std::fs::write(&out_p, &data)?;
        log_println!("Memory map written to {:?}", out_p);
    } else {
        log_println!("{}", data);
    }
    Ok(())
}

pub fn handle_verify_artifact(args: VerifyArtifactArgs) -> Result<()> {
    if !args.file.exists() {
        log_error!("ERROR: File not found: {:?}", args.file);
        std::process::exit(1);
    }
    let data = std::fs::read(&args.file)?;
    let size = data.len();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let computed_sha256 = hex::encode(hasher.finalize());

    log_println!("====================================================================");
    log_println!("            CRYPTOGRAPHIC ARTIFACT VERIFICATION");
    log_println!("====================================================================");
    log_println!(
        "File:     {}",
        args.file.file_name().and_then(|n| n.to_str()).unwrap_or("")
    );
    log_println!("Size:     {} bytes ({:.2} KB)", size, size as f64 / 1024.0);
    log_println!("SHA-256:  {}", computed_sha256);
    log_println!("--------------------------------------------------------------------");

    if let Some(expected) = args.sha256 {
        if computed_sha256.eq_ignore_ascii_case(&expected) {
            log_println!("✔ HASH CONFIRMED SUCCESSFULLY!");
        } else {
            log_error!("✖ HASH MISMATCH! Expected: {}", expected);
            std::process::exit(2);
        }
    }
    Ok(())
}
