//! Panadiga Zero-Error RAG Engine (Native Rust Implementation)
//!
//! Replaces legacy Python RAG with a deterministic, memory-safe, zero-copy,
//! 6-layer defense engine executing in sub-milliseconds without external runtime dependencies.

pub mod config;
pub mod critic;
pub mod generator;
pub mod guardrails;
pub mod indexer;
pub mod models;
pub mod pipeline;
pub mod retriever;
pub mod safety;

#[cfg(test)]
mod tests;

pub use config::*;
pub use critic::*;
pub use generator::*;
pub use guardrails::*;
pub use indexer::*;
pub use models::*;
pub use pipeline::*;
pub use retriever::*;
pub use safety::*;

use std::sync::{LazyLock, Mutex};

static GLOBAL_ENGINE: LazyLock<Mutex<ZeroErrorRagEngine>> = LazyLock::new(|| {
    let mut engine = ZeroErrorRagEngine::new();
    engine.initialize();
    Mutex::new(engine)
});

/// Convenience helper querying the global thread-safe Zero-Error RAG Engine.
pub fn query(text: &str) -> RagAnswer {
    let engine = GLOBAL_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
    engine.query(text)
}
