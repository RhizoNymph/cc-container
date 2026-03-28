// Stub module: will be replaced by WS B (feat/helm-services branch).
// Provides the agent value builder for Helm chart generation.

use crate::config::project::{AgentType, ProjectConfig};
use crate::helm::types::*;
use indexmap::IndexMap;

/// Build agent values for the Helm chart.
///
/// Constructs `AgentValues` from the project config, agent type, and
/// infrastructure environment variables collected from enabled services.
pub fn build(
    config: &ProjectConfig,
    _agent_type: AgentType,
    infra_env: &IndexMap<String, String>,
) -> AgentValues {
    let image_repo = config
        .helm
        .image_repository
        .clone()
        .unwrap_or_else(|| config.project.name.clone());

    let image = ImageRef {
        registry: config.helm.image_registry.clone(),
        repository: image_repo,
        tag: config.helm.image_tag.clone(),
        pull_policy: "IfNotPresent".to_string(),
    };

    // Merge infrastructure env + user-defined env
    let mut env = infra_env.clone();
    for (key, val) in &config.environment.vars {
        env.insert(key.clone(), val.clone());
    }

    AgentValues {
        agent_type: config.agent.agent_type.to_string(),
        image,
        replicas: 1,
        resources: ResourceLimits {
            requests: ResourceSpec {
                cpu: config.runtime.cpu_limit.clone(),
                memory: config.runtime.memory_limit.clone(),
            },
            limits: ResourceSpec {
                cpu: config.runtime.cpu_limit.clone(),
                memory: config.runtime.memory_limit.clone(),
            },
        },
        env,
        env_from_secret: Vec::new(),
        volume_mounts: Vec::new(),
        workspace_pvc_size: config.helm.default_pvc_size.clone(),
        workspace_mount_path: config.workspace.mount_path.clone(),
        security_context: SecurityContext {
            run_as_non_root: true,
            capabilities_add: config.runtime.cap_add.clone(),
            capabilities_drop: config.runtime.cap_drop.clone(),
        },
    }
}
