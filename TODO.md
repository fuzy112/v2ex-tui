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
- [ ] `SPC` for scrolling

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
