# V2EX TUI - Agent Guidelines

This document provides guidelines for AI agents working on the V2EX TUI project.

## Project Overview

V2EX TUI is a terminal-based viewer for the V2EX community using the V2EX API 2.0 Beta. It provides a TUI (Terminal User Interface) for browsing V2EX without a web browser.

## Development Environment

### Nix Development Shell
```bash
# Enter development environment
nix-shell

# Or using flake
nix develop
```

The development shell provides:
- Rust toolchain (rustc, cargo, rustfmt, clippy)
- Build dependencies (openssl, pkg-config)
- Development tools (git, ripgrep, fd)

### Available Commands (in nix-shell)
```bash
# Build commands
cargo build           # Build debug version
cargo build --release # Build optimized release
cargo run             # Build and run
./target/debug/v2ex-tui   # Run debug build
./target/release/v2ex-tui # Run release build

# Code quality
cargo fmt             # Format code with rustfmt
cargo clippy          # Run linter
cargo clippy -- -D warnings # Treat warnings as errors

# Testing
cargo test            # Run all tests
cargo test -- --nocapture # Run tests with output
cargo test test_name  # Run specific test

# Documentation
cargo doc --open      # Generate and open documentation
```

### Single Test Execution
```bash
# Run a specific test module
cargo test api::tests

# Run a specific test function
cargo test test_get_member

# Run tests with verbose output
cargo test -- --nocapture --test-threads=1
```

## Code Style Guidelines

### Import Order
Follow this import order (separated by blank lines):
1. Standard library imports
2. External crate imports
3. Internal module imports
4. Re-exports

Example:
```rust
use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
};
use anyhow::Result;

mod api;
mod ui;

use api::{V2exClient, Topic};
use ui::{Theme, render_topic_list};
```

### Naming Conventions
- **Types**: `PascalCase` (e.g., `V2exClient`, `AppState`)
- **Functions/Methods**: `snake_case` (e.g., `load_topics`, `render_ui`)
- **Variables**: `snake_case` (e.g., `selected_topic`, `status_message`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `BASE_URL`, `MAX_RETRIES`)
- **Modules**: `snake_case` (e.g., `api`, `ui`)

### Error Handling
- Use `anyhow::Result<T>` for functions that can fail
- Use `anyhow::Context` for adding context to errors
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

### Type Annotations
- Prefer explicit type annotations for public API
- Use type inference for local variables when clear
- Always annotate function return types

### Async/Await Patterns
- Use `#[tokio::main]` for main function
- Mark async functions with `async fn`
- Use `.await` for async operations
- Handle errors at await points

### Documentation
- Use `///` for public API documentation
- Include examples for complex functions
- Document error conditions
- Keep documentation up-to-date

Example:
```rust
/// Load topics from the current node
///
/// # Arguments
/// * `append` - If true, append to existing topics; if false, replace
///
/// # Errors
/// Returns an error if the API request fails or parsing fails
async fn load_topics(&mut self, append: bool) -> Result<()> {
    // implementation
}
```

### UI/State Management
- Keep UI state in `App` struct
- Use `ListState` for managing list selections
- Separate UI rendering from business logic
- Handle keyboard events consistently

### API Client Patterns
- Use `reqwest` for HTTP requests
- Implement proper error handling for API responses
- Handle rate limiting (600 requests per hour per IP)
- Cache responses when appropriate
- Handle inconsistent API response formats (notifications can be string or object)
- Handle pagination issues (replies pagination may not work correctly)
- Convert HTML content to plain text for display

## Project Structure

```
src/
├── main.rs      # Main application, event loop, state management
├── api.rs       # V2EX API client, data structures
└── ui.rs        # UI rendering functions, theme
```

### Key Components
1. **App State** (`main.rs`): Manages application state, view transitions, navigation
2. **API Client** (`api.rs`): Handles V2EX API requests, data serialization
3. **UI Rendering** (`ui.rs`): Renders TUI components, manages themes

## Testing Guidelines

### Unit Tests
- Place tests in the same file as the code being tested
- Use `#[cfg(test)]` attribute for test modules
- Test error conditions and edge cases

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_parsing() {
        // test code
    }
}
```

### Integration Tests
- Create separate test files for API integration
- Use environment variables for configuration
- Mock external dependencies when possible

### Running Tests
Always run tests after making changes:
```bash
cargo test
cargo clippy
cargo fmt --check
```

## Commit Guidelines

### Commit Messages
Follow conventional commits format:
```
type(scope): description

Body explaining the change in detail.

Fixes #issue
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Pre-commit Checks
Before committing:
1. Run `cargo fmt` to ensure consistent formatting
2. Run `cargo clippy` to catch lint issues
3. Run `cargo test` to ensure tests pass
4. Verify the application builds and runs

## Configuration

### Required Configuration
Users must create `~/.config/v2ex/token.txt` with their V2EX Personal Access Token.

### Environment Variables
- `RUST_BACKTRACE=1`: Enable backtraces for debugging
- `RUST_LOG=debug`: Enable debug logging (if logging is implemented)

## Common Tasks

### Adding New Features
1. Analyze existing patterns in similar features
2. Update `App` state if needed
3. Add UI rendering functions
4. Add keyboard event handling
5. Update help documentation
6. Test thoroughly

### Fixing Bugs
1. Reproduce the issue
2. Add test case if possible
3. Fix the root cause
4. Verify the fix doesn't break existing functionality
5. Update documentation if needed

### Refactoring
1. Ensure tests pass before refactoring
2. Make incremental changes
3. Keep the public API stable
4. Update documentation
5. Run all tests after refactoring

## V2EX API 2.0 Beta Details

### Base URL
```
https://www.v2ex.com/api/v2/
```

### Authentication
- Use Personal Access Token in Authorization header
- Format: `Authorization: Bearer <token>`
- Token stored in `~/.config/v2ex/token.txt`

### Rate Limits
- 600 requests per hour per IP
- Check headers: `X-Rate-Limit-Limit`, `X-Rate-Limit-Remaining`, `X-Rate-Limit-Reset`
- CDN-cached requests only count on first request

### Available Endpoints
1. **GET /notifications** - Get latest notifications (pagination: `?p=`)
2. **DELETE /notifications/:id** - Delete specific notification
3. **GET /member** - Get user profile
4. **GET /token** - Get current token info
5. **POST /tokens** - Create new token
6. **GET /nodes/:name** - Get node info
7. **GET /nodes/:name/topics** - Get topics in node (pagination: `?p=`)
8. **GET /topics/:id** - Get topic details
9. **GET /topics/:id/replies** - Get topic replies (pagination: `?p=`)

### Important Notes
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