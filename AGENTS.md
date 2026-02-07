# V2EX TUI - Agent Guidelines

Guidelines for AI agents working on V2EX TUI, a terminal-based viewer for V2EX using API 2.0 Beta.

## Development Commands

### Nix Development Shell
```bash
nix-shell        # Enter development environment
nix develop      # Alternative using flake
```

### Build & Run
```bash
cargo build           # Debug build
cargo build --release # Release build
cargo run             # Build and run
```

### Code Quality
```bash
cargo fmt             # Format with rustfmt
cargo clippy          # Lint (treat warnings as errors: cargo clippy -- -D warnings)
cargo check           # Compile check
```

### Testing
```bash
cargo test            # Run all tests (limited coverage)
cargo test -- --nocapture --test-threads=1  # Show test output

# Run single test
cargo test test_function_name
cargo test module_name::test_function
cargo test module_name  # All tests in module
```

### Documentation
```bash
cargo doc --open      # Generate and open docs
```

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

## Project Structure
```
src/
├── main.rs      # App state, event loop, navigation (~75 fields)
├── api.rs       # V2EX API client, data structures
├── ui.rs        # UI rendering, theme management
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
+         - Load more topics
g         - Refresh current view
?         - Help, q/Esc - Exit/back
```

## Common Tasks
- **Add Feature**: Analyze existing patterns, update `App` state, add UI functions in `ui.rs`, add key handling in `main.rs`, update `ui.rs::render_help()`
- **Fix Bug**: Reproduce issue, add test if possible, fix root cause, verify no regression
- **Refactor**: Ensure `cargo check` passes first, incremental changes, run `cargo fmt`, `cargo clippy`, `cargo check` after

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

Always follow existing patterns and maintain consistency.
