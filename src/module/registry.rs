use indexmap::IndexMap;
use std::path::Path;

use super::builtin;
use super::definition::ModuleDefinition;
use crate::error::{Error, Result};

/// An entry in the module registry.
pub struct ModuleEntry {
    pub definition: ModuleDefinition,
    pub template: String,
}

/// Registry of all available modules (built-in + user-defined).
pub struct ModuleRegistry {
    modules: IndexMap<String, ModuleEntry>,
}

impl ModuleRegistry {
    /// Create a new registry with all built-in modules loaded.
    pub fn new() -> Self {
        let mut modules = IndexMap::new();

        for builtin_mod in builtin::load_all() {
            let name = builtin_mod.definition.module.name.clone();
            modules.insert(
                name,
                ModuleEntry {
                    definition: builtin_mod.definition,
                    template: builtin_mod.template.to_string(),
                },
            );
        }

        Self { modules }
    }

    /// Load user-defined modules from a directory, adding to/overriding the registry.
    pub fn load_user_modules(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in walkdir::WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                let toml_content = std::fs::read_to_string(path)?;
                let definition: ModuleDefinition = toml::from_str(&toml_content)?;
                let name = definition.module.name.clone();

                // Look for the corresponding .dockerfile.j2 template
                let template_path = path.with_extension("dockerfile.j2");
                if !template_path.exists() {
                    return Err(Error::Other(format!(
                        "module '{}' missing template file: {}",
                        name,
                        template_path.display()
                    )));
                }
                let template = std::fs::read_to_string(&template_path)?;

                self.modules.insert(name, ModuleEntry {
                    definition,
                    template,
                });
            }
        }

        Ok(())
    }

    /// Get a module by name.
    pub fn get(&self, name: &str) -> Option<&ModuleEntry> {
        self.modules.get(name)
    }

    /// List all modules.
    pub fn all(&self) -> &IndexMap<String, ModuleEntry> {
        &self.modules
    }

    /// Check if a module exists.
    pub fn contains(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::definition::ModuleCategory;

    #[test]
    fn test_registry_new_loads_builtins() {
        let registry = ModuleRegistry::new();
        assert!(
            !registry.all().is_empty(),
            "Registry should have built-in modules"
        );
    }

    #[test]
    fn test_registry_contains_all_expected_builtins() {
        let registry = ModuleRegistry::new();

        let expected = vec![
            // Base
            "ubuntu", "debian", "alpine",
            // Lang
            "node", "python", "rust", "go", "java", "ruby", "dotnet", "zig", "cpp",
            // Tool
            "git", "build-essential", "docker-cli",
            // Agent
            "claude-code", "codex-cli",
            // Security
            "user-setup", "firewall",
        ];

        for name in &expected {
            assert!(
                registry.contains(name),
                "Registry missing expected built-in module: {}",
                name
            );
        }

        // Verify the total count matches
        assert_eq!(registry.all().len(), expected.len());
    }

    #[test]
    fn test_registry_get_returns_correct_module() {
        let registry = ModuleRegistry::new();

        let entry = registry.get("ubuntu").expect("ubuntu module should exist");
        assert_eq!(entry.definition.module.name, "ubuntu");
        assert_eq!(entry.definition.module.category, ModuleCategory::Base);
        assert!(!entry.template.is_empty());
    }

    #[test]
    fn test_registry_get_nonexistent_returns_none() {
        let registry = ModuleRegistry::new();
        assert!(registry.get("nonexistent-module").is_none());
    }

    #[test]
    fn test_registry_contains_true_for_existing() {
        let registry = ModuleRegistry::new();
        assert!(registry.contains("node"));
        assert!(registry.contains("claude-code"));
    }

    #[test]
    fn test_registry_contains_false_for_nonexistent() {
        let registry = ModuleRegistry::new();
        assert!(!registry.contains("foobar"));
        assert!(!registry.contains(""));
    }

    #[test]
    fn test_registry_modules_have_correct_categories() {
        let registry = ModuleRegistry::new();

        let base_modules = ["ubuntu", "debian", "alpine"];
        for name in &base_modules {
            let entry = registry.get(name).unwrap();
            assert_eq!(
                entry.definition.module.category,
                ModuleCategory::Base,
                "{} should be base category",
                name
            );
        }

        let lang_modules = ["node", "python", "rust", "go", "java", "ruby", "dotnet", "zig", "cpp"];
        for name in &lang_modules {
            let entry = registry.get(name).unwrap();
            assert_eq!(
                entry.definition.module.category,
                ModuleCategory::Lang,
                "{} should be lang category",
                name
            );
        }

        let tool_modules = ["git", "build-essential", "docker-cli"];
        for name in &tool_modules {
            let entry = registry.get(name).unwrap();
            assert_eq!(
                entry.definition.module.category,
                ModuleCategory::Tool,
                "{} should be tool category",
                name
            );
        }

        let agent_modules = ["claude-code", "codex-cli"];
        for name in &agent_modules {
            let entry = registry.get(name).unwrap();
            assert_eq!(
                entry.definition.module.category,
                ModuleCategory::Agent,
                "{} should be agent category",
                name
            );
        }

        let security_modules = ["user-setup", "firewall"];
        for name in &security_modules {
            let entry = registry.get(name).unwrap();
            assert_eq!(
                entry.definition.module.category,
                ModuleCategory::Security,
                "{} should be security category",
                name
            );
        }
    }

    #[test]
    fn test_registry_modules_have_templates() {
        let registry = ModuleRegistry::new();
        for (name, entry) in registry.all() {
            assert!(
                !entry.template.is_empty(),
                "Module {} should have a non-empty template",
                name
            );
        }
    }

    #[test]
    fn test_registry_module_names_match_keys() {
        let registry = ModuleRegistry::new();
        for (key, entry) in registry.all() {
            assert_eq!(
                key, &entry.definition.module.name,
                "Registry key '{}' should match module name '{}'",
                key, entry.definition.module.name
            );
        }
    }

    #[test]
    fn test_registry_base_modules_conflict_with_each_other() {
        let registry = ModuleRegistry::new();

        let ubuntu = registry.get("ubuntu").unwrap();
        assert!(ubuntu.definition.module.dependencies.conflicts.contains(&"debian".to_string()));
        assert!(ubuntu.definition.module.dependencies.conflicts.contains(&"alpine".to_string()));

        let debian = registry.get("debian").unwrap();
        assert!(debian.definition.module.dependencies.conflicts.contains(&"ubuntu".to_string()));
        assert!(debian.definition.module.dependencies.conflicts.contains(&"alpine".to_string()));

        let alpine = registry.get("alpine").unwrap();
        assert!(alpine.definition.module.dependencies.conflicts.contains(&"ubuntu".to_string()));
        assert!(alpine.definition.module.dependencies.conflicts.contains(&"debian".to_string()));
    }

    #[test]
    fn test_registry_agent_modules_require_node() {
        let registry = ModuleRegistry::new();

        let claude = registry.get("claude-code").unwrap();
        assert!(
            claude.definition.module.dependencies.requires.contains(&"node".to_string()),
            "claude-code should require node"
        );

        let codex = registry.get("codex-cli").unwrap();
        assert!(
            codex.definition.module.dependencies.requires.contains(&"node".to_string()),
            "codex-cli should require node"
        );
    }

    #[test]
    fn test_load_user_modules_nonexistent_dir_is_ok() {
        let mut registry = ModuleRegistry::new();
        let result = registry.load_user_modules(Path::new("/nonexistent/path"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_user_modules_from_temp_dir() {
        let dir = tempfile::tempdir().unwrap();

        let toml_content = r#"
[module]
name = "custom-mod"
category = "tool"
description = "A custom user module"
"#;
        let template_content = "# Custom module\nRUN echo hello\n";

        std::fs::write(dir.path().join("custom-mod.toml"), toml_content).unwrap();
        std::fs::write(
            dir.path().join("custom-mod.dockerfile.j2"),
            template_content,
        )
        .unwrap();

        let mut registry = ModuleRegistry::new();
        let builtin_count = registry.all().len();

        registry.load_user_modules(dir.path()).unwrap();

        assert_eq!(registry.all().len(), builtin_count + 1);
        assert!(registry.contains("custom-mod"));

        let entry = registry.get("custom-mod").unwrap();
        assert_eq!(entry.definition.module.name, "custom-mod");
        assert_eq!(entry.template, template_content);
    }

    #[test]
    fn test_load_user_modules_missing_template_errors() {
        let dir = tempfile::tempdir().unwrap();

        let toml_content = r#"
[module]
name = "no-template"
category = "tool"
description = "Missing template"
"#;
        std::fs::write(dir.path().join("no-template.toml"), toml_content).unwrap();
        // Deliberately do NOT create the .dockerfile.j2 file

        let mut registry = ModuleRegistry::new();
        let result = registry.load_user_modules(dir.path());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("missing template file"),
            "Error should mention missing template: {}",
            err_msg
        );
    }

    #[test]
    fn test_load_user_modules_overrides_builtin() {
        let dir = tempfile::tempdir().unwrap();

        // Override the "git" module with a custom one
        let toml_content = r#"
[module]
name = "git"
category = "tool"
description = "Custom git module"
"#;
        let template_content = "# My custom git\nRUN apt-get install git-custom\n";

        std::fs::write(dir.path().join("git.toml"), toml_content).unwrap();
        std::fs::write(dir.path().join("git.dockerfile.j2"), template_content).unwrap();

        let mut registry = ModuleRegistry::new();
        registry.load_user_modules(dir.path()).unwrap();

        let entry = registry.get("git").unwrap();
        assert_eq!(entry.definition.module.description, "Custom git module");
        assert_eq!(entry.template, template_content);
    }
}
