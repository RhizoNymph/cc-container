use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::project::{BaseOs, ShellType};

/// User-level defaults (~/.config/cc-container/config.toml).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    #[serde(default)]
    pub defaults: UserDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserDefaults {
    #[serde(default)]
    pub base: Option<BaseOs>,
    #[serde(default)]
    pub base_version: Option<String>,
    #[serde(default)]
    pub shell: Option<ShellType>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub modules: IndexMap<String, toml::Value>,
}
