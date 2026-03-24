use indexmap::IndexMap;
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use std::collections::HashMap;

use super::registry::ModuleRegistry;
use crate::error::{Error, Result};

/// Resolves module dependencies and produces a topologically sorted order.
pub struct ModuleResolver<'a> {
    registry: &'a ModuleRegistry,
}

impl<'a> ModuleResolver<'a> {
    pub fn new(registry: &'a ModuleRegistry) -> Self {
        Self { registry }
    }

    /// Given the set of enabled module names (from config), resolve dependencies
    /// and return the ordered list of module names for Dockerfile generation.
    pub fn resolve(&self, enabled: &IndexMap<String, toml::Value>) -> Result<Vec<String>> {
        let mut all_modules: IndexMap<String, ()> = IndexMap::new();

        // Collect all enabled modules and auto-add required dependencies
        let mut queue: Vec<String> = enabled.keys().cloned().collect();
        while let Some(name) = queue.pop() {
            if all_modules.contains_key(&name) {
                continue;
            }

            let entry = self
                .registry
                .get(&name)
                .ok_or_else(|| Error::ModuleNotFound(name.clone()))?;

            all_modules.insert(name.clone(), ());

            // Auto-add required modules
            for req in &entry.definition.module.dependencies.requires {
                if !all_modules.contains_key(req) {
                    if !self.registry.contains(req) {
                        return Err(Error::MissingDependency {
                            required: req.clone(),
                            requester: name.clone(),
                        });
                    }
                    queue.push(req.clone());
                }
            }
        }

        // Check for conflicts
        let module_names: Vec<&String> = all_modules.keys().collect();
        for name in &module_names {
            let entry = self.registry.get(name.as_str()).unwrap();
            for conflict in &entry.definition.module.dependencies.conflicts {
                if all_modules.contains_key(conflict) {
                    return Err(Error::ModuleConflict {
                        a: name.to_string(),
                        b: conflict.clone(),
                    });
                }
            }
        }

        // Build dependency graph for topological sort
        let mut graph = DiGraph::<&str, ()>::new();
        let mut node_map: HashMap<&str, petgraph::graph::NodeIndex> = HashMap::new();

        for name in all_modules.keys() {
            let idx = graph.add_node(name.as_str());
            node_map.insert(name.as_str(), idx);
        }

        for name in all_modules.keys() {
            let entry = self.registry.get(name.as_str()).unwrap();
            let to_idx = node_map[name.as_str()];

            for after in &entry.definition.module.dependencies.after {
                if let Some(&from_idx) = node_map.get(after.as_str()) {
                    // `after` means this module comes after `after_module`,
                    // so add edge from `after_module` -> this module
                    graph.add_edge(from_idx, to_idx, ());
                }
            }

            for req in &entry.definition.module.dependencies.requires {
                if let Some(&from_idx) = node_map.get(req.as_str()) {
                    graph.add_edge(from_idx, to_idx, ());
                }
            }
        }

        // Topological sort
        let sorted = toposort(&graph, None).map_err(|_| Error::CircularDependency)?;

        Ok(sorted
            .into_iter()
            .map(|idx| graph[idx].to_string())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::module::registry::ModuleRegistry;

    #[test]
    fn test_resolve_single_module_no_deps() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "ubuntu".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();
        assert_eq!(result, vec!["ubuntu"]);
    }

    #[test]
    fn test_resolve_empty_modules() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let enabled = IndexMap::new();
        let result = resolver.resolve(&enabled).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_auto_adds_required_dependencies() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        // claude-code requires node; only enable claude-code
        let mut enabled = IndexMap::new();
        enabled.insert(
            "claude-code".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();
        assert!(
            result.contains(&"node".to_string()),
            "node should be auto-added as a dependency of claude-code"
        );
        assert!(result.contains(&"claude-code".to_string()));

        // node must come before claude-code
        let node_pos = result.iter().position(|m| m == "node").unwrap();
        let claude_pos = result.iter().position(|m| m == "claude-code").unwrap();
        assert!(
            node_pos < claude_pos,
            "node (pos {}) must come before claude-code (pos {})",
            node_pos,
            claude_pos
        );
    }

    #[test]
    fn test_resolve_conflict_detection() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        // ubuntu and debian conflict with each other
        let mut enabled = IndexMap::new();
        enabled.insert(
            "ubuntu".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "debian".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ModuleConflict { a, b } => {
                // One of them should report the conflict
                let pair = format!("{}-{}", a, b);
                assert!(
                    pair.contains("ubuntu") && pair.contains("debian"),
                    "Conflict should be between ubuntu and debian, got: {} and {}",
                    a,
                    b
                );
            }
            other => panic!("Expected ModuleConflict, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_triple_base_conflict() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "ubuntu".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "alpine".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_unknown_module() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "nonexistent-module".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ModuleNotFound(name) => {
                assert_eq!(name, "nonexistent-module");
            }
            other => panic!("Expected ModuleNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_after_ordering() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        // node has `after = ["ubuntu", ...]` and user-setup has `after = ["ubuntu", ...]`
        let mut enabled = IndexMap::new();
        enabled.insert(
            "ubuntu".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "node".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "user-setup".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();

        let ubuntu_pos = result.iter().position(|m| m == "ubuntu").unwrap();
        let node_pos = result.iter().position(|m| m == "node").unwrap();
        let user_pos = result.iter().position(|m| m == "user-setup").unwrap();

        assert!(
            ubuntu_pos < node_pos,
            "ubuntu must come before node (after constraint)"
        );
        assert!(
            ubuntu_pos < user_pos,
            "ubuntu must come before user-setup (after constraint)"
        );
    }

    #[test]
    fn test_resolve_after_constraint_ignored_for_absent_modules() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        // node has `after = ["ubuntu", "debian", "alpine", "user-setup"]`
        // but if ubuntu/debian/alpine/user-setup aren't enabled, the constraint is skipped
        let mut enabled = IndexMap::new();
        enabled.insert(
            "node".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();
        assert_eq!(result, vec!["node"]);
    }

    #[test]
    fn test_resolve_complex_dependency_chain() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        // Build a realistic scenario: ubuntu + git + node + claude-code + user-setup
        let mut enabled = IndexMap::new();
        enabled.insert(
            "ubuntu".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "git".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "user-setup".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "claude-code".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();

        // node should be auto-added (required by claude-code)
        assert!(result.contains(&"node".to_string()));

        // Verify ordering constraints
        let pos = |name: &str| result.iter().position(|m| m == name).unwrap();

        assert!(pos("ubuntu") < pos("git"), "ubuntu before git");
        assert!(pos("ubuntu") < pos("node"), "ubuntu before node");
        assert!(pos("ubuntu") < pos("user-setup"), "ubuntu before user-setup");
        assert!(pos("ubuntu") < pos("claude-code"), "ubuntu before claude-code");
        assert!(pos("node") < pos("claude-code"), "node before claude-code");
        assert!(pos("git") < pos("claude-code"), "git before claude-code (after constraint)");
    }

    #[test]
    fn test_resolve_codex_auto_adds_node() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "codex-cli".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();
        assert!(result.contains(&"node".to_string()));

        let node_pos = result.iter().position(|m| m == "node").unwrap();
        let codex_pos = result.iter().position(|m| m == "codex-cli").unwrap();
        assert!(node_pos < codex_pos);
    }

    #[test]
    fn test_resolve_multiple_independent_modules() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "git".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "build-essential".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"git".to_string()));
        assert!(result.contains(&"build-essential".to_string()));
    }

    #[test]
    fn test_resolve_with_base_and_langs() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "debian".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "python".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "node".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();
        let pos = |name: &str| result.iter().position(|m| m == name).unwrap();

        assert!(pos("debian") < pos("python"), "debian before python");
        assert!(pos("debian") < pos("node"), "debian before node");
    }

    #[test]
    fn test_resolve_already_included_dependency_not_duplicated() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        // Explicitly include node AND claude-code (which requires node)
        let mut enabled = IndexMap::new();
        enabled.insert(
            "node".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "claude-code".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();

        // node should appear exactly once
        let node_count = result.iter().filter(|m| m.as_str() == "node").count();
        assert_eq!(node_count, 1, "node should appear exactly once");
    }

    #[test]
    fn test_resolve_preserves_all_enabled_modules() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "ubuntu".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "git".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "python".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "user-setup".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "firewall".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled).unwrap();

        for name in enabled.keys() {
            assert!(
                result.contains(name),
                "Enabled module '{}' should be in result",
                name
            );
        }
    }

    #[test]
    fn test_resolve_missing_dependency_in_registry() {
        let registry = ModuleRegistry::new();
        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "totally-fake-module".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        match resolver.resolve(&enabled) {
            Err(Error::ModuleNotFound(name)) => {
                assert_eq!(name, "totally-fake-module");
            }
            other => panic!(
                "Expected ModuleNotFound error, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_resolve_user_module_with_missing_required_dep() {
        // Use load_user_modules to create a module that requires a nonexistent module
        let dir = tempfile::tempdir().unwrap();

        let toml_content = r#"
[module]
name = "needs-phantom"
category = "tool"
description = "Requires a phantom module"

[module.dependencies]
requires = ["phantom-module"]
"#;
        let template_content = "# placeholder\n";

        std::fs::write(dir.path().join("needs-phantom.toml"), toml_content).unwrap();
        std::fs::write(
            dir.path().join("needs-phantom.dockerfile.j2"),
            template_content,
        )
        .unwrap();

        let mut registry = ModuleRegistry::new();
        registry.load_user_modules(dir.path()).unwrap();

        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "needs-phantom".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        match resolver.resolve(&enabled) {
            Err(Error::MissingDependency {
                required,
                requester,
            }) => {
                assert_eq!(required, "phantom-module");
                assert_eq!(requester, "needs-phantom");
            }
            other => panic!(
                "Expected MissingDependency error, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_resolve_circular_dependency() {
        let dir = tempfile::tempdir().unwrap();

        // Module A requires B, module B requires A
        let toml_a = r#"
[module]
name = "circ-a"
category = "tool"
description = "Circular A"

[module.dependencies]
requires = ["circ-b"]
"#;
        let toml_b = r#"
[module]
name = "circ-b"
category = "tool"
description = "Circular B"

[module.dependencies]
requires = ["circ-a"]
"#;
        let template = "# placeholder\n";

        std::fs::write(dir.path().join("circ-a.toml"), toml_a).unwrap();
        std::fs::write(dir.path().join("circ-a.dockerfile.j2"), template).unwrap();
        std::fs::write(dir.path().join("circ-b.toml"), toml_b).unwrap();
        std::fs::write(dir.path().join("circ-b.dockerfile.j2"), template).unwrap();

        let mut registry = ModuleRegistry::new();
        registry.load_user_modules(dir.path()).unwrap();

        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "circ-a".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::CircularDependency => {} // expected
            other => panic!("Expected CircularDependency, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_user_module_conflict_with_builtin() {
        let dir = tempfile::tempdir().unwrap();

        let toml_content = r#"
[module]
name = "anti-git"
category = "tool"
description = "Conflicts with git"

[module.dependencies]
conflicts = ["git"]
"#;
        let template = "# placeholder\n";

        std::fs::write(dir.path().join("anti-git.toml"), toml_content).unwrap();
        std::fs::write(dir.path().join("anti-git.dockerfile.j2"), template).unwrap();

        let mut registry = ModuleRegistry::new();
        registry.load_user_modules(dir.path()).unwrap();

        let resolver = ModuleResolver::new(&registry);

        let mut enabled = IndexMap::new();
        enabled.insert(
            "git".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        enabled.insert(
            "anti-git".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = resolver.resolve(&enabled);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ModuleConflict { a, b } => {
                let pair = format!("{}-{}", a, b);
                assert!(
                    pair.contains("anti-git") && pair.contains("git"),
                    "Conflict should involve anti-git and git: got {} and {}",
                    a,
                    b
                );
            }
            other => panic!("Expected ModuleConflict, got: {:?}", other),
        }
    }
}
