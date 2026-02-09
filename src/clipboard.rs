use anyhow::{Context, Result};

/// Copy text to clipboard
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("Failed to access clipboard")?;
    clipboard
        .set_text(text)
        .context("Failed to copy text to clipboard")?;
    Ok(())
}
