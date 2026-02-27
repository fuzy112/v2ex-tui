use anyhow::Result;
use std::fmt;

/// Browser operation result types
#[allow(dead_code)] // Error variant not currently used, but kept for completeness
#[derive(Debug, Clone)]
pub enum BrowserResult {
    Success { url: String, description: String },
    Error { description: String, error: String },
}

impl BrowserResult {
    pub fn success(url: impl Into<String>, description: impl Into<String>) -> Self {
        Self::Success {
            url: url.into(),
            description: description.into(),
        }
    }

    #[allow(dead_code)] // Error creation helper not currently used, but kept for API completeness
    pub fn error(description: impl Into<String>, error: impl Into<String>) -> Self {
        Self::Error {
            description: description.into(),
            error: error.into(),
        }
    }
}

/// Browser command configuration - can be set to use a custom browser
use std::sync::OnceLock;

static BROWSER_COMMAND: OnceLock<Vec<String>> = OnceLock::new();

/// Set the browser command to use for opening URLs
pub fn set_browser_command(cmd: Vec<String>) {
    let _ = BROWSER_COMMAND.set(cmd);
}

/// Centralized browser operations
pub struct Browser;

impl Browser {
    /// Open URL in browser with consistent error handling
    /// Uses custom browser command if set, otherwise uses system default
    pub fn open_url(url: impl AsRef<str>) -> Result<BrowserResult> {
        let url_str = url.as_ref().to_string();
        let url_for_thread = url_str.clone();

        // Spawn a thread to open browser without blocking the main event loop
        std::thread::spawn(move || {
            // Check if custom browser command is configured
            if let Some(cmd) = BROWSER_COMMAND.get() {
                // Use custom browser command
                let mut command = std::process::Command::new(&cmd[0]);
                for arg in &cmd[1..] {
                    command.arg(arg);
                }
                command.arg(&url_for_thread);

                match command.spawn() {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!(
                            "Failed to open browser command for {}: {}",
                            url_for_thread, e
                        );
                        // Fallback to webbrowser crate
                        if let Err(e2) = webbrowser::open(&url_for_thread) {
                            eprintln!("Fallback also failed for {}: {}", url_for_thread, e2);
                        }
                    }
                }
            } else {
                // Use system default browser
                match webbrowser::open(&url_for_thread) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Failed to open browser for {}: {}", url_for_thread, e);
                    }
                }
            }
        });

        // Return success immediately (non-blocking)
        Ok(BrowserResult::success(
            &url_str,
            format!("Opening {} in browser", url_str),
        ))
    }

    /// Open V2EX topic in browser
    pub fn open_topic(topic_id: i64) -> Result<BrowserResult> {
        let url = format!("https://www.v2ex.com/t/{}", topic_id);
        Self::open_url(&url)
    }

    /// Open V2EX topic reply in browser
    pub fn open_topic_reply(topic_id: i64, reply_id: i64) -> Result<BrowserResult> {
        let url = format!("https://www.v2ex.com/t/{}#r_{}", topic_id, reply_id);
        Self::open_url(&url)
    }

    /// Open V2EX node in browser
    #[allow(dead_code)] // Node browser opening not currently used in UI, but kept for API completeness
    pub fn open_node(node_name: impl AsRef<str>) -> Result<BrowserResult> {
        let url = format!("https://www.v2ex.com/go/{}", node_name.as_ref());
        Self::open_url(&url)
    }
}

/// Display implementation for user-facing messages
impl fmt::Display for BrowserResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserResult::Success { description, .. } => write!(f, "{}", description),
            BrowserResult::Error { description, error } => write!(f, "{}: {}", description, error),
        }
    }
}
