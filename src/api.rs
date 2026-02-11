use anyhow::{Context, Result};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const BASE_URL: &str = "https://www.v2ex.com/api/v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub username: String,
    pub url: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub psn: Option<String>,
    pub github: Option<String>,
    pub btc: Option<String>,
    pub location: Option<String>,
    pub tagline: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub avatar_mini: Option<String>,
    pub avatar_normal: Option<String>,
    pub avatar_large: Option<String>,
    #[serde(default)]
    pub created: i64,
    pub last_modified: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub title: String,
    pub title_alternative: Option<String>,
    pub topics: i64,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub avatar: Option<String>,
    pub avatar_mini: Option<String>,
    pub avatar_normal: Option<String>,
    pub avatar_large: Option<String>,
    pub created: i64,
    pub last_modified: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: i64,
    pub node: Option<Node>,
    pub member: Option<Member>,
    pub last_reply_by: Option<String>,
    pub last_touched: Option<i64>,
    pub title: String,
    pub url: String,
    pub created: i64,
    pub deleted: Option<i64>,
    pub content: Option<String>,
    pub content_rendered: Option<String>,
    pub last_modified: Option<i64>,
    pub replies: i64,
}

impl Topic {
    /// Get the node title for display
    pub fn node_title(&self) -> &str {
        self.node
            .as_ref()
            .map(|n| n.title.as_str())
            .unwrap_or("Unknown")
    }

    /// Get the member username for display
    pub fn author_name(&self) -> &str {
        self.member
            .as_ref()
            .map(|m| m.username.as_str())
            .unwrap_or("Unknown")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub id: i64,
    pub member: Option<Member>,
    pub content: Option<String>,
    pub content_rendered: Option<String>,
    pub created: i64,
    pub last_modified: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotificationPayload {
    String(String),
    Object {
        topic_id: Option<i64>,
        topic_title: Option<String>,
        reply_id: Option<i64>,
        body: Option<String>,
    },
}

impl NotificationPayload {
    pub fn extract_body(&self) -> Option<String> {
        match self {
            NotificationPayload::String(s) => {
                // Parse format like "@chingyat #20\r\n\r\nsystemd 会在后面的章节里面。"
                // Extract everything after the second newline
                let parts: Vec<&str> = s.splitn(3, '\n').collect();
                if parts.len() >= 3 {
                    Some(parts[2].trim().to_string())
                } else {
                    Some(s.clone())
                }
            }
            NotificationPayload::Object { body, .. } => body.clone(),
        }
    }

    #[allow(dead_code)] // API completeness - may be used in future
    pub fn extract_reply_id(&self) -> Option<i64> {
        match self {
            NotificationPayload::String(s) => {
                // Try to extract reply ID from format like "@chingyat #20"
                if let Some(hash_pos) = s.find('#') {
                    let after_hash = &s[hash_pos + 1..];
                    let end_pos = after_hash
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(after_hash.len());
                    after_hash[..end_pos].parse().ok()
                } else {
                    None
                }
            }
            NotificationPayload::Object { reply_id, .. } => *reply_id,
        }
    }
}

impl Notification {
    /// Extract topic ID from notification text
    pub fn extract_topic_id(&self) -> Option<i64> {
        // Look for pattern like /t/1180785 in the text
        let text = &self.text;

        // Find /t/ pattern
        if let Some(t_pos) = text.find("/t/") {
            let after_t = &text[t_pos + 3..];
            let end_pos = after_t
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_t.len());
            after_t[..end_pos].parse().ok()
        } else {
            None
        }
    }

    /// Extract reply ID from notification
    #[allow(dead_code)] // API completeness - may be used in future
    pub fn extract_reply_id(&self) -> Option<i64> {
        self.payload.as_ref().and_then(|p| p.extract_reply_id())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub member_id: i64,
    pub member: Option<Member>,
    pub for_member_id: i64,
    pub text: String,
    pub payload: Option<NotificationPayload>,
    pub payload_rendered: Option<String>,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by get_token_info() for GET /token endpoint
pub struct TokenInfo {
    pub token: String,
    pub scope: String,
    pub expiration: i64,
    pub good_for_days: i64,
    pub total_used: i64,
    pub last_used: Option<i64>,
    pub last_use_ip: Option<String>,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub result: Option<T>,
}

#[derive(Clone)]
pub struct V2exClient {
    token: String,
    client: reqwest::Client,
}

impl V2exClient {
    pub fn new(token: String) -> Self {
        Self {
            token,
            client: reqwest::Client::new(),
        }
    }

    pub fn load_token() -> Result<String> {
        let config_dir = Self::config_dir()?;
        let token_path = config_dir.join("token.txt");
        let token = std::fs::read_to_string(&token_path)
            .with_context(|| format!("Failed to read token from {:?}", token_path))?;
        Ok(token.trim().to_string())
    }

    pub fn config_dir() -> Result<PathBuf> {
        let base_dirs = BaseDirs::new().context("Failed to get base directories")?;
        let config_dir = base_dirs.config_dir().join("v2ex");
        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
    ) -> Result<ApiResponse<T>> {
        self.request_with_optional_body::<T, ()>(method, endpoint, None)
            .await
    }

    #[allow(dead_code)]
    async fn request_with_body<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: &B,
    ) -> Result<ApiResponse<T>> {
        self.request_with_optional_body(method, endpoint, Some(body))
            .await
    }

    async fn request_with_optional_body<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<&B>,
    ) -> Result<ApiResponse<T>> {
        let url = format!("{}/{}", BASE_URL, endpoint);
        let mut request = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.token));

        if let Some(body) = body {
            request = request
                .header("Content-Type", "application/json")
                .json(body);
        }

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;

        // Handle empty responses
        if text.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "API returned empty response (status: {})",
                status
            ));
        }

        // Parse JSON response
        let api_response: ApiResponse<T> = match serde_json::from_str(&text) {
            Ok(resp) => resp,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse API response: {}. Status: {}. Raw response: {}",
                    e,
                    status,
                    &text[..text.len().min(500)]
                ));
            }
        };

        // Check HTTP status
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "API error: {} - {:?}",
                status,
                api_response.message
            ));
        }

        Ok(api_response)
    }

    pub async fn get_member(&self) -> Result<Member> {
        let response: ApiResponse<Member> = self.request(reqwest::Method::GET, "member").await?;
        response.result.context("No member data in response")
    }

    #[allow(dead_code)] // For GET /token endpoint (available but not used in UI)
    pub async fn get_token_info(&self) -> Result<TokenInfo> {
        let response: ApiResponse<TokenInfo> = self.request(reqwest::Method::GET, "token").await?;
        response.result.context("No token data in response")
    }

    pub async fn get_notifications(&self, page: i32) -> Result<Vec<Notification>> {
        let endpoint = format!("notifications?p={}", page);
        let response: ApiResponse<Vec<Notification>> =
            self.request(reqwest::Method::GET, &endpoint).await?;
        Ok(response.result.unwrap_or_default())
    }

    #[allow(dead_code)]
    pub async fn delete_notification(&self, notification_id: i64) -> Result<()> {
        let endpoint = format!("notifications/{}", notification_id);
        let _: ApiResponse<serde_json::Value> =
            self.request(reqwest::Method::DELETE, &endpoint).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_node(&self, node_name: &str) -> Result<Node> {
        let endpoint = format!("nodes/{}", node_name);
        let response: ApiResponse<Node> = self.request(reqwest::Method::GET, &endpoint).await?;
        response.result.context("No node data in response")
    }

    pub async fn get_node_topics(&self, node_name: &str, page: i32) -> Result<Vec<Topic>> {
        let endpoint = format!("nodes/{}/topics?p={}", node_name, page);
        let response: ApiResponse<Vec<Topic>> =
            self.request(reqwest::Method::GET, &endpoint).await?;
        Ok(response.result.unwrap_or_default())
    }

    pub async fn get_topic(&self, topic_id: i64) -> Result<Topic> {
        let endpoint = format!("topics/{}", topic_id);
        let response: ApiResponse<Topic> = self.request(reqwest::Method::GET, &endpoint).await?;
        response.result.context("No topic data in response")
    }

    pub async fn get_topic_replies(&self, topic_id: i64, page: i32) -> Result<Vec<Reply>> {
        let endpoint = format!("topics/{}/replies?p={}", topic_id, page);
        let response: ApiResponse<Vec<Reply>> =
            self.request(reqwest::Method::GET, &endpoint).await?;
        Ok(response.result.unwrap_or_default())
    }
}

#[derive(Debug, Clone)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    pub date: String,
    #[allow(dead_code)] // Not currently used, but kept for future display
    pub author: Option<String>,
    /// Unix timestamp for relative time display
    pub timestamp: Option<i64>,
}

impl RssItem {
    /// Extract topic ID from RSS item link
    /// V2EX links are typically: https://www.v2ex.com/t/123456 or https://www.v2ex.com/t/123456#reply1
    pub fn extract_topic_id(&self) -> Option<i64> {
        // Find /t/ pattern in the link
        if let Some(t_pos) = self.link.find("/t/") {
            let after_t = &self.link[t_pos + 3..];
            let end_pos = after_t
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_t.len());
            after_t[..end_pos].parse().ok()
        } else {
            None
        }
    }
}

impl V2exClient {
    pub async fn get_rss_feed(&self, tab: &str) -> Result<Vec<RssItem>> {
        use anyhow::Context;
        use atom_syndication::Feed;

        let url = if tab == "index" {
            "https://www.v2ex.com/index.xml".to_string()
        } else {
            format!("https://www.v2ex.com/feed/tab/{}.xml", tab)
        };

        // Create a client with custom settings for RSS fetching
        let rss_client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; v2ex-tui/0.1.0)")
            .danger_accept_invalid_certs(true)
            .build()
            .context("Failed to create RSS client")?;

        let response = rss_client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch RSS feed from {}", url))?;

        let content = response
            .bytes()
            .await
            .with_context(|| format!("Failed to read RSS feed content from {}", url))?;

        // Convert to string for debugging
        let content_str = String::from_utf8_lossy(&content);

        let feed = match Feed::read_from(&content[..]) {
            Ok(feed) => feed,
            Err(e) => {
                let preview = if content_str.len() > 200 {
                    &content_str[..200]
                } else {
                    &content_str
                };
                return Err(anyhow::anyhow!(
                    "Failed to parse Atom feed from {}: {}. Content preview: {}",
                    url,
                    e,
                    preview
                ));
            }
        };

        let items: Vec<RssItem> = feed
            .entries()
            .iter()
            .map(|entry| {
                let title = entry.title().to_string();
                let link = entry
                    .links()
                    .first()
                    .map(|link| link.href().to_string())
                    .unwrap_or_else(|| "".to_string());

                // Format date and extract timestamp
                let (date, timestamp) = entry
                    .published()
                    .or_else(|| Some(entry.updated()))
                    .map(|d| {
                        // Convert to chrono DateTime and format as YYYY-MM-DD HH:MM
                        let dt: chrono::DateTime<chrono::Utc> = (*d).into();
                        let formatted = dt.format("%Y-%m-%d %H:%M").to_string();
                        let ts = dt.timestamp();
                        (formatted, Some(ts))
                    })
                    .unwrap_or_else(|| ("Unknown date".to_string(), None));

                let author = entry
                    .authors()
                    .first()
                    .map(|author| author.name().to_string());

                RssItem {
                    title,
                    link,
                    date,
                    author,
                    timestamp,
                }
            })
            .collect();

        Ok(items)
    }
}
