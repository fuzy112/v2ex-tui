use anyhow::{Context, Result};
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

/// Centralized browser operations
pub struct Browser;

impl Browser {
    /// Open URL in default browser with consistent error handling
    pub fn open_url(url: impl AsRef<str>) -> Result<BrowserResult> {
        let url = url.as_ref();
        webbrowser::open(url)
            .map(|_| BrowserResult::success(url, format!("Opened {} in browser", url)))
            .context("Failed to open browser")
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
