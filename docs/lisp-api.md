# Lisp API Reference

## Settings Functions

### `(set! symbol value)`

Set a configuration variable.

```lisp
(set! initial-view 'topic-list)
(set! topics-per-page 20)
```

### `(set-favorite-nodes list)`

Set favorite nodes list.

```lisp
(set-favorite-nodes
  '((python "Python")
    (rust "Rust")))
```

### `(set-quick-keys list)`

Map keys 1-9 to nodes.

```lisp
(set-quick-keys '(1 python 2 rust 3 go))
```

### `(set-browser command)`

Set browser command. Use `#f` for system default.

```lisp
(set-browser #f)
(set-browser "firefox")
(set-browser '("firefox" "--new-tab"))
```

### `(set-theme theme)`

Set color theme: `'dark`, `'light`, or `'custom`.

```lisp
(set-theme 'dark)
```

### `(set-custom-theme alist)`

Define custom colors when theme is `'custom`.

```lisp
(set-custom-theme
  '((foreground "#ebdbb2")
    (background "#282828")))
```

### `(set-timestamp-format format)`

`'relative` or `'absolute`.

```lisp
(set-timestamp-format 'relative)
```

### `(set-absolute-time-format format)`

strftime format string for absolute timestamps.

```lisp
(set-absolute-time-format "%Y-%m-%d %H:%M")
```

### `(set-inline-images bool)`

Enable/disable inline image rendering (future feature).

```lisp
(set-inline-images #t)
```

### `(set-image-protocol protocol)`

`'auto`, `'sixel`, `'kitty`, `'iterm2`, or `'none`.

```lisp
(set-image-protocol 'auto)
```

## Keymap Functions

### `(bind-global key action)`

Bind key globally (all views).

```lisp
(bind-global "C-c" 'quit-immediate)
(bind-global "l" 'history-back)
```

### `(bind key action)`

Bind key in current view (inside `with-view`).

```lisp
(with-view 'topic-list
  (bind "n" 'next-topic))
```

### `(with-view view bindings...)`

Define view-specific keymap.

```lisp
(with-view 'topic-detail
  (bind "t" 'toggle-replies)
  (bind "o" 'open-in-browser))
```

### `(with-mode mode bindings...)`

Define minor mode keymap (inside `with-view` or standalone).

```lisp
(with-view 'topic-detail
  (with-mode 'replies
    (bind "n" 'next-reply)))
```

### `(define-key key-sequence action)`

Define multi-key sequence.

```lisp
(define-key "C-x C-s" 'save-config)
(define-key "C-x C-c" 'quit-immediate)
```

## Action Functions

### `(reload-config)`

Reload configuration from disk.

```lisp
(reload-config)
```

## System Functions

### `(getenv name)`

Get environment variable.

```lisp
(getenv "HOME")
(getenv "TERM")
```

### `(system-type)`

Get operating system: `'linux`, `'macos`, `'windows`, etc.

```lisp
(when (eq system-type 'linux)
  ...)
```

## Action Reference

All actions available for binding:

### Global Actions

| Action | Description |
|--------|-------------|
| `quit-immediate` | Exit app immediately |
| `remove-from-history` | Remove current view from history |
| `history-back` | Go to previous view in history |
| `history-forward` | Go to next view in history |
| `show-help` | Show help view |
| `refresh` | Refresh current view |
| `go-to-aggregate` | Navigate to aggregate view |
| `go-to-notifications` | Navigate to notifications view |
| `go-to-profile` | Navigate to profile view |
| `go-to-node-select` | Navigate to node selection view |

### Navigation Actions

| Action | Description |
|--------|-------------|
| `next-topic` | Next topic in list |
| `previous-topic` | Previous topic in list |
| `next-reply` | Next reply |
| `previous-reply` | Previous reply |
| `first-topic` | First topic |
| `last-topic` | Last topic |
| `first-reply` | First reply |
| `last-reply` | Last reply |
| `page-up` | Page up |
| `page-down` | Page down |

### Topic Actions

| Action | Description |
|--------|-------------|
| `open-topic` | Open selected topic |
| `toggle-replies` | Toggle replies visibility |
| `load-more-topics` | Load more topics |
| `load-more-replies` | Load more replies |
| `open-in-browser` | Open in default browser |
| `copy-to-clipboard` | Copy to clipboard |
| `enter-link-mode` | Enter link selection mode |
| `exit-link-mode` | Exit link selection mode |
| `link-select` | Select link by shortcut (takes string arg) |
| `switch-node` | Switch to node (takes string arg) |
| `select-node` | Open node selection view |

## Open Questions

- Should we provide `(unbind key)` function?
- Should actions support arguments beyond simple strings?
- Should we support action hooks (before/after)?
