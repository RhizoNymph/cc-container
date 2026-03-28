// Stub module: will be replaced by WS B (feat/helm-services branch).
// Provides secrets value builder for Helm chart generation.

use crate::config::project::ProjectConfig;
use crate::helm::types::SecretsValues;
use indexmap::IndexMap;

/// Build secrets values from config.
///
/// Maps auth config and service credentials to Helm `SecretsValues`
/// for Secret template generation.
pub fn build(_config: &ProjectConfig) -> SecretsValues {
    SecretsValues {
        auth_keys: Vec::new(),
        service_credentials: IndexMap::new(),
    }
}
