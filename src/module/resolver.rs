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
