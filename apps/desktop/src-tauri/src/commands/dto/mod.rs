//! Panasonic DMR-EH55 DIGA Suite - Desktop IPC Data Transfer Objects (DTOs)
//!
//! Modularized by domain concern while re-exporting shared hardware SSOT types from `panadiga-core`.

pub mod hdd;
pub mod paths;
pub mod system;

pub use crate::pico::{
    Am34RegisterState, FirstContactBackupResult, GdbBridgeStatus, GdbPacketTelemetryEvent,
    InCircuitNvramEraseResult, PicoDeviceStatusDto, PicoStatusResponse,
};
pub use panadiga_core::crypto::{
    CodebookReport, CryptanalysisReport as FrmEntropyReport, FrmTocDumpReport,
};
pub use panadiga_core::hdd::{
    ExtractedPartitionReport, HddAnalysisReport, MeihdfsAllocationReport,
};
pub use panadiga_core::manuals::PdfSearchResult as ManualSearchResult;
pub use panadiga_core::nvram::NvramCleanReport;
pub use panadiga_core::ssot::{
    DigaModelProfileDto, HardwareSsotSpecDto, build_hardware_ssot_spec,
    detect_diga_model_from_slice, get_diga_model_catalog_dto, is_valid_nor_flash_size,
};

pub use hdd::{ExtractedPartitionItemDto, PartitionExtractionManifestDto};
pub use paths::{resolve_candidate, resolve_default_flash_path};
pub use system::{
    BinaryFileStatusDto, HexDumpRowDto, LiveNvramCaptureEventDto, ServiceManualCatalogItemDto,
    SystemOverviewDto,
};
