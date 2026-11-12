//! Panasonic DMR-EH55 DIGA Suite - Desktop IPC Data Transfer Objects (DTOs)
//!
//! Modularized by domain concern while preserving a unified re-export surface
//! for all Tauri IPC command handlers (`crate::desktop::commands::*`).

pub mod catalog;
pub mod hdd;
pub mod paths;
pub mod system;

pub use super::super::pico::{
    Am34RegisterState, FirstContactBackupResult, GdbBridgeStatus, GdbPacketTelemetryEvent,
    InCircuitNvramEraseResult, PicoDeviceStatusDto, PicoStatusResponse,
};
pub use crate::crypto::{
    CodebookReport, CryptanalysisReport as FrmEntropyReport, FrmTocDumpReport,
};
pub use crate::hdd::{ExtractedPartitionReport, HddAnalysisReport, MeihdfsAllocationReport};
pub use crate::manuals::PdfSearchResult as ManualSearchResult;
pub use crate::nvram::NvramCleanReport;

pub use catalog::{
    DigaModelProfileDto, detect_diga_model_from_slice, get_diga_model_catalog_dto,
    is_valid_nor_flash_size,
};
pub use hdd::{ExtractedPartitionItemDto, PartitionExtractionManifestDto};
pub use paths::{resolve_candidate, resolve_default_flash_path};
pub use system::{BinaryFileStatusDto, HexDumpRowDto, SystemOverviewDto};
