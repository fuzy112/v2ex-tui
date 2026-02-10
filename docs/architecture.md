# V2EX TUI Architecture

## Overview

V2EX TUI is a terminal-based V2EX client with configurable keymaps using an embedded Lisp interpreter (Ketos).

## Core Components

### 1. Keymap System (`src/keymap/`)

Handles all keyboard input through a hierarchical keymap system inspired by Emacs.

- **KeyMap**: Sparse keymap with parent pointer for inheritance
- **KeyMapChain**: Priority-based lookup across multiple keymaps
- **Binding**: Either an Action or a Prefix (for key sequences)
- **ActionRegistry**: Maps action names to Rust implementations

### 2. Configuration System (`src/config/`)

Uses Ketos Lisp interpreter for configuration files.

- **Config**: Struct holding all user settings
- **ConfigEngine**: Ketos interpreter with registered builtins
- **Loader**: File I/O and auto-generation of defaults
- **Reload**: Hot-reload functionality

### 3. Application State (`src/app.rs`)

- **App**: Main application state
- **View**: Enum of all views (TopicList, TopicDetail, etc.)
- **ModeStack**: Active minor modes that persist across views

## Data Flow

```
Keyboard Input
    ↓
KeyEvent (from crossterm)
    ↓
KeyMapChain.lookup()
    ↓
Binding::Action or Binding::Prefix
    ↓
ActionRegistry.execute()
    ↓
Application State Update
    ↓
UI Re-render (ratatui)
```

## Mode Persistence

Modes persist across view switches:

1. Enter link-selection mode in TopicDetail
2. Switch to TopicList (mode persists)
3. Switch back to TopicDetail (still in link-selection mode)
4. Press Esc to exit mode

This allows for workflows like:
- Enter link-selection mode
- Browse multiple topics while staying in link mode
- Exit when done

## Configuration Files

- `~/.config/v2ex/config.lisp` - User configuration (auto-generated on first run)
- `config/default.lisp` - Embedded defaults

## Design Principles

1. **Config-driven**: All behavior defined in Lisp, no hardcoded keys
2. **Hierarchical keymaps**: Global → View → Mode with inheritance
3. **Mode persistence**: Modes stay active across view switches
4. **Hot reload**: Configuration can be reloaded at runtime
5. **Error resilience**: Invalid config shows error but doesn't crash
