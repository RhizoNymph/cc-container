use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A module definition loaded from a .toml metadata file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDefinition {
    pub module: ModuleMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMeta {
    pub name: String,
    pub category: ModuleCategory,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub parameters: IndexMap<String, ParameterDef>,
    #[serde(default)]
    pub dependencies: ModuleDependencies,
    #[serde(default)]
    pub metadata: ModuleMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleCategory {
    Base,
    Lang,
    Tool,
    Agent,
    Security,
    Custom,
}

impl std::fmt::Display for ModuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base => write!(f, "base"),
            Self::Lang => write!(f, "lang"),
            Self::Tool => write!(f, "tool"),
            Self::Agent => write!(f, "agent"),
            Self::Security => write!(f, "security"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    #[serde(rename = "type")]
    pub param_type: ParamType,
    #[serde(default)]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub allowed_values: Option<Vec<toml::Value>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Bool,
    Int,
    List,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleDependencies {
    /// Modules that must be present (auto-added if missing).
    #[serde(default)]
    pub requires: Vec<String>,
    /// Modules that cannot coexist with this one.
    #[serde(default)]
    pub conflicts: Vec<String>,
    /// Modules that must appear before this one in the Dockerfile.
    #[serde(default)]
    pub after: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleMetadata {
    #[serde(default)]
    pub env_vars: Vec<String>,
    #[serde(default)]
    pub exposed_ports: Vec<u16>,
    #[serde(default)]
    pub volumes: Vec<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_module_definition_deserialization() {
        let toml_str = r#"
[module]
name = "test-mod"
category = "lang"
description = "A test module"
version = "2.0.0"

[module.parameters]
version = { type = "string", default = "1.0", description = "Version string" }
enable_feature = { type = "bool", default = true, description = "Toggle feature" }
count = { type = "int", default = 3, description = "A count param" }

[module.dependencies]
requires = ["node"]
conflicts = ["python"]
after = ["ubuntu", "user-setup"]

[module.metadata]
env_vars = ["MY_VAR"]
exposed_ports = [8080, 3000]
volumes = ["/data"]
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(def.module.name, "test-mod");
        assert_eq!(def.module.category, ModuleCategory::Lang);
        assert_eq!(def.module.description, "A test module");
        assert_eq!(def.module.version, "2.0.0");

        // Parameters
        assert_eq!(def.module.parameters.len(), 3);

        let version_param = &def.module.parameters["version"];
        assert_eq!(version_param.param_type, ParamType::String);
        assert_eq!(
            version_param.default,
            Some(toml::Value::String("1.0".to_string()))
        );
        assert_eq!(version_param.description, "Version string");

        let bool_param = &def.module.parameters["enable_feature"];
        assert_eq!(bool_param.param_type, ParamType::Bool);
        assert_eq!(bool_param.default, Some(toml::Value::Boolean(true)));

        let int_param = &def.module.parameters["count"];
        assert_eq!(int_param.param_type, ParamType::Int);
        assert_eq!(int_param.default, Some(toml::Value::Integer(3)));

        // Dependencies
        assert_eq!(def.module.dependencies.requires, vec!["node"]);
        assert_eq!(def.module.dependencies.conflicts, vec!["python"]);
        assert_eq!(def.module.dependencies.after, vec!["ubuntu", "user-setup"]);

        // Metadata
        assert_eq!(def.module.metadata.env_vars, vec!["MY_VAR"]);
        assert_eq!(def.module.metadata.exposed_ports, vec![8080, 3000]);
        assert_eq!(def.module.metadata.volumes, vec!["/data"]);
    }

    #[test]
    fn test_minimal_module_definition_uses_defaults() {
        let toml_str = r#"
[module]
name = "minimal"
category = "tool"
description = "Minimal module"
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(def.module.name, "minimal");
        assert_eq!(def.module.category, ModuleCategory::Tool);
        assert_eq!(def.module.version, "1.0.0"); // default_version()
        assert!(def.module.parameters.is_empty());
        assert!(def.module.dependencies.requires.is_empty());
        assert!(def.module.dependencies.conflicts.is_empty());
        assert!(def.module.dependencies.after.is_empty());
        assert!(def.module.metadata.env_vars.is_empty());
        assert!(def.module.metadata.exposed_ports.is_empty());
        assert!(def.module.metadata.volumes.is_empty());
    }

    #[test]
    fn test_all_categories_deserialize() {
        for (cat_str, expected) in [
            ("base", ModuleCategory::Base),
            ("lang", ModuleCategory::Lang),
            ("tool", ModuleCategory::Tool),
            ("agent", ModuleCategory::Agent),
            ("security", ModuleCategory::Security),
            ("custom", ModuleCategory::Custom),
        ] {
            let toml_str = format!(
                r#"
[module]
name = "cat-test"
category = "{}"
description = "test"
"#,
                cat_str
            );
            let def: ModuleDefinition = toml::from_str(&toml_str).unwrap();
            assert_eq!(def.module.category, expected, "Failed for category: {}", cat_str);
        }
    }

    #[test]
    fn test_category_display() {
        assert_eq!(ModuleCategory::Base.to_string(), "base");
        assert_eq!(ModuleCategory::Lang.to_string(), "lang");
        assert_eq!(ModuleCategory::Tool.to_string(), "tool");
        assert_eq!(ModuleCategory::Agent.to_string(), "agent");
        assert_eq!(ModuleCategory::Security.to_string(), "security");
        assert_eq!(ModuleCategory::Custom.to_string(), "custom");
    }

    #[test]
    fn test_invalid_category_fails() {
        let toml_str = r#"
[module]
name = "bad"
category = "nonexistent"
description = "test"
"#;
        let result: std::result::Result<ModuleDefinition, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields_fails() {
        // Missing name
        let toml_str = r#"
[module]
category = "tool"
description = "test"
"#;
        assert!(toml::from_str::<ModuleDefinition>(toml_str).is_err());

        // Missing category
        let toml_str = r#"
[module]
name = "test"
description = "test"
"#;
        assert!(toml::from_str::<ModuleDefinition>(toml_str).is_err());

        // Missing description
        let toml_str = r#"
[module]
name = "test"
category = "tool"
"#;
        assert!(toml::from_str::<ModuleDefinition>(toml_str).is_err());
    }

    #[test]
    fn test_parameter_without_default() {
        let toml_str = r#"
[module]
name = "no-defaults"
category = "tool"
description = "test"

[module.parameters]
name = { type = "string", description = "A required param" }
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        let param = &def.module.parameters["name"];
        assert_eq!(param.param_type, ParamType::String);
        assert!(param.default.is_none());
        assert_eq!(param.description, "A required param");
    }

    #[test]
    fn test_parameter_with_allowed_values() {
        let toml_str = r#"
[module]
name = "allowed-vals"
category = "lang"
description = "test"

[module.parameters]
version = { type = "string", default = "22", description = "Version", allowed_values = ["18", "20", "22"] }
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        let param = &def.module.parameters["version"];
        let allowed = param.allowed_values.as_ref().unwrap();
        assert_eq!(allowed.len(), 3);
        assert_eq!(allowed[0], toml::Value::String("18".to_string()));
        assert_eq!(allowed[1], toml::Value::String("20".to_string()));
        assert_eq!(allowed[2], toml::Value::String("22".to_string()));
    }

    #[test]
    fn test_all_param_types() {
        for (type_str, expected) in [
            ("string", ParamType::String),
            ("bool", ParamType::Bool),
            ("int", ParamType::Int),
            ("list", ParamType::List),
        ] {
            let toml_str = format!(
                r#"
[module]
name = "type-test"
category = "tool"
description = "test"

[module.parameters]
p = {{ type = "{}", description = "test" }}
"#,
                type_str
            );
            let def: ModuleDefinition = toml::from_str(&toml_str).unwrap();
            assert_eq!(
                def.module.parameters["p"].param_type, expected,
                "Failed for type: {}",
                type_str
            );
        }
    }

    #[test]
    fn test_parameter_insertion_order_preserved() {
        let toml_str = r#"
[module]
name = "ordered"
category = "tool"
description = "test"

[module.parameters]
alpha = { type = "string", description = "first" }
beta = { type = "string", description = "second" }
gamma = { type = "string", description = "third" }
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        let keys: Vec<&String> = def.module.parameters.keys().collect();
        assert_eq!(keys, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_dependencies_default_to_empty() {
        let toml_str = r#"
[module]
name = "no-deps"
category = "tool"
description = "test"
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        assert!(def.module.dependencies.requires.is_empty());
        assert!(def.module.dependencies.conflicts.is_empty());
        assert!(def.module.dependencies.after.is_empty());
    }

    #[test]
    fn test_metadata_defaults_to_empty() {
        let toml_str = r#"
[module]
name = "no-meta"
category = "tool"
description = "test"
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        assert!(def.module.metadata.env_vars.is_empty());
        assert!(def.module.metadata.exposed_ports.is_empty());
        assert!(def.module.metadata.volumes.is_empty());
    }

    #[test]
    fn test_multiple_dependencies() {
        let toml_str = r#"
[module]
name = "multi-deps"
category = "agent"
description = "test"

[module.dependencies]
requires = ["node", "git"]
conflicts = ["python", "ruby"]
after = ["ubuntu", "debian", "alpine", "user-setup"]
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(def.module.dependencies.requires, vec!["node", "git"]);
        assert_eq!(def.module.dependencies.conflicts, vec!["python", "ruby"]);
        assert_eq!(
            def.module.dependencies.after,
            vec!["ubuntu", "debian", "alpine", "user-setup"]
        );
    }

    #[test]
    fn test_real_ubuntu_module_format() {
        // Matches the actual ubuntu.toml structure
        let toml_str = r#"
[module]
name = "ubuntu"
category = "base"
description = "Ubuntu base image"
version = "1.0.0"

[module.parameters]
version = { type = "string", default = "24.04", description = "Ubuntu version" }

[module.dependencies]
requires = []
conflicts = ["debian", "alpine"]
after = []
"#;
        let def: ModuleDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(def.module.name, "ubuntu");
        assert_eq!(def.module.category, ModuleCategory::Base);
        assert_eq!(def.module.dependencies.conflicts, vec!["debian", "alpine"]);
        assert_eq!(
            def.module.parameters["version"].default,
            Some(toml::Value::String("24.04".to_string()))
        );
    }
}
