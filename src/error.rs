use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("config file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("invalid config: {0}")]
    ConfigInvalid(String),

    #[error("failed to parse config: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("failed to serialize YAML: {0}")]
    YamlSerialize(#[from] serde_yaml::Error),

    #[error("failed to serialize JSON: {0}")]
    JsonSerialize(#[from] serde_json::Error),

    #[error("module not found: {0}")]
    ModuleNotFound(String),

    #[error("module conflict: {a} conflicts with {b}")]
    ModuleConflict { a: String, b: String },

    #[error("missing required module: {required} (needed by {requester})")]
    MissingDependency { required: String, requester: String },

    #[error("circular dependency detected in modules")]
    CircularDependency,

    #[error("template rendering error: {0}")]
    TemplateRender(String),

    #[error("service template not found: {0}")]
    ServiceNotFound(String),

    #[error("port conflict: port {port} used by both {a} and {b}")]
    PortConflict { port: u16, a: String, b: String },

    #[error("unsupported auth method '{method}' for {agent}")]
    UnsupportedAuthMethod { agent: String, method: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
