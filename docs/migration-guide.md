# Migration Guide: Hardcoded → Configurable

## Overview

This guide helps migrate from the old hardcoded keymap system to the new configurable system.

## Breaking Changes

- All keybindings are now defined in `~/.config/v2ex/config.lisp`
- No hardcoded fallback keys (config is required)
- First run auto-generates default config

## Migration Steps

### 1. First Run

On first run, the application will:
- Create `~/.config/v2ex/`
- Write `config.lisp` with default keybindings
- Show message: "Created default configuration"

### 2. Customize Config

Edit `~/.config/v2ex/config.lisp`:

```lisp
;; Example: Change navigation keys to Vim-style
(with-view 'topic-list
  (bind "j" 'next-topic)
  (bind "k" 'previous-topic)
  (unbind "n")  ; Remove old binding
  (unbind "p"))
```

### 3. Reload Config

Either:
- Restart application
- Press `C-c C-r` (if bound) to reload
- Or run `(reload-config)` in future REPL

## Common Customizations

### Vim-style Navigation

```lisp
;; In all views
(bind-global "j" 'next-item)
(bind-global "k" 'previous-item)
(bind-global "h" 'history-back)
(bind-global "l" 'history-forward)

;; Unbind Emacs-style
(unbind-global "n")
(unbind-global "p")
(unbind-global "C-n")
(unbind-global "C-p")
```

### Custom Theme

```lisp
(set-theme 'custom)
(set-custom-theme
  '((foreground "#ebdbb2")
    (background "#282828")
    (primary "#b8bb26")
    (secondary "#fabd2f")))
```

### Different Browser per Link Type

```lisp
;; (Future feature)
(define (smart-open)
  (if (image-url? (current-link))
      (open-with "gimp")
      (open-in-browser)))

(bind-global "o" 'smart-open)
```

## Troubleshooting

### Config not loading

Check:
- File exists: `~/.config/v2ex/config.lisp`
- Valid Lisp syntax
- Check error message in status bar

### Missing keybindings

The default config includes all standard bindings. If something is missing:
1. Check `config/default.lisp` for reference
2. Add to your `config.lisp`
3. Reload

### Reset to defaults

Delete `~/.config/v2ex/config.lisp` and restart - defaults will be regenerated.

## Open Questions

- Should we provide a migration tool to convert old configs?
- Should we version the config format for future migrations?
