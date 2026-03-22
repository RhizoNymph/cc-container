pub mod claude;
pub mod codex;

use indexmap::IndexMap;

/// Volume mount needed for auth.
#[derive(Debug, Clone)]
pub struct AuthVolume {
    pub source: String,
    pub target: String,
    pub read_only: bool,
}

/// Auth requirements for a compose service.
#[derive(Debug, Clone, Default)]
pub struct AuthRequirements {
    /// Environment variables to set (key -> value or ${REF}).
    pub env_vars: IndexMap<String, String>,
    /// Volume mounts needed for credential files.
    pub volumes: Vec<AuthVolume>,
    /// Lines to add to .env.example.
    pub env_example_lines: Vec<String>,
}
