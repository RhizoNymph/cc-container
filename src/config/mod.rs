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
        let raw: toml::Value = toml::from_str(&content)?;
        let raw_image = raw.get("image");
        merge::merge_configs(&mut config, &user_config, raw_image);
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const MINIMAL_CONFIG: &str = r#"
[project]
name = "test-project"
[agent]
type = "claude"
"#;

    // ───────────────────── load_project_config ─────────────────────

    #[test]
    fn load_project_config_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cc-container.toml");
        std::fs::write(&path, MINIMAL_CONFIG).unwrap();
        let config = load_project_config(&path).unwrap();
        assert_eq!(config.project.name, "test-project");
    }

    #[test]
    fn load_project_config_not_found() {
        let path = std::path::Path::new("/nonexistent/cc-container.toml");
        let err = load_project_config(path).unwrap_err();
        match err {
            Error::ConfigNotFound(p) => assert_eq!(p, path),
            other => panic!("Expected ConfigNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn load_project_config_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "this is not valid toml {{{{").unwrap();
        let err = load_project_config(&path).unwrap_err();
        assert!(matches!(err, Error::ConfigParse(_)));
    }

    #[test]
    fn load_project_config_valid_toml_but_wrong_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wrong.toml");
        std::fs::write(&path, "[something]\nkey = \"value\"\n").unwrap();
        let err = load_project_config(&path).unwrap_err();
        assert!(matches!(err, Error::ConfigParse(_)));
    }

    #[test]
    fn load_project_config_full() {
        let toml_str = r#"
[project]
name = "full"
description = "A full config"

[agent]
type = "both"
claude_version = "1.0"
codex_version = "2.0"

[image]
base = "debian"
base_version = "bookworm"
platform = "linux/arm64"
tag = "v1"
user = "coder"
shell = "zsh"

[modules]
nodejs = { version = "20" }

[auth.claude]
method = "api-key"
[auth.codex]
method = "oauth"

[firewall]
enabled = true
allowed_domains = ["github.com"]
allowed_cidrs = ["10.0.0.0/8"]

[workspace]
mount_path = "/src"

[services.postgres]
enabled = true
version = "16"
port = 5432

[runtime]
cap_add = ["NET_ADMIN"]
memory_limit = "4g"
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cc-container.toml");
        std::fs::write(&path, toml_str).unwrap();
        let config = load_project_config(&path).unwrap();
        assert_eq!(config.project.name, "full");
        assert_eq!(config.image.base, project::BaseOs::Debian);
        assert!(config.auth.claude.is_some());
        assert!(config.firewall.enabled);
    }

    // ───────────────────── load_effective_config ─────────────────────

    #[test]
    fn load_effective_config_not_found() {
        let path = std::path::Path::new("/nonexistent/cc-container.toml");
        let err = load_effective_config(path).unwrap_err();
        assert!(matches!(err, Error::ConfigNotFound(_)));
    }

    #[test]
    fn load_effective_config_loads_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cc-container.toml");
        std::fs::write(&path, MINIMAL_CONFIG).unwrap();
        // This will attempt to load user config from the real home directory.
        // Even if user config exists, the project config should be loaded correctly.
        let config = load_effective_config(&path).unwrap();
        assert_eq!(config.project.name, "test-project");
    }

    // ───────────────────── load_user_config ─────────────────────

    #[test]
    fn load_user_config_returns_option() {
        // We can't control the filesystem easily here, but we can at least
        // verify it doesn't panic and returns Ok.
        let result = load_user_config();
        assert!(result.is_ok());
    }

    // ───────────────────── Empty file ─────────────────────

    #[test]
    fn load_project_config_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"").unwrap();
        let err = load_project_config(&path).unwrap_err();
        assert!(matches!(err, Error::ConfigParse(_)));
    }
}
