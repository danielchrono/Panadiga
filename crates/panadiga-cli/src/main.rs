//! Panasonic DIGA Engineering CLI Suite (`panadiga`)
//!
//! Cross-platform engineering & forensics CLI:
//! - Without arguments or with `desktop` / `gui`: locates and launches the sibling native GUI (`panadiga-desktop`).
//! - With CLI subcommands or `-i` / `--interactive`: executes the forensic CLI / TUI engine.

use anyhow::Result;
use clap::Parser;
use panadiga_cli::cli::{Cli, Commands, dispatch, launch_desktop_executable};
use panadiga_cli::tui::run_interactive_pipeline;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "desktop" || args[1] == "gui" {
        if launch_desktop_executable().is_ok() {
            return Ok(());
        }
        return run_interactive_pipeline();
    }

    let cli = Cli::parse();

    if cli.interactive
        || cli.command.is_none()
        || matches!(cli.command, Some(Commands::Interactive))
    {
        return run_interactive_pipeline();
    }

    dispatch(cli.command.unwrap())
}
