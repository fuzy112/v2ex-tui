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
- `C-v`, `M-v` for page navigation
- `SPC` for scrolling
- `r`, `l` for forward/backward (as in info)

**Configurable options:**
- [ ] Make keymaps configurable
- [ ] Make favorite nodes configurable
- [ ] Make the initial view configurable
- [ ] Make replies number loaded configurable
- [ ] Make count of loaded topics configurable
- [ ] Make the browser used to open links configurable
- [ ] Make color/theme configurable

## 📝 **Future Enhancements**

1. **Integration tests** - Add API mocking and end-to-end app flow tests
2. **Enhanced error handling** - Better user feedback and error recovery
3. **Performance optimization** - Caching, async improvements, and reduced redraws
4. **Additional features** - Search, bookmarking, offline reading

## ✅ Validation Checklist

**Before committing:**
- [ ] `cargo fmt` formatted
- [ ] `cargo check` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo test` passes

---
*Last updated: 2025-02-09*
*Status: Core features complete, configuration system planned*
