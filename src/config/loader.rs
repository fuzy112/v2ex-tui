//! Configuration loader with auto-generation

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::ConfigEngine;

/// Default configuration embedded in the binary
const DEFAULT_CONFIG: &str = include_str!("../../config/default.lisp");

/// Initialize configuration
///
/// If config file doesn't exist, creates it from defaults.
/// Returns the config engine and an optional message about auto-generation.
pub fn init_config() -> Result<(ConfigEngine, Option<String>)> {
    let config_dir = crate::config::Config::config_dir()?;
    let config_file = config_dir.join("config.lisp");

    let mut engine = ConfigEngine::new();
    let mut message = None;

    if !config_file.exists() {
        // Create config directory
        fs::create_dir_all(&config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;

        // Write default config
        fs::write(&config_file, DEFAULT_CONFIG).with_context(|| {
            format!(
                "Failed to write default config to: {}",
                config_file.display()
            )
        })?;

        message = Some(format!(
            "Created default configuration at {}",
            config_file.display()
        ));
    }

    // Load the config file
    engine.load_file(&config_file)?;

    Ok((engine, message))
}

/// Load configuration from a specific path
pub fn load_config_from(path: &Path) -> Result<ConfigEngine> {
    let mut engine = ConfigEngine::new();
    engine.load_file(path)?;
    Ok(engine)
}

/// Reload configuration
pub fn reload_config(engine: &mut ConfigEngine) -> anyhow::Result<()> {
    let config_file = crate::config::Config::config_file()?;

    // Reset the runtime config
    engine.runtime_config().borrow_mut().reset();

    // Reload the file
    engine.load_file(&config_file)?;

    Ok(())
}
