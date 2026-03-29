use super::project::{ProjectConfig, ShellType};
use super::user::UserConfig;

/// Modules that are auto-managed by the renderer from top-level config fields.
/// User defaults for these would create broken output (e.g. a firewall COPY
/// without the corresponding init-firewall.sh file).
const AUTO_MANAGED_MODULES: &[&str] = &[
    "ubuntu",
    "debian",
    "alpine",
    "claude-code",
    "codex-cli",
    "firewall",
];

/// Returns true if `key` was explicitly written in the `[image]` table of the
/// project TOML.  Returns false when the table or key is absent (i.e. serde
/// filled it with a default).
fn image_field_present(raw_image: Option<&toml::Value>, key: &str) -> bool {
    raw_image
        .and_then(|v| v.as_table())
        .is_some_and(|t| t.contains_key(key))
}

/// Merge user defaults into a project config where project values are not set.
/// Project config takes priority over user config.
///
/// `raw_image` is the raw `[image]` TOML value from the project file (if any).
/// It is used to tell whether a field was explicitly written or filled by serde
/// defaults — equality-based checks cannot distinguish those two cases.
pub fn merge_configs(
    project: &mut ProjectConfig,
    user: &UserConfig,
    raw_image: Option<&toml::Value>,
) {
    let defaults = &user.defaults;

    // Apply top-level image defaults only when the project file did not
    // contain the key at all.  This avoids overwriting a project that
    // explicitly wrote `base = "ubuntu"` when the user default is "alpine".
    if let Some(base) = defaults.base
        && !image_field_present(raw_image, "base")
    {
        project.image.base = base;
    }

    if let Some(shell) = defaults.shell
        && !image_field_present(raw_image, "shell")
    {
        project.image.shell = shell;
    }

    if let Some(ref platform) = defaults.platform
        && !image_field_present(raw_image, "platform")
    {
        project.image.platform = platform.clone();
    }

    // Inherit base_version only when:
    //  - the project didn't set base_version, AND
    //  - neither side introduced an OS that could conflict with the version.
    // A version string is OS-specific (e.g. "22.04" is Ubuntu, "bookworm" is
    // Debian), so it's only safe to inherit when the effective base was chosen
    // by the same side that chose the version — i.e. both came from user
    // defaults.  If the project explicitly set base, or the user default's
    // base doesn't match, the version may be wrong for the effective OS.
    if let Some(ref base_version) = defaults.base_version {
        let project_set_base = image_field_present(raw_image, "base");
        let default_base_mismatch = defaults.base.is_some_and(|b| b != project.image.base);
        if !image_field_present(raw_image, "base_version")
            && !project_set_base
            && !default_base_mismatch
        {
            project.image.base_version = Some(base_version.clone());
        }
    }

    // Merge default modules into project config.
    for (name, value) in &defaults.modules {
        // Skip modules that are auto-managed by the renderer — injecting them
        // via user defaults would produce broken Dockerfiles.
        if AUTO_MANAGED_MODULES.contains(&name.as_str()) {
            continue;
        }

        if let Some(existing) = project.modules.get(name) {
            // Module already enabled in project: merge user default params
            // as fallbacks (project params take priority).
            if let (toml::Value::Table(user_table), toml::Value::Table(existing_table)) =
                (value, existing)
            {
                let mut merged = user_table.clone();
                // Project values overwrite user defaults
                for (k, v) in existing_table {
                    merged.insert(k.clone(), v.clone());
                }
                project
                    .modules
                    .insert(name.clone(), toml::Value::Table(merged));
            }
            // If either side is not a table, project value wins unchanged.
        } else {
            // Module not in project: add it from user defaults.
            project.modules.insert(name.clone(), value.clone());
        }
    }

    // Post-merge consistency: if user-setup was merged from user defaults,
    // propagate its username/shell params back to the top-level image fields
    // so that the renderer's final `USER` instruction (which reads
    // config.image.user) stays in sync with the user created by the module.
    if let Some(user_setup) = project.modules.get("user-setup")
        && let Some(table) = user_setup.as_table()
    {
        if !image_field_present(raw_image, "user")
            && let Some(username) = table.get("username").and_then(|v| v.as_str())
        {
            project.image.user = username.to_string();
        }
        if !image_field_present(raw_image, "shell")
            && let Some(shell_str) = table.get("shell").and_then(|v| v.as_str())
        {
            let shell_name = shell_str.rsplit('/').next().unwrap_or(shell_str);
            match shell_name {
                "bash" => project.image.shell = ShellType::Bash,
                "zsh" => project.image.shell = ShellType::Zsh,
                "sh" => project.image.shell = ShellType::Sh,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::{BaseOs, ProjectConfig, ShellType};
    use crate::config::user::{UserConfig, UserDefaults};
    use indexmap::IndexMap;

    /// Helper: parse a minimal project config from TOML, returning both
    /// the deserialized struct and the raw TOML value for the [image] table.
    fn parse_project(toml_str: &str) -> (ProjectConfig, Option<toml::Value>) {
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let raw: toml::Value = toml::from_str(toml_str).unwrap();
        let raw_image = raw.get("image").cloned();
        (config, raw_image)
    }

    const MINIMAL: &str = r#"
[project]
name = "test"
[agent]
type = "claude"
"#;

    // ───────────────────── Base OS merging ─────────────────────

    #[test]
    fn merge_base_os_from_user_when_project_omits() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let user = UserConfig {
            defaults: UserDefaults {
                base: Some(BaseOs::Alpine),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.base, BaseOs::Alpine);
    }

    #[test]
    fn project_base_os_takes_precedence() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[image]
base = "debian"
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let user = UserConfig {
            defaults: UserDefaults {
                base: Some(BaseOs::Alpine),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.base, BaseOs::Debian);
    }

    // ───────────────────── Shell merging ─────────────────────

    #[test]
    fn merge_shell_from_user_when_project_omits() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let user = UserConfig {
            defaults: UserDefaults {
                shell: Some(ShellType::Zsh),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.shell, ShellType::Zsh);
    }

    #[test]
    fn project_shell_takes_precedence() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[image]
shell = "sh"
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let user = UserConfig {
            defaults: UserDefaults {
                shell: Some(ShellType::Zsh),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.shell, ShellType::Sh);
    }

    // ───────────────────── Platform merging ─────────────────────

    #[test]
    fn merge_platform_from_user_when_project_omits() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let user = UserConfig {
            defaults: UserDefaults {
                platform: Some("linux/arm64".to_string()),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.platform, "linux/arm64");
    }

    #[test]
    fn project_platform_takes_precedence() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[image]
platform = "linux/amd64"
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let user = UserConfig {
            defaults: UserDefaults {
                platform: Some("linux/arm64".to_string()),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.platform, "linux/amd64");
    }

    // ───────────────────── base_version merging ─────────────────────

    #[test]
    fn merge_base_version_when_both_from_user() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let user = UserConfig {
            defaults: UserDefaults {
                base: Some(BaseOs::Alpine),
                base_version: Some("3.21".to_string()),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.base, BaseOs::Alpine);
        assert_eq!(config.image.base_version.as_deref(), Some("3.21"));
    }

    #[test]
    fn no_merge_base_version_when_project_sets_base() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[image]
base = "debian"
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let user = UserConfig {
            defaults: UserDefaults {
                base_version: Some("3.21".to_string()),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert!(config.image.base_version.is_none());
    }

    #[test]
    fn no_merge_base_version_when_user_base_differs_from_project() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[image]
base = "ubuntu"
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let user = UserConfig {
            defaults: UserDefaults {
                base: Some(BaseOs::Alpine),
                base_version: Some("3.21".to_string()),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert!(config.image.base_version.is_none());
    }

    #[test]
    fn project_base_version_takes_precedence() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[image]
base_version = "22.04"
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let user = UserConfig {
            defaults: UserDefaults {
                base_version: Some("24.04".to_string()),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.base_version.as_deref(), Some("22.04"));
    }

    #[test]
    fn merge_base_version_when_user_base_none_and_project_omits_both() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let user = UserConfig {
            defaults: UserDefaults {
                base_version: Some("22.04".to_string()),
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.base_version.as_deref(), Some("22.04"));
    }

    // ───────────────────── Module merging ─────────────────────

    #[test]
    fn merge_new_module_from_user() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let mut modules = IndexMap::new();
        modules.insert(
            "nodejs".to_string(),
            toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert("version".to_string(), toml::Value::String("20".to_string()));
                t
            }),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert!(config.modules.contains_key("nodejs"));
        let node = config.modules["nodejs"].as_table().unwrap();
        assert_eq!(node["version"].as_str(), Some("20"));
    }

    #[test]
    fn project_module_params_take_precedence() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[modules]
nodejs = { version = "18" }
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let mut modules = IndexMap::new();
        modules.insert(
            "nodejs".to_string(),
            toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert("version".to_string(), toml::Value::String("20".to_string()));
                t.insert(
                    "global_packages".to_string(),
                    toml::Value::String("typescript".to_string()),
                );
                t
            }),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        let node = config.modules["nodejs"].as_table().unwrap();
        assert_eq!(node["version"].as_str(), Some("18"));
        assert_eq!(node["global_packages"].as_str(), Some("typescript"));
    }

    #[test]
    fn auto_managed_modules_not_merged() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let mut modules = IndexMap::new();
        for name in [
            "ubuntu",
            "debian",
            "alpine",
            "claude-code",
            "codex-cli",
            "firewall",
        ] {
            modules.insert(name.to_string(), toml::Value::Table(toml::map::Map::new()));
        }
        modules.insert(
            "nodejs".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert!(!config.modules.contains_key("ubuntu"));
        assert!(!config.modules.contains_key("debian"));
        assert!(!config.modules.contains_key("alpine"));
        assert!(!config.modules.contains_key("claude-code"));
        assert!(!config.modules.contains_key("codex-cli"));
        assert!(!config.modules.contains_key("firewall"));
        assert!(config.modules.contains_key("nodejs"));
    }

    // ───────────────────── user-setup propagation ─────────────────────

    #[test]
    fn user_setup_propagates_username() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let mut modules = IndexMap::new();
        modules.insert(
            "user-setup".to_string(),
            toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert(
                    "username".to_string(),
                    toml::Value::String("coder".to_string()),
                );
                t
            }),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.user, "coder");
    }

    #[test]
    fn user_setup_propagates_shell() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let mut modules = IndexMap::new();
        modules.insert(
            "user-setup".to_string(),
            toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert(
                    "shell".to_string(),
                    toml::Value::String("/bin/zsh".to_string()),
                );
                t
            }),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.shell, ShellType::Zsh);
    }

    #[test]
    fn user_setup_shell_with_full_path() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let mut modules = IndexMap::new();
        modules.insert(
            "user-setup".to_string(),
            toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert(
                    "shell".to_string(),
                    toml::Value::String("/usr/bin/zsh".to_string()),
                );
                t
            }),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.shell, ShellType::Zsh);
    }

    #[test]
    fn user_setup_does_not_override_explicit_image_user() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[image]
user = "admin"
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let mut modules = IndexMap::new();
        modules.insert(
            "user-setup".to_string(),
            toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert(
                    "username".to_string(),
                    toml::Value::String("coder".to_string()),
                );
                t
            }),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.user, "admin");
    }

    #[test]
    fn user_setup_does_not_override_explicit_image_shell() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[image]
shell = "sh"
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let mut modules = IndexMap::new();
        modules.insert(
            "user-setup".to_string(),
            toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert(
                    "shell".to_string(),
                    toml::Value::String("/bin/zsh".to_string()),
                );
                t
            }),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.shell, ShellType::Sh);
    }

    // ───────────────────── Empty user config ─────────────────────

    #[test]
    fn merge_with_empty_user_config_is_noop() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let original_base = config.image.base;
        let original_shell = config.image.shell;
        let original_platform = config.image.platform.clone();
        let user = UserConfig::default();
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.base, original_base);
        assert_eq!(config.image.shell, original_shell);
        assert_eq!(config.image.platform, original_platform);
        assert!(config.modules.is_empty());
    }

    // ───────────────────── Multiple fields at once ─────────────────────

    #[test]
    fn merge_multiple_fields_simultaneously() {
        let (mut config, raw_image) = parse_project(MINIMAL);
        let mut modules = IndexMap::new();
        modules.insert(
            "rust".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                base: Some(BaseOs::Debian),
                shell: Some(ShellType::Zsh),
                platform: Some("linux/arm64".to_string()),
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.image.base, BaseOs::Debian);
        assert_eq!(config.image.shell, ShellType::Zsh);
        assert_eq!(config.image.platform, "linux/arm64");
        assert!(config.modules.contains_key("rust"));
    }

    // ───────────────────── Non-table module values ─────────────────────

    #[test]
    fn merge_module_non_table_project_wins() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[modules]
nodejs = true
"#;
        let (mut config, raw_image) = parse_project(toml_str);
        let mut modules = IndexMap::new();
        modules.insert(
            "nodejs".to_string(),
            toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert("version".to_string(), toml::Value::String("20".to_string()));
                t
            }),
        );
        let user = UserConfig {
            defaults: UserDefaults {
                modules,
                ..Default::default()
            },
        };
        merge_configs(&mut config, &user, raw_image.as_ref());
        assert_eq!(config.modules["nodejs"], toml::Value::Boolean(true));
    }

    // ───────────────────── image_field_present helper ─────────────────────

    #[test]
    fn image_field_present_when_absent() {
        assert!(!image_field_present(None, "base"));
    }

    #[test]
    fn image_field_present_when_table_has_key() {
        let raw: toml::Value = toml::from_str(r#"base = "debian""#).unwrap();
        assert!(image_field_present(Some(&raw), "base"));
        assert!(!image_field_present(Some(&raw), "shell"));
    }

    #[test]
    fn image_field_present_with_non_table_value() {
        let raw = toml::Value::String("not a table".to_string());
        assert!(!image_field_present(Some(&raw), "base"));
    }
}
