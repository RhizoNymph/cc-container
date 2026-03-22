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
