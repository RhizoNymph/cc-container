use super::project::ProjectConfig;
use super::user::UserConfig;

/// Modules that are auto-managed by the renderer from top-level config fields.
/// User defaults for these would create broken output (e.g. a firewall COPY
/// without the corresponding init-firewall.sh file).
const AUTO_MANAGED_MODULES: &[&str] = &[
    "ubuntu", "debian", "alpine",
    "claude-code", "codex-cli",
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
    let base_inherited = if let Some(base) = defaults.base {
        if !image_field_present(raw_image, "base") {
            project.image.base = base;
            true
        } else {
            false
        }
    } else {
        false
    };

    if let Some(shell) = defaults.shell {
        if !image_field_present(raw_image, "shell") {
            project.image.shell = shell;
        }
    }

    if let Some(ref platform) = defaults.platform {
        if !image_field_present(raw_image, "platform") {
            project.image.platform = platform.clone();
        }
    }

    // Only inherit base_version when the project didn't set it AND the base
    // was also inherited.  base_version is only valid relative to the selected
    // base OS, so copying a Debian version into an Ubuntu project is wrong.
    if let Some(ref base_version) = defaults.base_version {
        if !image_field_present(raw_image, "base_version") && base_inherited {
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
}
