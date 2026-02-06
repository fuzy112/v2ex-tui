# V2EX TUI - Agent Guidelines

This document provides guidelines for AI agents working on the V2EX TUI project.

## Project Overview

V2EX TUI is a terminal-based viewer for the V2EX community using V2EX API 2.0 Beta. It provides a keyboard-driven TUI for browsing topics, notifications, and user profiles.

## Development Environment

### Nix Development Shell
```bash
nix-shell        # Enter development environment
nix develop      # Alternative using flake
```

The development shell provides Rust toolchain (rustc, cargo, rustfmt, clippy), build dependencies (openssl, pkg-config), and development tools (git, ripgrep, fd).

### Available Commands (in nix-shell)
```bash
# Build
cargo build           # Build debug version
cargo build --release # Build optimized release
cargo run             # Build and run main app
./target/debug/v2ex-tui   # Run debug build
./target/release/v2ex-tui # Run release build

# Code quality
cargo fmt             # Format code with rustfmt
cargo clippy          # Run linter (treat warnings as errors: cargo clippy -- -D warnings)
cargo check           # Compile check without building

# Testing
cargo test            # Run all tests (limited unit test coverage)
cargo test -- --nocapture --test-threads=1  # Run tests with output

# Test API binary (integration testing)
mv test_api.rs src/bin/test_api.rs   # Move to bin directory first
cargo run --bin test_api             # Run API integration tests

# Documentation
cargo doc --open      # Generate and open documentation
```

### Single Test Execution
```bash
cargo test test_name          # Run specific test function
cargo test module_name::tests # Run specific test module
```

## Code Style Guidelines

### Import Order
1. Standard library imports
2. External crate imports (grouped by crate)
3. Internal module declarations
4. Re-exports

Separate groups with blank lines. Example:
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

### Naming Conventions
- **Types**: `PascalCase` (`V2exClient`, `AppState`)
- **Functions/Methods**: `snake_case` (`load_topics`, `render_ui`)
- **Variables**: `snake_case` (`selected_topic`, `status_message`)
- **Constants**: `SCREAMING_SNAKE_CASE` (`BASE_URL`, `MAX_RETRIES`)
- **Modules**: `snake_case` (`api`, `ui`, `nodes`)

### Error Handling
- Use `anyhow::Result<T>` for fallible functions
- Use `anyhow::Context` to add context to errors
- Propagate errors with `?` operator
- Provide meaningful error messages

Example:
```rust
use anyhow::{Result, Context};

async fn load_topics(&self) -> Result<Vec<Topic>> {
    let response = self.client
        .get(&format!("{}/topics", BASE_URL))
        .await
        .context("Failed to fetch topics")?;
    
    response.json::<Vec<Topic>>()
        .await
        .context("Failed to parse topics response")
}
```

### Type Annotations & Async
- Prefer explicit type annotations for public API
- Use type inference for local variables when clear
- Always annotate function return types
- Use `#[tokio::main]` for main function
- Mark async functions with `async fn`
- Handle errors at `.await` points

### Documentation
- Use `///` for public API documentation
- Include examples for complex functions
- Document error conditions
- Keep documentation up-to-date

## Project Structure

```
src/
├── main.rs      # Main application, event loop, state management
├── api.rs       # V2EX API client, data structures, serialization
├── ui.rs        # UI rendering functions, theme management
└── nodes.rs     # All V2EX nodes (1333 entries) for autocompletion
```

### Key Components
1. **App State** (`main.rs`): Manages application state, view transitions, navigation, node autocompletion
2. **API Client** (`api.rs`): Handles V2EX API requests, data serialization, error handling
3. **UI Rendering** (`ui.rs`): Renders TUI components, manages themes
4. **Node Database** (`nodes.rs`): Complete node list for fuzzy-matching autocompletion

## Key Patterns

### UI/State Management
- Keep UI state in `App` struct (currently ~75 fields)
- Use `ListState` for managing list selections
- Separate UI rendering from business logic
- Handle keyboard events consistently (Emacs/dired style: n/p, h/l)

### API Client Patterns
- Use `reqwest` for HTTP requests with `Authorization: Bearer <token>`
- Handle rate limiting (600 requests per hour per IP)
- Cache responses when appropriate
- Handle inconsistent API response formats (notifications can be string or object)
- Convert HTML content to plain text for display using `html2text`

### Node Autocompletion
- Direct completing-read mode: Press `s` to enter search/filter mode
- Uses `fuzzy-matcher` crate with SkimMatcherV2 for fuzzy matching
- 1333 nodes from `nodes.rs` (generated from V2EX planes page using `extract_nodes.py` and `generate_nodes_rs.py`)
- Real-time suggestions update as you type, showing top 20 matches
- Quick keys 1-9 still work for favorite nodes
- Tab toggles between manual input and selection mode

### Keyboard Shortcuts (Current)
```
n/p (↓/↑) - Next/previous item
h/l (←/→) - Back/forward navigation
Enter/l   - Open selected item
s         - Node selection (direct completing-read mode)
1-9       - Quick node switches (python, programmer, share, create, jobs, go, rust, javascript, linux)
t         - Toggle topic/replies view
o         - Open in browser
m         - Notifications, u - Profile
+         - Load more topics, </> - First/last item
N/P       - Next/previous topic in detail view
r         - Refresh current view
?         - Help, q/Esc - Exit/back
```

## Common Tasks

### Adding New Features
1. Analyze existing patterns in similar features
2. Update `App` state if needed
3. Add UI rendering functions in `ui.rs`
4. Add keyboard event handling in `main.rs` key match block
5. Update help documentation in `ui.rs::render_help`
6. Test thoroughly

### Fixing Bugs
1. Reproduce the issue
2. Add test case if possible (currently limited test coverage)
3. Fix the root cause
4. Verify fix doesn't break existing functionality
5. Update documentation if needed

### Refactoring
1. Ensure `cargo check` passes before refactoring
2. Make incremental changes
3. Keep the public API stable
4. Update documentation
5. Run `cargo fmt`, `cargo clippy`, `cargo check` after changes

## V2EX API 2.0 Beta Notes

### Base URL & Authentication
```
https://www.v2ex.com/api/v2/
Authorization: Bearer <token> (stored at ~/.config/v2ex/token.txt)
```

### Rate Limits & Endpoints
- 600 requests per hour per IP
- Check headers: `X-Rate-Limit-Limit`, `X-Rate-Limit-Remaining`, `X-Rate-Limit-Reset`
- Key endpoints: `/member`, `/notifications`, `/nodes/:name/topics`, `/topics/:id`, `/topics/:id/replies`

### Important Caveats
- Replies pagination may not work correctly (known issue)
- Notification payload format varies (string or object)
- Member API returns minimal avatar URLs (mini size)
- API responses may have inconsistent data structures

## Resources

- [V2EX API 2.0 Beta Documentation](https://www.v2ex.com/help/api)
- [V2EX API Discussion Node](https://v2ex.com/go/v2exapi)
- [Ratatui Documentation](https://docs.rs/ratatui)
- [Reqwest Documentation](https://docs.rs/reqwest)
- [Anyhow Documentation](https://docs.rs/anyhow)

Remember: Always follow existing patterns in the codebase and maintain consistency with the project's style and architecture.
