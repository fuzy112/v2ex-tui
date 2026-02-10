# V2EX TUI - TODO

## ✅ Current Status

- **main.rs**: 206 lines
- **app.rs**: 420 lines
- **View modularization**: Complete
- **Terminal/Browser abstraction**: Complete
- **Testing foundation**: 14 unit tests, CI pipeline
- **Code quality**: Passes `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
- **Dependencies**: Updated to latest versions
- **Documentation**: README.md with Vibe Coding warning

## 🔧 **Configuration System (Planned)**

**Add more Emacs-style keybinds:**
- [x] `r`, `l` for forward/backward (as in info) - ✅ DONE: History navigation implemented
- [x] `C-v`, `M-v` for page navigation - ✅ DONE: Page up/down with Ctrl+v and Alt+v
- [x] `SPC` for scrolling - ✅ DONE: Space key scrolls down in all views

**Configurable options:**
- [ ] Make keymaps configurable
- [ ] Make favorite nodes configurable
- [ ] Make the initial view configurable
- [ ] Make replies number loaded configurable
- [ ] Make count of loaded topics configurable
- [ ] Make the browser used to open links configurable
- [ ] Make color/theme configurable
- [ ] Make timestamp format configurable (absolute vs relative) - Currently shows relative time (e.g., "2 hours ago"), add option for absolute time (e.g., "2026-02-09 14:30")

## 📝 **Future Enhancements**

1. **Integration tests** - Add API mocking and end-to-end app flow tests
2. **Enhanced error handling** - Better user feedback and error recovery
3. **Performance optimization** - Caching, async improvements, and reduced redraws
4. **Additional features** - Search, bookmarking, offline reading
5. **Image display** - See section below

## 🖼️ **Image Display Enhancement**

**Current Implementation (2026-02-10):**
- Images in topic/reply content are extracted and replaced with markdown-style placeholders: `![Image](url)`
- Image placeholders appear inline in the text where images would be
- When in link selection mode (press `f`), images are labeled and selectable like regular links
- Selecting an image label opens the image URL in the default browser
- Format: `![Image](https://example.com/image.jpg)`

**Future Enhancement: Inline Image Rendering**
- [ ] Add `ratatui-image` and `image` crate dependencies
- [ ] Detect terminal graphics protocol support (Sixel, Kitty, iTerm2)
- [ ] Parse and download images asynchronously using existing `reqwest` client
- [ ] Display actual images inline in supported terminals (Kitty, iTerm2, Foot, Wezterm)
- [ ] Fallback to placeholder approach in unsupported terminals (Alacritty, Konsole, Warp)
- [ ] Handle image resizing, cropping, and scaling to fit terminal width
- [ ] Cache downloaded images to disk to avoid re-downloading
- [ ] Add configuration option to enable/disable inline images
- [ ] Test terminal compatibility matrix:
  - Sixel: xterm, foot, mlterm, Black Box
  - Kitty: Kitty, Ghostty
  - iTerm2: iTerm2, Wezterm, Rio, Bobcat
  - Unsupported: Alacritty, Konsole, Warp (keep placeholders)

**Technical Notes:**
- Use `Picker::from_query_stdio()` from ratatui-image to detect terminal capabilities
- Async image loading to avoid blocking UI (use existing tokio runtime)
- Consider memory usage with large images - implement size limits
- Handle network errors gracefully (fallback to placeholder if download fails)

## 🐛 **Code TODOs**

- [ ] **Link highlighting for replies** (`src/views/topic_detail.rs:277`) - Add link highlighting in replies when in link selection mode. Currently only topic content shows highlighted links.

## ✅ Validation Checklist

**Before committing:**
- [ ] `cargo fmt` formatted
- [ ] `cargo check` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo test` passes

---
*Last updated: 2026-02-09*
*Status: History navigation implemented, configuration system planned*
