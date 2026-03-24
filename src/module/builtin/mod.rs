use super::definition::ModuleDefinition;

/// A built-in module: its TOML definition + Dockerfile template.
pub struct BuiltinModule {
    pub definition: ModuleDefinition,
    pub template: &'static str,
}

macro_rules! builtin {
    ($toml:expr, $template:expr) => {{
        let def_str = include_str!($toml);
        let definition: ModuleDefinition =
            toml::from_str(def_str).expect(concat!("invalid built-in module TOML: ", $toml));
        BuiltinModule {
            definition,
            template: include_str!($template),
        }
    }};
}

/// Load all built-in modules.
pub fn load_all() -> Vec<BuiltinModule> {
    vec![
        // Base
        builtin!("base/ubuntu.toml", "base/ubuntu.dockerfile.j2"),
        builtin!("base/debian.toml", "base/debian.dockerfile.j2"),
        builtin!("base/alpine.toml", "base/alpine.dockerfile.j2"),
        // Lang
        builtin!("lang/node.toml", "lang/node.dockerfile.j2"),
        builtin!("lang/python.toml", "lang/python.dockerfile.j2"),
        builtin!("lang/rust.toml", "lang/rust.dockerfile.j2"),
        builtin!("lang/go.toml", "lang/go.dockerfile.j2"),
        builtin!("lang/java.toml", "lang/java.dockerfile.j2"),
        builtin!("lang/ruby.toml", "lang/ruby.dockerfile.j2"),
        builtin!("lang/dotnet.toml", "lang/dotnet.dockerfile.j2"),
        builtin!("lang/zig.toml", "lang/zig.dockerfile.j2"),
        builtin!("lang/cpp.toml", "lang/cpp.dockerfile.j2"),
        // Tool
        builtin!("tool/git.toml", "tool/git.dockerfile.j2"),
        builtin!(
            "tool/build_essential.toml",
            "tool/build_essential.dockerfile.j2"
        ),
        builtin!("tool/docker_cli.toml", "tool/docker_cli.dockerfile.j2"),
        // Agent
        builtin!("agent/claude_code.toml", "agent/claude_code.dockerfile.j2"),
        builtin!("agent/codex_cli.toml", "agent/codex_cli.dockerfile.j2"),
        // Security
        builtin!(
            "security/user_setup.toml",
            "security/user_setup.dockerfile.j2"
        ),
        builtin!(
            "security/firewall.toml",
            "security/firewall.dockerfile.j2"
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::definition::ModuleCategory;
    use std::collections::HashSet;

    #[test]
    fn test_load_all_succeeds() {
        // This verifies all built-in TOML files parse successfully
        let modules = load_all();
        assert!(!modules.is_empty());
    }

    #[test]
    fn test_load_all_expected_count() {
        let modules = load_all();
        // 3 base + 9 lang + 3 tool + 2 agent + 2 security = 19
        assert_eq!(modules.len(), 19);
    }

    #[test]
    fn test_load_all_unique_names() {
        let modules = load_all();
        let mut names: HashSet<String> = HashSet::new();
        for m in &modules {
            let name = m.definition.module.name.clone();
            assert!(
                names.insert(name.clone()),
                "Duplicate module name found: {}",
                name
            );
        }
    }

    #[test]
    fn test_load_all_every_module_has_template() {
        let modules = load_all();
        for m in &modules {
            assert!(
                !m.template.is_empty(),
                "Module '{}' has an empty template",
                m.definition.module.name
            );
        }
    }

    #[test]
    fn test_load_all_every_module_has_description() {
        let modules = load_all();
        for m in &modules {
            assert!(
                !m.definition.module.description.is_empty(),
                "Module '{}' has an empty description",
                m.definition.module.name
            );
        }
    }

    #[test]
    fn test_load_all_every_module_has_version() {
        let modules = load_all();
        for m in &modules {
            assert!(
                !m.definition.module.version.is_empty(),
                "Module '{}' has an empty version",
                m.definition.module.name
            );
        }
    }

    #[test]
    fn test_load_all_category_distribution() {
        let modules = load_all();

        let mut base_count = 0;
        let mut lang_count = 0;
        let mut tool_count = 0;
        let mut agent_count = 0;
        let mut security_count = 0;

        for m in &modules {
            match m.definition.module.category {
                ModuleCategory::Base => base_count += 1,
                ModuleCategory::Lang => lang_count += 1,
                ModuleCategory::Tool => tool_count += 1,
                ModuleCategory::Agent => agent_count += 1,
                ModuleCategory::Security => security_count += 1,
                ModuleCategory::Custom => {}
            }
        }

        assert_eq!(base_count, 3, "Expected 3 base modules");
        assert_eq!(lang_count, 9, "Expected 9 lang modules");
        assert_eq!(tool_count, 3, "Expected 3 tool modules");
        assert_eq!(agent_count, 2, "Expected 2 agent modules");
        assert_eq!(security_count, 2, "Expected 2 security modules");
    }

    #[test]
    fn test_load_all_base_modules_have_version_parameter() {
        let modules = load_all();
        for m in &modules {
            if m.definition.module.category == ModuleCategory::Base {
                assert!(
                    m.definition.module.parameters.contains_key("version"),
                    "Base module '{}' should have a 'version' parameter",
                    m.definition.module.name
                );
            }
        }
    }

    #[test]
    fn test_load_all_base_modules_conflict_with_each_other() {
        let modules = load_all();
        let base_names: Vec<String> = modules
            .iter()
            .filter(|m| m.definition.module.category == ModuleCategory::Base)
            .map(|m| m.definition.module.name.clone())
            .collect();

        for m in &modules {
            if m.definition.module.category == ModuleCategory::Base {
                for other in &base_names {
                    if other != &m.definition.module.name {
                        assert!(
                            m.definition
                                .module
                                .dependencies
                                .conflicts
                                .contains(other),
                            "Base module '{}' should conflict with '{}'",
                            m.definition.module.name,
                            other
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_load_all_templates_contain_dockerfile_instructions() {
        let modules = load_all();
        for m in &modules {
            let template = m.template;
            let name = &m.definition.module.name;

            // Base modules should have FROM
            if m.definition.module.category == ModuleCategory::Base {
                assert!(
                    template.contains("FROM"),
                    "Base module '{}' template should contain FROM instruction",
                    name
                );
            }

            // Non-base modules should have RUN or other instructions
            if m.definition.module.category != ModuleCategory::Base {
                assert!(
                    template.contains("RUN")
                        || template.contains("COPY")
                        || template.contains("ENV")
                        || template.contains("USER"),
                    "Module '{}' template should contain Dockerfile instructions",
                    name
                );
            }
        }
    }

    #[test]
    fn test_load_all_agent_modules_require_node() {
        let modules = load_all();
        for m in &modules {
            if m.definition.module.category == ModuleCategory::Agent {
                assert!(
                    m.definition
                        .module
                        .dependencies
                        .requires
                        .contains(&"node".to_string()),
                    "Agent module '{}' should require 'node'",
                    m.definition.module.name
                );
            }
        }
    }

    #[test]
    fn test_load_all_no_self_referencing_dependencies() {
        let modules = load_all();
        for m in &modules {
            let name = &m.definition.module.name;
            let deps = &m.definition.module.dependencies;

            assert!(
                !deps.requires.contains(name),
                "Module '{}' should not require itself",
                name
            );
            assert!(
                !deps.conflicts.contains(name),
                "Module '{}' should not conflict with itself",
                name
            );
            assert!(
                !deps.after.contains(name),
                "Module '{}' should not have 'after' on itself",
                name
            );
        }
    }
}
