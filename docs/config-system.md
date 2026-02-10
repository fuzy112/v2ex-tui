# Configuration System

## Overview

Configuration is written in Ketos Lisp and loaded from `~/.config/v2ex/config.lisp`.

## First-Run Behavior

On startup:

1. Check if `~/.config/v2ex/config.lisp` exists
2. If not, create directory and write embedded `default.lisp`
3. Show status: "Created default configuration at ~/.config/v2ex/config.lisp"
4. Load config from file

## File Locations

| File | Purpose |
|------|---------|
| `~/.config/v2ex/config.lisp` | User configuration (created on first run) |
| `config/default.lisp` (embedded) | Default configuration |
| `~/.config/v2ex/init.lisp` | Auto-loaded after config (optional) |

## Settings

### General

```lisp
(set! initial-view 'topic-list)           ; Startup view
(set! topics-per-page 20)                 ; Topics per API call
(set! replies-per-page 20)                ; Replies per API call
(set! auto-refresh-interval 0)            ; Auto-refresh (0 = off)
(set! key-sequence-timeout 1000)          ; Multi-key timeout (ms)
```

### Favorite Nodes

```lisp
(set-favorite-nodes
  '((python "Python")
    (programmer "程序员")
    (share "分享发现")))

(set-quick-keys
  '(1 python 2 programmer 3 share))
```

### Browser

```lisp
(set-browser #f)                          ; System default
(set-browser "firefox")                   ; Specific browser
(set-browser '("firefox" "--new-tab"))    ; With args
```

### Theme

```lisp
(set-theme 'dark)                         ; Preset: dark, light
(set-theme 'custom)                       ; Use custom colors

(set-custom-theme
  '((foreground "#ebdbb2")
    (background "#282828")
    (primary "#b8bb26")))
```

### Timestamp

```lisp
(set-timestamp-format 'relative)          ; "2 hours ago"
(set-timestamp-format 'absolute)          ; "2026-02-09 14:30"
(set-absolute-time-format "%Y-%m-%d %H:%M")
```

## Error Handling

If config has errors:

1. Show detailed error in status bar
2. Fall back to embedded defaults
3. Continue running (don't crash)
4. Allow fixing config and reloading

Example error:
```
Config error at line 42: Undefined variable 'nex-topic'. Did you mean 'next-topic'?
```

## Hot Reload

Reload configuration at runtime:

```lisp
;; In config or interactive
(reload-config)
```

Or bind to a key:
```lisp
(bind-global "C-c C-r" 'reload-config)
```

Reload process:
1. Reset keymaps to defaults
2. Re-parse config.lisp
3. Apply new configuration
4. Show success or error message

## Conditionals

Full Lisp conditionals supported:

```lisp
(when (eq system-type 'linux)
  (bind-global "M-return" 'linux-specific-action))

(if (getenv "TERM") 
    (set! terminal (getenv "TERM"))
    (set! terminal "xterm-256color"))
```

## User-Defined Functions

```lisp
(define (load-and-next)
  (load-more-topics)
  (next-topic))

(bind-global "C-c C-l" 'load-and-next)
```

## Open Questions

- Should we support config validation before applying?
- Should we create a config backup before reload?
- How to handle config migrations when adding new options?
