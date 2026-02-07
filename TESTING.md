# V2EX TUI - Testing Guide

## Overview

This document describes the testing strategy and setup for V2EX TUI.

## Test Structure

### Unit Tests
- **Location**: Inline within source files using `#[cfg(test)]` modules
- **State tests**: `src/state.rs` - Tests for state management
- **View tests**: `src/views/*.rs` - Tests for view rendering components
- **Run**: `cargo test`

### Integration Tests
- **Location**: `tests/integration/`
  - `api_mock.rs`: Mock API response tests
  - `app_flow.rs`: End-to-end application flow tests
- **Run**: `cargo test --test integration`

### Unit Test Directories
- `tests/unit/views/`: Additional view-related tests
- `tests/unit/state/`: Additional state-related tests

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test Categories
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# Single test module
cargo test state::tests::test_topic_state_next_topic
```

### Code Quality Checks
```bash
# Format check
cargo fmt --check

# Lint check
cargo clippy -- -D warnings

# Compilation check
cargo check
```

## Continuous Integration

GitHub Actions CI is configured in `.github/workflows/ci.yml`. It runs on every push to master and on pull requests, executing:

1. Format check (`cargo fmt --check`)
2. Lint check (`cargo clippy -- -D warnings`)
3. Unit and integration tests (`cargo test`)
4. Release compilation check (`cargo check --release`)

## Test Coverage

Current test coverage includes:
- State management (8 tests)
- View components (6 tests)
- Total: 14 tests

## Adding New Tests

### Unit Tests
Add tests within the source file using:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_functionality() {
        // Test logic
    }
}
```

### Integration Tests
Create new files in `tests/integration/` or `tests/unit/` directories.

## Mock API Testing

The `tests/integration/api_mock.rs` file demonstrates how to mock V2EX API responses. In a real testing scenario, you would use a mocking library like `mockito` or set up a mock server.

## Future Testing Improvements

1. **UI Testing**: Add tests for UI rendering using terminal emulation
2. **API Mocking**: Implement comprehensive API mocking with `mockito`
3. **Snapshot Testing**: Add snapshot tests for view rendering output
4. **Performance Tests**: Add benchmarks for rendering performance
5. **Cross-platform Testing**: Test on different platforms (Linux, macOS, Windows)