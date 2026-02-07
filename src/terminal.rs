use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

/// RAII wrapper for terminal management
pub struct TerminalManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalManager {
    /// Initialize terminal with raw mode and alternate screen
    pub fn new() -> Result<Self> {
        enable_raw_mode().context("Failed to enable raw mode")?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .context("Failed to initialize terminal")?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;

        Ok(Self { terminal })
    }

    /// Access underlying terminal for drawing
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    /// Clean shutdown with automatic restoration
    pub fn shutdown(&mut self) -> Result<()> {
        disable_raw_mode().context("Failed to disable raw mode")?;

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .context("Failed to restore terminal")?;

        self.terminal
            .show_cursor()
            .context("Failed to show cursor")?;

        Ok(())
    }
}

impl Drop for TerminalManager {
    /// Ensure terminal is restored even on panic
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Run a closure with terminal management
#[allow(dead_code)] // Utility function not currently used, but kept for completeness
pub fn with_terminal<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut Terminal<CrosstermBackend<Stdout>>) -> Result<R>,
{
    let mut manager = TerminalManager::new()?;
    let result = f(manager.terminal());
    manager.shutdown()?;
    result
}
