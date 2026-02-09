use anyhow::{Context, Result};
use crossterm::execute;
use std::io::{stdout, Write};

/// Copy text to clipboard using OSC 52 escape sequence
///
/// OSC 52 is a terminal escape sequence that allows applications to
/// set the system clipboard. This works in most modern terminal
/// emulators and is particularly useful over SSH connections.
///
/// The sequence format is: ESC ] 52 ; c ; <base64-data> BEL
/// Where 'c' is the clipboard selection (c = clipboard, p = primary, s = secondary)
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    if !is_osc52_supported() {
        return Err(anyhow::anyhow!(
            "OSC 52 not supported by this terminal emulator"
        ));
    }

    use base64::Engine;

    // Encode text as base64
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);

    // Build OSC 52 sequence
    // ESC ] 52 ; c ; <base64> BEL
    let osc52 = format!("\x1b]52;c;{}\x07", encoded);

    // Write using crossterm to ensure proper terminal handling
    let mut stdout = stdout();
    execute!(stdout, crossterm::style::Print(&osc52)).context("Failed to write OSC 52 sequence")?;

    stdout.flush().context("Failed to flush stdout")?;

    Ok(())
}

/// Check if the terminal supports OSC 52
///
/// This is a best-effort check. We look for common terminal emulators
/// that are known to support OSC 52.
pub fn is_osc52_supported() -> bool {
    // Check for known supporting terminals via environment variables
    let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
    let term = std::env::var("TERM").unwrap_or_default();

    match term_program.as_str() {
        "iTerm.app" | "Apple_Terminal" | "Hyper" | "vscode" | "WezTerm" => true,
        _ => {
            // Check for other indicators
            if std::env::var("KITTY_WINDOW_ID").is_ok() {
                return true;
            }
            if term.contains("kitty") || term.contains("alacritty") {
                return true;
            }
            // Default to true - most modern terminals support OSC 52
            // and it won't cause issues if they don't
            true
        }
    }
}
