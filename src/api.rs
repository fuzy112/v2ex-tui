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
    /// Get the node name for display
    pub fn node_name(&self) -> &str {
        self.node
            .as_ref()
            .map(|n| n.name.as_str())
            .unwrap_or("unknown")
    }

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

    pub fn extract_reply_id(&self) -> Option<i64> {
        match self {
            NotificationPayload::String(s) => {
                // Try to extract reply ID from format like "@chingyat #20"
                if let Some(hash_pos) = s.find('#') {
                    let after_hash = &s[hash_pos + 1..];
                    let end_pos = after_hash
                        .find(|c: char| !c.is_digit(10))
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
                .find(|c: char| !c.is_digit(10))
                .unwrap_or(after_t.len());
            after_t[..end_pos].parse().ok()
        } else {
            None
        }
    }

    /// Extract reply ID from notification
    pub fn extract_reply_id(&self) -> Option<i64> {
        self.payload.as_ref().and_then(|p| p.extract_reply_id())
    }

    /// Check if this notification has a topic link
    pub fn has_topic_link(&self) -> bool {
        self.extract_topic_id().is_some()
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
        let url = format!("{}/{}", BASE_URL, endpoint);
        let response = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        let status = response.status();
        let api_response: ApiResponse<T> = response.json().await?;

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

    pub async fn delete_notification(&self, notification_id: i64) -> Result<()> {
        let endpoint = format!("notifications/{}", notification_id);
        let _: ApiResponse<serde_json::Value> =
            self.request(reqwest::Method::DELETE, &endpoint).await?;
        Ok(())
    }

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
