//! Desktop DTOs for MEIHFS HDD partition extraction and manifest serialization.

use serde::{Deserialize, Serialize};

/// Frontend-compatible extracted partition item for `extract_partitions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPartitionItemDto {
    pub id: usize,
    pub name: String,
    pub filename: String,
    #[serde(rename = "offsetHex")]
    pub offset_hex: String,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: usize,
    pub sha256: String,
    pub md5: String,
    pub entropy: f64,
}

/// Frontend-compatible partition extraction manifest for `extract_partitions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionExtractionManifestDto {
    pub timestamp: String,
    #[serde(rename = "sourceImage")]
    pub source_image: String,
    #[serde(rename = "outputDirectory")]
    pub output_directory: String,
    #[serde(rename = "manifestSha256Sums")]
    pub manifest_sha256_sums: String,
    #[serde(rename = "extractedPartitions")]
    pub extracted_partitions: Vec<ExtractedPartitionItemDto>,
}
