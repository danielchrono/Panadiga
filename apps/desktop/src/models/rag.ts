/**
 * @file rag.ts
 * @brief Zero-Error RAG Engine Domain Models & Citations.
 */

export interface RagCitation {
  readonly doc: string;
  readonly section: string;
  readonly lines: string;
  readonly excerpt: string;
}

export interface RagQueryResult {
  readonly query: string;
  readonly answer: string;
  readonly confidence: number;
  readonly groundedness_score: number;
  readonly is_abstained: boolean;
  readonly abstention_reason: string | null;
  readonly citations: readonly RagCitation[];
  readonly safety: {
    readonly is_safe: boolean;
    readonly violations: readonly string[];
    readonly alerts: readonly string[];
  };
  readonly critic: {
    readonly is_faithful: boolean;
    readonly ungrounded_entities: readonly string[];
  };
  readonly retrieved_chunks_count: number;
  readonly execution_time_ms: number;
}
