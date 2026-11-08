//! Panasonic DIGA Engineering CLI & Desktop Suite (`panadiga`)
//!
//! Unified multi-platform binary:
//! - Without arguments or with `desktop` / `gui`: launches the native desktop GUI (WebView2 / WebKitGTK / WKWebView).
//! - With CLI subcommands or `-i` / `--interactive`: attaches to caller terminal and executes the forensic CLI / TUI engine.

use anyhow::Result;
use clap::Parser;
use panadiga::cli::{dispatch, Cli, Commands};
use panadiga::tui::run_interactive_pipeline;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // 1. If launched without arguments or explicitly with "desktop" / "gui", run Desktop GUI
    if args.len() < 2 || (args.len() >= 2 && (args[1] == "desktop" || args[1] == "gui")) {
        #[cfg(windows)]
        unsafe {
            // Detach console window cleanly when double-clicked from Explorer
            let _ = windows_sys::Win32::System::Console::FreeConsole();
        }
        panadiga::desktop::run_desktop_app();
        return Ok(());
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
