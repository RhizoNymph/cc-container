pub mod merge;
pub mod project;
pub mod user;
pub mod validate;

pub use project::ProjectConfig;
pub use user::UserConfig;

use crate::error::{Error, Result};
use std::path::Path;

/// Load project config from a file path.
pub fn load_project_config(path: &Path) -> Result<ProjectConfig> {
    if !path.exists() {
        return Err(Error::ConfigNotFound(path.to_path_buf()));
    }
    let content = std::fs::read_to_string(path)?;
    let config: ProjectConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Load user config from the default location (~/.config/cc-container/config.toml).
/// Returns None if the file doesn't exist.
pub fn load_user_config() -> Result<Option<UserConfig>> {
    let Some(config_dir) = dirs::config_dir() else {
        return Ok(None);
    };
    let path = config_dir.join("cc-container").join("config.toml");
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let config: UserConfig = toml::from_str(&content)?;
    Ok(Some(config))
}
