use super::project::ProjectConfig;
use super::user::UserConfig;

/// Merge user defaults into a project config where project values are not set.
/// Project config takes priority over user config.
pub fn merge_configs(project: &mut ProjectConfig, user: &UserConfig) {
    let defaults = &user.defaults;

    if let Some(base) = defaults.base {
        // Only override if project is using the built-in default
        // (we can't distinguish "explicitly set" from "default" with serde,
        // so we only merge modules from user config)
        let _ = base; // reserved for future use
    }

    if let Some(ref shell) = defaults.shell {
        let _ = shell;
    }

    // Merge default modules: add user default modules that aren't in the project config
    for (name, value) in &defaults.modules {
        if !project.modules.contains_key(name) {
            project.modules.insert(name.clone(), value.clone());
        }
    }
}
