# Keymap System Design

## Sparse Keymap (Emacs-style)

```rust
pub struct KeyMap {
    /// Local bindings (overrides parent)
    bindings: HashMap<KeyEvent, Binding>,
    
    /// Parent keymap (for inheritance)
    parent: Option<Rc<KeyMap>>,
}

impl KeyMap {
    /// Lookup with parent fallback
    fn lookup(&self, key: &KeyEvent) -> Option<&Binding> {
        self.bindings.get(key)
            .or_else(|| self.parent.as_ref()?.lookup(key))
    }
}
```

## KeyMap Chain (Priority)

```rust
pub struct KeyMapChain {
    /// Ordered by priority (highest first)
    /// 0: Temporary keymap (key sequences)
    /// 1: Active minor modes
    /// 2: View-specific keymap
    /// 3: Global keymap
    keymaps: Vec<Rc<KeyMap>>,
}
```

## Key Notation

Emacs-style notation for configuration:

| Notation | Meaning |
|----------|---------|
| `a` | Plain 'a' key |
| `C-a` | Ctrl+a |
| `M-a` | Alt+a |
| `S-a` | Shift+a (or just 'A') |
| `C-x C-s` | Key sequence: Ctrl+x, then Ctrl+s |
| `C-c C-c` | Common exit sequence |
| `ESC` | Escape key |
| `RET` | Return/Enter |
| `SPC` | Space |
| `TAB` | Tab |
| `DEL` | Delete |
| `BACKSPACE` | Backspace |

## Multi-Key Sequences

When a prefix key is pressed (e.g., `C-x`):

1. Push temporary keymap for remaining keys
2. Show "C-x-" in status bar
3. Next key looked up in temporary map only
4. On match: execute action, pop temporary map
5. On timeout (configurable, default: 1s): cancel, show error, pop map
6. On mismatch: cancel, show error, pop map

Timeout is configurable via `(set! key-sequence-timeout 1000)` in milliseconds.

## Mode System

### Major Modes (Views)

Each view has a major mode keymap:
- `topic-list`
- `topic-detail`
- `notifications`
- `profile`
- `aggregate`
- `node-select`
- `help`

### Minor Modes

Can be active across multiple views:
- `replies` - When replies visible in topic-detail
- `link-selection` - Modal link selection
- `completion` - When in completion mode

### Mode Stack

```rust
pub struct ModeStack {
    /// Active minor modes (persist across views)
    active_modes: Vec<String>,
    
    /// Current major mode (view)
    major_mode: String,
}
```

## Example Configuration

```lisp
;; Global bindings (available everywhere)
(bind-global "C-c" 'quit-immediate)

;; View-specific
(with-view 'topic-detail
  ;; Base bindings
  (bind "t" 'toggle-replies)
  
  ;; Minor mode: replies visible
  (with-mode 'replies
    (bind "n" 'next-reply)  ; Overrides view's "n"
    (bind "p" 'previous-reply))
  
  ;; Modal: link selection
  (with-mode 'link-selection
    (bind "a" '(link-select "a"))
    (bind "Esc" 'exit-link-mode)))

;; Key sequences
(define-key "C-x C-s" 'save-config)
```

## Open Questions

- Should we support key sequence abortion with C-g?
- Should prefix keys be configurable or fixed?
- How to handle conflicts between minor modes?
