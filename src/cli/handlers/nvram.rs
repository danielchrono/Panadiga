//! CLI handler for Spansion NOR Flash NVRAM sanitization.

use super::super::args::{NvramActions, NvramArgs};
use crate::nvram::clean_flash_nvram;
use anyhow::Result;

pub fn handle_nvram(args: NvramArgs) -> Result<()> {
    match args.action {
        NvramActions::Clean { input, output } => {
            clean_flash_nvram(&input, output.as_deref())?;
        }
    }
    Ok(())
}
