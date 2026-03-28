// Stub module: will be replaced by WS B (feat/helm-services branch).
// Provides per-service value builders for Helm chart generation.

use crate::config::project::ServiceConfig;
use crate::error::Result;
use crate::helm::types::*;
use indexmap::IndexMap;

/// Build Helm `ServiceValues` and agent environment variables for a single
/// infrastructure service.
///
/// Returns a tuple of (service values, agent env vars to inject).
pub fn build_service(
    name: &str,
    config: &ServiceConfig,
) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    // Minimal stub implementation -- WS B will provide full per-service logic
    let svc = ServiceValues {
        enabled: config.enabled,
        image: ImageRef {
            registry: None,
            repository: name.to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "unknown".to_string(),
        stateful: true,
        ports: Vec::new(),
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: Vec::new(),
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec!["true".to_string()],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: ResourceLimits {
            requests: ResourceSpec {
                cpu: None,
                memory: None,
            },
            limits: ResourceSpec {
                cpu: None,
                memory: None,
            },
        },
    };

    let agent_env = IndexMap::new();
    Ok((svc, agent_env))
}
