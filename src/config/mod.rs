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

/// Load project config and merge user defaults into it.
/// This is the canonical way to obtain the effective config for generation
/// or display — every command that needs the "resolved" config should call this.
pub fn load_effective_config(path: &Path) -> Result<ProjectConfig> {
    if !path.exists() {
        return Err(Error::ConfigNotFound(path.to_path_buf()));
    }
    let content = std::fs::read_to_string(path)?;
    let mut config: ProjectConfig = toml::from_str(&content)?;

    if let Some(user_config) = load_user_config()? {
        // Parse the raw TOML table so merge can distinguish "field absent"
        // from "field explicitly set to the serde default value".
        let raw: toml::Value = toml::from_str(&content)
            .expect("already parsed successfully above");
        let raw_image = raw.get("image");
        merge::merge_configs(&mut config, &user_config, raw_image);
    }

    Ok(config)
}
