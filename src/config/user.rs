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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_user_config() {
        let config: UserConfig = toml::from_str("").unwrap();
        assert!(config.defaults.base.is_none());
        assert!(config.defaults.base_version.is_none());
        assert!(config.defaults.shell.is_none());
        assert!(config.defaults.platform.is_none());
        assert!(config.defaults.modules.is_empty());
    }

    #[test]
    fn parse_full_user_config() {
        let toml_str = r#"
[defaults]
base = "alpine"
base_version = "3.21"
shell = "zsh"
platform = "linux/arm64"

[defaults.modules]
nodejs = { version = "20" }
python = { version = "3.12" }
"#;
        let config: UserConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.defaults.base, Some(BaseOs::Alpine));
        assert_eq!(config.defaults.base_version.as_deref(), Some("3.21"));
        assert_eq!(config.defaults.shell, Some(ShellType::Zsh));
        assert_eq!(config.defaults.platform.as_deref(), Some("linux/arm64"));
        assert_eq!(config.defaults.modules.len(), 2);
        let node = config.defaults.modules["nodejs"].as_table().unwrap();
        assert_eq!(node["version"].as_str(), Some("20"));
    }

    #[test]
    fn parse_user_config_partial_defaults() {
        let toml_str = r#"
[defaults]
base = "debian"
"#;
        let config: UserConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.defaults.base, Some(BaseOs::Debian));
        assert!(config.defaults.shell.is_none());
        assert!(config.defaults.platform.is_none());
        assert!(config.defaults.modules.is_empty());
    }

    #[test]
    fn user_config_with_only_modules() {
        let toml_str = r#"
[defaults.modules]
rust = {}
golang = { version = "1.22" }
"#;
        let config: UserConfig = toml::from_str(toml_str).unwrap();
        assert!(config.defaults.base.is_none());
        assert_eq!(config.defaults.modules.len(), 2);
    }

    #[test]
    fn user_config_default_trait() {
        let config = UserConfig::default();
        assert!(config.defaults.base.is_none());
        assert!(config.defaults.base_version.is_none());
        assert!(config.defaults.shell.is_none());
        assert!(config.defaults.platform.is_none());
        assert!(config.defaults.modules.is_empty());
    }

    #[test]
    fn user_defaults_default_trait() {
        let defaults = UserDefaults::default();
        assert!(defaults.base.is_none());
        assert!(defaults.base_version.is_none());
        assert!(defaults.shell.is_none());
        assert!(defaults.platform.is_none());
        assert!(defaults.modules.is_empty());
    }

    #[test]
    fn user_config_all_shell_types() {
        for (shell_str, expected) in [
            ("bash", ShellType::Bash),
            ("zsh", ShellType::Zsh),
            ("sh", ShellType::Sh),
        ] {
            let toml_str = format!(
                r#"
[defaults]
shell = "{shell_str}"
"#
            );
            let config: UserConfig = toml::from_str(&toml_str).unwrap();
            assert_eq!(config.defaults.shell, Some(expected));
        }
    }

    #[test]
    fn user_config_all_base_os() {
        for (os_str, expected) in [
            ("ubuntu", BaseOs::Ubuntu),
            ("debian", BaseOs::Debian),
            ("alpine", BaseOs::Alpine),
        ] {
            let toml_str = format!(
                r#"
[defaults]
base = "{os_str}"
"#
            );
            let config: UserConfig = toml::from_str(&toml_str).unwrap();
            assert_eq!(config.defaults.base, Some(expected));
        }
    }

    #[test]
    fn user_config_invalid_base_os_fails() {
        let toml_str = r#"
[defaults]
base = "fedora"
"#;
        let result: Result<UserConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }
}
