# V2EX TUI - Agent Guidelines

Guidelines for AI agents working on V2EX TUI, a terminal-based viewer for V2EX using API 2.0 Beta.

## Development Commands

### Nix Development Shell
```bash
nix-shell        # Enter development environment
nix develop      # Alternative using flake
```

### Code Quality
```bash
cargo fmt             # Format with rustfmt
cargo clippy          # Lint (treat warnings as errors: cargo clippy -- -D warnings)
cargo check           # Compile check
```

**IMPORTANT**: ALWAYS run `cargo fmt` and `cargo check` before committing code. This ensures:
1. Code is properly formatted according to Rust conventions
2. No compilation errors are introduced
3. Code style consistency is maintained across the project

## Code Style

### Imports
Group imports with blank lines between:
```rust
use std::io;
use crossterm::{event::{self, Event, KeyCode}, execute};
use anyhow::Result;

mod api;
mod ui;
mod nodes;

use api::{V2exClient, Topic};
use ui::{Theme, render_topic_list};
```

### Naming
- **Types**: `PascalCase` (`V2exClient`, `AppState`)
- **Functions/Methods**: `snake_case` (`load_topics`, `render_ui`)
- **Variables**: `snake_case` (`selected_topic`, `status_message`)
- **Constants**: `SCREAMING_SNAKE_CASE` (`BASE_URL`, `MAX_RETRIES`)
- **Modules**: `snake_case` (`api`, `ui`, `nodes`)

### Error Handling
Use `anyhow::Result<T>` with context:
```rust
use anyhow::{Result, Context};

async fn load_topics(&self) -> Result<Vec<Topic>> {
    let response = self.client
        .get(&format!("{}/topics", BASE_URL))
        .await
        .context("Failed to fetch topics")?;
    
    response.json::<Vec<Topic>>()
        .await
        .context("Failed to parse response")
}
```

### Async & Types
- Annotate public API return types
- Use `#[tokio::main]` for main
- Mark async functions with `async fn`
- Handle errors at `.await` points

### Dead Code Handling
When using `#[allow(dead_code)]` attributes, ALWAYS include a comment explaining why the code is kept:
- **Legacy functions**: Mark as `// Legacy function replaced by <NewModule/NewFunction>`
- **API completeness**: Mark as `// Not currently used, but kept for API completeness`
- **Future features**: Mark as `// Reserved for future feature: <description>`
- **Testing utilities**: Mark as `// Test utility, used in integration tests`

**Examples:**
```rust
#[allow(dead_code)] // Legacy function replaced by TopicListView
pub fn render_topic_list(...)

#[allow(dead_code)] // Error variant not currently used, but kept for completeness
pub enum BrowserResult { ... }

#[allow(dead_code)] // Utility function not currently used, but kept for completeness
pub fn with_terminal(...)
```

**Cleanup policy:** Legacy functions marked with `allow(dead_code)` should be removed after confirming new implementations are stable and tested.

## Project Structure
```
src/
├── main.rs      # App state, event loop, navigation
├── api.rs       # V2EX API client, data structures
├── app.rs       # App struct and data loading methods
├── state.rs     # State management (TopicState, NodeState, etc.)
├── keymap.rs    # Modular key mappings for different views
├── clipboard.rs # Clipboard utilities (OSC 52)
├── browser.rs   # Browser integration
├── terminal.rs  # Terminal management
├── ui.rs        # UI rendering, theme management
├── views/       # View components
│   ├── topic_list.rs
│   ├── topic_detail.rs
│   ├── notifications.rs
│   ├── profile.rs
│   ├── help.rs
│   ├── node_select.rs
│   └── aggregate.rs
└── nodes.rs     # 1333 nodes for autocompletion
```

## Key Patterns

### UI/State Management
- State in `App` struct, `ListState` for selections
- Emacs/dired style keys: n/p (down/up), h/l (back/forward)

### API Client
- `Authorization: Bearer <token>` (from `~/.config/v2ex/token.txt`)
- Rate limit: 600 requests/hour per IP
- Handle inconsistent formats (notifications: string or object)

### Node Autocompletion
- Press `s` for direct completing-read mode
- Fuzzy matching with `fuzzy-matcher` (SkimMatcherV2)
- 1333 nodes from `nodes.rs`
- Tab toggles manual input/selection mode
- Quick keys 1-9 for favorite nodes

### Keyboard Shortcuts
```
n/p (↓/↑) - Next/previous item
h/l (←/→) - Back/forward, Enter/l - Open
s         - Node selection mode
1-9       - Quick nodes (python, programmer, share, create, jobs, go, rust, javascript, linux)
t         - Toggle topic/replies view
o         - Open in browser
m/u       - Notifications/Profile
a         - Aggregated topics (RSS feeds)
+         - Load more topics/replies
g         - Refresh current view
f         - Enter link selection mode (topic detail)
w         - Copy selected reply to clipboard (topic detail)
?         - Help, q/Esc - Exit/back
```

## Common Tasks
- **Add Feature**: Analyze existing patterns, update `App` state, add UI functions in `ui.rs`, add key handling in `keymap.rs`, update `views/help.rs`
- **Fix Bug**: Reproduce issue, add test if possible, fix root cause, verify no regression
- **Refactor**: Ensure `cargo check` passes first, incremental changes, run `cargo fmt`, `cargo clippy`, `cargo check` after

**Commit Protocol**: Before creating any commit:
1. Run `cargo fmt` to format code
2. Run `cargo check` to ensure no compilation errors
3. Run `cargo clippy` to check for warnings (optional but recommended)
4. Only commit if all checks pass

## V2EX API 2.0 Beta
```
https://www.v2ex.com/api/v2/
Authorization: Bearer <token>
```
- `GET /member` - User profile
- `GET /notifications` - Notifications  
- `GET /nodes/:name/topics` - Node topics
- `GET /topics/:id` - Topic detail
- `GET /topics/:id/replies` - Topic replies

**Caveats**: Replies pagination may not work, notification payload varies (string/object), inconsistent data structures.

## Related Links
- **V2EX Official Blog RSS**: https://blog.v2ex.com/rss/ - For updates and announcements about V2EX platform

Always follow existing patterns and maintain consistency.

## Documentation

- TODO.md:  Planned but not yet implemented features
- docs/*.md:  Design for planned features
