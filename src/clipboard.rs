use anyhow::{Context, Result};
use std::thread;
use std::time::Duration;

/// Copy text to clipboard
/// 
/// Note: On Linux, we need to keep the clipboard alive for a short time
/// to ensure clipboard managers can capture the contents.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("Failed to access clipboard")?;
    clipboard
        .set_text(text)
        .context("Failed to copy text to clipboard")?;
    
    // Keep clipboard alive for a short time to ensure clipboard managers
    // on Linux can capture the contents
    #[cfg(target_os = "linux")]
    thread::sleep(Duration::from_millis(100));
    
    Ok(())
}
