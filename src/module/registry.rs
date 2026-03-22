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
    #[allow(dead_code)]
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
