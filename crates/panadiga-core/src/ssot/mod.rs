//! Hardware & Firmware Single Source of Truth (SSOT)
//!
//! Shared across CLI, Desktop/Android UI (`apps/desktop/src-tauri`), MCP server, and RAG engine.

pub mod catalog;
pub mod spec;

pub use catalog::{
    DigaModelProfileDto, detect_diga_model_from_slice, get_diga_model_catalog_dto,
    is_valid_nor_flash_size,
};
pub use spec::{
    BreakInModeSpecDto, FlashGeometrySpecDto, HardwareSsotSpecDto, HardwareThresholdsDto,
    MemoryRegionDto, P7501PinMappingDto, PicoCommandArgOptionDto, PicoCommandSpecDto,
    PicoUsbIdentityDto, build_hardware_ssot_spec,
};
