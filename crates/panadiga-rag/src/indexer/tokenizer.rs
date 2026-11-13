//! Technical domain tokenizer preserving hexadecimal literals, ICs, pin names, and error codes.

use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub const STOPWORDS: &[&str] = &[
    "a", "about", "above", "after", "again", "against", "all", "am", "an", "and", "any", "are",
    "as", "at", "be", "because", "been", "before", "being", "below", "between", "both", "but",
    "by", "can", "cannot", "could", "did", "do", "does", "doing", "down", "during", "each", "few",
    "for", "from", "further", "had", "has", "have", "having", "he", "her", "here", "hers", "him",
    "his", "how", "i", "if", "in", "into", "is", "it", "its", "itself", "me", "more", "most", "my",
    "myself", "no", "nor", "not", "of", "off", "on", "once", "only", "or", "other", "our", "ours",
    "out", "over", "own", "same", "she", "should", "so", "some", "such", "than", "that", "the",
    "their", "theirs", "them", "then", "there", "these", "they", "this", "those", "through", "to",
    "too", "under", "until", "up", "very", "was", "we", "were", "what", "when", "where", "which",
    "while", "who", "whom", "why", "with", "would", "you", "your", "yours", "da", "de", "do",
    "das", "dos", "em", "no", "na", "nos", "nas", "para", "por", "com", "um", "uma",
];

static TOKEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b0x[0-9A-Fa-f]+\b|[A-Za-z0-9_#/]+").unwrap());

static STOPWORDS_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| STOPWORDS.iter().copied().collect());

/// Folds common Latin accented characters to ASCII equivalents so Portuguese queries match corpus tokens.
fn fold_latin_accents(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let mapped = match ch {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'ê' | 'è' | 'ë' | 'É' | 'Ê' | 'È' | 'Ë' => 'e',
            'í' | 'î' | 'ì' | 'ï' | 'Í' | 'Î' | 'Ì' | 'Ï' => 'i',
            'ó' | 'ô' | 'õ' | 'ò' | 'ö' | 'Ó' | 'Ô' | 'Õ' | 'Ò' | 'Ö' => 'o',
            'ú' | 'û' | 'ù' | 'ü' | 'Ú' | 'Û' | 'Ù' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            'ñ' | 'Ñ' => 'n',
            _ => ch,
        };
        out.push(mapped);
    }
    out
}

/// Tokenizes text preserving hexadecimal values, IC numbers, pins, and error codes.
pub fn technical_tokenize(text: &str, filter_stopwords: bool) -> Vec<String> {
    let normalized = fold_latin_accents(text);
    let mut tokens = Vec::new();
    for m in TOKEN_REGEX.find_iter(&normalized) {
        let tok = m.as_str().trim_matches(|c: char| {
            matches!(
                c,
                '.' | ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '\'' | '"'
            )
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
