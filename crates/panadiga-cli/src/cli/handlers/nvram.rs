//! CLI handler for Spansion NOR Flash NVRAM sanitization.

use super::super::args::{NvramActions, NvramArgs};
use anyhow::Result;
use panadiga_core::nvram::clean_flash_nvram;

pub fn handle_nvram(args: NvramArgs) -> Result<()> {
    match args.action {
        NvramActions::Clean { input, output } => {
            clean_flash_nvram(&input, output.as_deref())?;
        }
    }
    Ok(())
}
