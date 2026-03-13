use crate::api::{Member, Notification, Reply, RssItem, Topic};
use crate::nodes::get_all_nodes;

/// Format a topic for text output
pub fn format_topic(topic: &Topic, index: Option<usize>) -> String {
    let idx_str = match index {
        Some(i) => format!("[{}] ", i + 1),
        None => String::new(),
    };

    let node = topic.node_title();
    let author = topic.author_name();
    let replies = if topic.replies > 0 {
        format!(" [{} replies]", topic.replies)
    } else {
        String::new()
    };

    format!(
        "{}{}\n  ID: {} | Node: {} | Author: {}{}\n",
        idx_str, topic.title, topic.id, node, author, replies
    )
}

/// Format topic details for text output
pub fn format_topic_detail(topic: &Topic) -> String {
    let mut output = String::new();

    output.push_str(&format!("Title: {}\n", topic.title));
    output.push_str(&format!("ID: {}\n", topic.id));
    output.push_str(&format!("Node: {}\n", topic.node_title()));
    output.push_str(&format!("Author: {}\n", topic.author_name()));
    output.push_str(&format!("Replies: {}\n", topic.replies));
    output.push_str(&format!("URL: {}\n", topic.url));

    if let Some(ref content) = topic.content {
        output.push_str("\n--- Content ---\n");
        output.push_str(content);
        output.push('\n');
    }

    output
}

/// Format a reply for text output
pub fn format_reply(reply: &Reply, index: Option<usize>) -> String {
    let idx_str = match index {
        Some(i) => format!("[{}] ", i + 1),
        None => String::new(),
    };

    let author = reply
        .member
        .as_ref()
        .map(|m| m.username.as_str())
        .unwrap_or("Unknown");

    let mut output = format!("{}Reply #{} by {}\n", idx_str, reply.id, author);

    if let Some(ref content) = reply.content {
        output.push_str(content);
        output.push('\n');
    }

    output
}

/// Format a notification for text output
pub fn format_notification(notification: &Notification, index: Option<usize>) -> String {
    let idx_str = match index {
        Some(i) => format!("[{}] ", i + 1),
        None => String::new(),
    };

    let author = notification
        .member
        .as_ref()
        .map(|m| m.username.as_str())
        .unwrap_or("Unknown");

    let mut output = format!(
        "{}Notification #{} from {}\n",
        idx_str, notification.id, author
    );
    output.push_str(&format!("  {}\n", notification.text));

    if let Some(ref payload) = notification.payload {
        if let Some(body) = payload.extract_body() {
            output.push_str("  ---\n");
            output.push_str(&format!("  {}\n", body));
        }
    }

    output
}

/// Format member profile for text output
pub fn format_member(member: &Member) -> String {
    let mut output = String::new();

    output.push_str(&format!("Username: {}\n", member.username));
    output.push_str(&format!("ID: {}\n", member.id));

    if let Some(ref tagline) = member.tagline {
        output.push_str(&format!("Tagline: {}\n", tagline));
    }

    if let Some(ref bio) = member.bio {
        output.push_str(&format!("Bio: {}\n", bio));
    }

    if let Some(ref location) = member.location {
        output.push_str(&format!("Location: {}\n", location));
    }

    if let Some(ref website) = member.website {
        output.push_str(&format!("Website: {}\n", website));
    }

    if let Some(ref github) = member.github {
        output.push_str(&format!("GitHub: {}\n", github));
    }

    if let Some(ref twitter) = member.twitter {
        output.push_str(&format!("Twitter: {}\n", twitter));
    }

    output.push_str(&format!("Created: {}\n", format_timestamp(member.created)));

    output
}

/// Format a node for text output
pub fn format_node(node: &(String, String), index: Option<usize>) -> String {
    let idx_str = match index {
        Some(i) => format!("[{}] ", i + 1),
        None => String::new(),
    };

    let (name, title) = node;
    let mut output = format!("{}{} ({})", idx_str, title, name);
    output.push('\n');
    output
}

/// Format RSS item for text output
pub fn format_rss_item(item: &RssItem, index: Option<usize>) -> String {
    let idx_str = match index {
        Some(i) => format!("[{}] ", i + 1),
        None => String::new(),
    };

    let topic_id = item.extract_topic_id();
    let id_str = topic_id
        .map(|id| format!(" (ID: {})", id))
        .unwrap_or_default();

    format!(
        "{}{}{}\n  Date: {}\n  URL: {}\n",
        idx_str, item.title, id_str, item.date, item.link
    )
}

/// Format timestamp to human-readable string
fn format_timestamp(ts: i64) -> String {
    use chrono::{DateTime, Utc};

    let dt = DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now());
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Find nodes by filter keyword
pub fn find_nodes(filter: Option<&str>, limit: Option<usize>) -> Vec<(String, String)> {
    let nodes = get_all_nodes();
    let mut result: Vec<(String, String)> = nodes.iter().cloned().collect();

    if let Some(f) = filter {
        let f_lower = f.to_lowercase();
        result.retain(|(name, title)| {
            name.to_lowercase().contains(&f_lower) || title.to_lowercase().contains(&f_lower)
        });
    }

    if let Some(l) = limit {
        result.truncate(l);
    }

    result
}

/// Print topics in text format
pub fn print_topics(topics: &[Topic], limit: Option<usize>) {
    let to_show = match limit {
        Some(n) if n < topics.len() => &topics[..n],
        _ => topics,
    };

    for (i, topic) in to_show.iter().enumerate() {
        print!("{}", format_topic(topic, Some(i)));
    }
}

/// Print topic detail in text format
pub fn print_topic_detail(topic: &Topic) {
    print!("{}", format_topic_detail(topic));
}

/// Print replies in text format
pub fn print_replies(replies: &[Reply], limit: Option<usize>) {
    let to_show = match limit {
        Some(n) if n < replies.len() => &replies[..n],
        _ => replies,
    };

    for (i, reply) in to_show.iter().enumerate() {
        print!("{}", format_reply(reply, Some(i)));
    }
}

/// Print notifications in text format
pub fn print_notifications(notifications: &[Notification], limit: Option<usize>) {
    let to_show = match limit {
        Some(n) if n < notifications.len() => &notifications[..n],
        _ => notifications,
    };

    for (i, notification) in to_show.iter().enumerate() {
        print!("{}", format_notification(notification, Some(i)));
    }
}

/// Print member profile in text format
pub fn print_member(member: &Member) {
    print!("{}", format_member(member));
}

/// Print nodes in text format
pub fn print_nodes(nodes: &[(String, String)], _limit: Option<usize>) {
    for (i, node) in nodes.iter().enumerate() {
        print!("{}", format_node(node, Some(i)));
    }
}

/// Print RSS items in text format
pub fn print_rss_items(items: &[RssItem], limit: Option<usize>) {
    let to_show = match limit {
        Some(n) if n < items.len() => &items[..n],
        _ => items,
    };

    for (i, item) in to_show.iter().enumerate() {
        print!("{}", format_rss_item(item, Some(i)));
    }
}
