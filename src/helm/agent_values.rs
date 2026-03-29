use crate::auth;
use crate::config::project::{AgentType, ProjectConfig};
use crate::helm::types::{
    AgentValues, ImageRef, ResourceLimits, ResourceSpec, SecurityContext, VolumeDefinition,
    VolumeMount,
};
use indexmap::IndexMap;

/// Build the AgentValues for the Helm chart from project config.
///
/// Maps project configuration fields to the Helm chart's agent container spec:
/// image, resources, security context, environment variables, and volumes.
pub fn build(
    config: &ProjectConfig,
    agent_type: AgentType,
    infra_env: &IndexMap<String, String>,
) -> AgentValues {
    let container_user = &config.image.user;

    // Image reference
    let image = ImageRef {
        registry: config.helm.image_registry.clone(),
        repository: config
            .helm
            .image_repository
            .clone()
            .unwrap_or_else(|| config.project.name.clone()),
        tag: config.helm.image_tag.clone(),
        pull_policy: "IfNotPresent".to_string(),
    };

    // Resource limits from runtime config
    let resources = ResourceLimits {
        requests: ResourceSpec {
            cpu: None,
            memory: None,
        },
        limits: ResourceSpec {
            cpu: config.runtime.cpu_limit.clone(),
            memory: config.runtime.memory_limit.clone(),
        },
    };

    // Security context from runtime config
    let security_context = SecurityContext {
        run_as_non_root: true,
        capabilities_add: config.runtime.cap_add.clone(),
        capabilities_drop: config.runtime.cap_drop.clone(),
    };

    // Auth env vars that should come from the K8s Secret
    let auth_reqs = get_auth_requirements(config, agent_type, container_user);
    let env_from_secret: Vec<String> = auth_reqs.env_vars.keys().cloned().collect();

    // Merge infrastructure env vars + user-defined env vars
    let mut env: IndexMap<String, String> = IndexMap::new();
    for (key, val) in infra_env {
        env.insert(key.clone(), val.clone());
    }
    for (key, val) in &config.environment.vars {
        env.insert(key.clone(), val.clone());
    }

    // Workspace PVC
    let workspace_pvc_size = config
        .helm
        .default_pvc_size
        .clone();
    let workspace_mount_path = config.workspace.mount_path.clone();

    // Volume mounts and volume definitions built in parallel
    let mut volume_mounts: Vec<VolumeMount> = Vec::new();
    let mut volumes: Vec<VolumeDefinition> = Vec::new();

    // Named volumes -> PVC
    for (name, vol) in &config.volumes {
        volume_mounts.push(VolumeMount {
            name: name.clone(),
            mount_path: vol.target.clone(),
            read_only: false,
        });
        volumes.push(VolumeDefinition {
            name: name.clone(),
            pvc_claim_name: Some(format!("{}-{}", config.project.name, name)),
            secret_name: None,
            empty_dir: false,
        });
    }

    // Auth volumes -> Secret
    for (i, auth_vol) in auth_reqs.volumes.iter().enumerate() {
        let vol_name = format!("auth-{i}");
        volume_mounts.push(VolumeMount {
            name: vol_name.clone(),
            mount_path: auth_vol.target.clone(),
            read_only: auth_vol.read_only,
        });
        volumes.push(VolumeDefinition {
            name: vol_name,
            pvc_claim_name: None,
            secret_name: Some(format!("{}-secrets", config.project.name)),
            empty_dir: false,
        });
    }

    // Additional mounts -> emptyDir
    for (i, mount) in config.workspace.additional_mounts.iter().enumerate() {
        let vol_name = format!("mount-{i}");
        volume_mounts.push(VolumeMount {
            name: vol_name.clone(),
            mount_path: mount.target.clone(),
            read_only: mount.read_only,
        });
        volumes.push(VolumeDefinition {
            name: vol_name,
            pvc_claim_name: None,
            secret_name: None,
            empty_dir: true,
        });
    }

    AgentValues {
        agent_type: agent_type.to_string(),
        image,
        replicas: 1,
        resources,
        env,
        env_from_secret,
        volume_mounts,
        volumes,
        workspace_pvc_size,
        workspace_mount_path,
        security_context,
    }
}

fn get_auth_requirements(
    config: &ProjectConfig,
    agent_type: AgentType,
    container_user: &str,
) -> auth::AuthRequirements {
    let mut reqs = auth::AuthRequirements::default();

    match agent_type {
        AgentType::Claude => {
            if let Some(ref claude_auth) = config.auth.claude {
                reqs = auth::claude::requirements(claude_auth, container_user);
            }
        }
        AgentType::Codex => {
            if let Some(ref codex_auth) = config.auth.codex {
                reqs = auth::codex::requirements(codex_auth, container_user);
            }
        }
        AgentType::Both => {
            // For "both", the caller should build two separate AgentValues.
            // This arm is a no-op.
        }
    }

    reqs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::*;
    use indexmap::IndexMap;

    fn minimal_config() -> ProjectConfig {
        ProjectConfig {
            project: ProjectMeta {
                name: "test-project".to_string(),
                description: None,
            },
            agent: AgentConfig {
                agent_type: AgentType::Claude,
                claude_version: "latest".to_string(),
                codex_version: "latest".to_string(),
            },
            image: ImageConfig::default(),
            modules: IndexMap::new(),
            auth: AuthConfig::default(),
            firewall: FirewallConfig::default(),
            workspace: WorkspaceConfig::default(),
            volumes: IndexMap::new(),
            environment: EnvironmentConfig::default(),
            services: IndexMap::new(),
            mcp: IndexMap::new(),
            runtime: RuntimeConfig::default(),
            helm: HelmConfig::default(),
        }
    }

    // -- Basic construction --

    #[test]
    fn build_minimal_returns_valid_agent_values() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.agent_type, "claude");
        assert_eq!(av.replicas, 1);
        assert_eq!(av.workspace_mount_path, "/workspace");
        assert!(av.security_context.run_as_non_root);
    }

    #[test]
    fn build_uses_project_name_as_default_repository() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.image.repository, "test-project");
    }

    #[test]
    fn build_uses_helm_image_config() {
        let mut config = minimal_config();
        config.helm.image_registry = Some("ghcr.io/myorg".to_string());
        config.helm.image_repository = Some("my-agent".to_string());
        config.helm.image_tag = "v2.0".to_string();

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.image.registry, Some("ghcr.io/myorg".to_string()));
        assert_eq!(av.image.repository, "my-agent");
        assert_eq!(av.image.tag, "v2.0");
    }

    // -- Resource limits --

    #[test]
    fn build_maps_runtime_limits() {
        let mut config = minimal_config();
        config.runtime.cpu_limit = Some("2.0".to_string());
        config.runtime.memory_limit = Some("4Gi".to_string());

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.resources.limits.cpu, Some("2.0".to_string()));
        assert_eq!(av.resources.limits.memory, Some("4Gi".to_string()));
        assert!(av.resources.requests.cpu.is_none());
    }

    #[test]
    fn build_no_limits_produces_none() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert!(av.resources.limits.cpu.is_none());
        assert!(av.resources.limits.memory.is_none());
    }

    // -- Security context --

    #[test]
    fn build_maps_capabilities() {
        let mut config = minimal_config();
        config.runtime.cap_add = vec!["SYS_PTRACE".to_string()];
        config.runtime.cap_drop = vec!["NET_RAW".to_string()];

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.security_context.capabilities_add, vec!["SYS_PTRACE"]);
        assert_eq!(av.security_context.capabilities_drop, vec!["NET_RAW"]);
    }

    // -- Environment variables --

    #[test]
    fn build_merges_infra_env() {
        let config = minimal_config();
        let infra_env = IndexMap::from([
            ("DATABASE_URL".to_string(), "postgres://dev:pw@pg:5432/db".to_string()),
            ("REDIS_URL".to_string(), "redis://redis:6379".to_string()),
        ]);
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.env["DATABASE_URL"], "postgres://dev:pw@pg:5432/db");
        assert_eq!(av.env["REDIS_URL"], "redis://redis:6379");
    }

    #[test]
    fn build_merges_user_env() {
        let mut config = minimal_config();
        config.environment.vars.insert("MY_VAR".to_string(), "my_value".to_string());

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.env["MY_VAR"], "my_value");
    }

    #[test]
    fn build_user_env_overwrites_infra_env() {
        let mut config = minimal_config();
        config.environment.vars.insert("DATABASE_URL".to_string(), "override".to_string());

        let infra_env = IndexMap::from([
            ("DATABASE_URL".to_string(), "postgres://dev:pw@pg:5432/db".to_string()),
        ]);
        let av = build(&config, AgentType::Claude, &infra_env);

        // User env comes after infra, so it wins
        assert_eq!(av.env["DATABASE_URL"], "override");
    }

    // -- Auth env from secret --

    #[test]
    fn build_claude_api_key_env_from_secret() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::ApiKey,
        });

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert!(av.env_from_secret.contains(&"ANTHROPIC_API_KEY".to_string()));
    }

    #[test]
    fn build_codex_api_key_env_from_secret() {
        let mut config = minimal_config();
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::ApiKey,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Codex, &infra_env);

        assert!(av.env_from_secret.contains(&"OPENAI_API_KEY".to_string()));
    }

    #[test]
    fn build_claude_bedrock_env_from_secret() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Bedrock,
        });

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert!(av.env_from_secret.contains(&"AWS_ACCESS_KEY_ID".to_string()));
        assert!(av.env_from_secret.contains(&"AWS_SECRET_ACCESS_KEY".to_string()));
        assert!(av.env_from_secret.contains(&"AWS_REGION".to_string()));
    }

    #[test]
    fn build_no_auth_empty_env_from_secret() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert!(av.env_from_secret.is_empty());
    }

    // -- Volume mounts --

    #[test]
    fn build_maps_named_volumes() {
        let mut config = minimal_config();
        config.volumes.insert(
            "cache-vol".to_string(),
            crate::config::project::VolumeMount {
                target: "/cache".to_string(),
            },
        );

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        let cache_mount = av.volume_mounts.iter().find(|v| v.name == "cache-vol");
        assert!(cache_mount.is_some());
        assert_eq!(cache_mount.unwrap().mount_path, "/cache");
    }

    #[test]
    fn build_with_additional_mounts() {
        let mut config = minimal_config();
        config.workspace.additional_mounts.push(MountSpec {
            source: "/host/data".to_string(),
            target: "/container/data".to_string(),
            read_only: true,
        });

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        let data_mount = av.volume_mounts.iter().find(|v| v.mount_path == "/container/data");
        assert!(data_mount.is_some());
        assert!(data_mount.unwrap().read_only);
    }

    #[test]
    fn build_claude_oauth_has_auth_volume() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Oauth,
        });

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        let auth_mount = av.volume_mounts.iter().find(|v| v.mount_path.contains(".credentials.json"));
        assert!(auth_mount.is_some());
        assert!(auth_mount.unwrap().read_only);
    }

    // -- Workspace config --

    #[test]
    fn build_uses_workspace_config() {
        let mut config = minimal_config();
        config.workspace.mount_path = "/app".to_string();
        config.helm.default_pvc_size = "50Gi".to_string();

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.workspace_mount_path, "/app");
        assert_eq!(av.workspace_pvc_size, "50Gi");
    }

    // -- Agent type --

    #[test]
    fn build_codex_agent_type() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Codex, &infra_env);

        assert_eq!(av.agent_type, "codex");
    }

    #[test]
    fn build_both_agent_type() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Both, &infra_env);

        assert_eq!(av.agent_type, "both");
        // "both" type doesn't add auth env from secret (caller should split)
        assert!(av.env_from_secret.is_empty());
    }

    // -- Volume definitions match volume mounts --

    #[test]
    fn build_volumes_match_volume_mounts_for_named_volumes() {
        let mut config = minimal_config();
        config.volumes.insert(
            "data-vol".to_string(),
            crate::config::project::VolumeMount {
                target: "/data".to_string(),
            },
        );

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        assert_eq!(av.volumes.len(), 1);
        assert_eq!(av.volume_mounts.len(), 1);
        assert_eq!(av.volumes[0].name, "data-vol");
        assert_eq!(av.volume_mounts[0].name, "data-vol");
        assert_eq!(
            av.volumes[0].pvc_claim_name,
            Some("test-project-data-vol".to_string())
        );
        assert!(av.volumes[0].secret_name.is_none());
        assert!(!av.volumes[0].empty_dir);
    }

    #[test]
    fn build_volumes_match_volume_mounts_for_auth() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Oauth,
        });

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        // OAuth produces at least one auth volume
        let auth_vols: Vec<_> = av.volumes.iter().filter(|v| v.name.starts_with("auth-")).collect();
        let auth_mounts: Vec<_> = av.volume_mounts.iter().filter(|v| v.name.starts_with("auth-")).collect();

        assert!(!auth_vols.is_empty());
        assert_eq!(auth_vols.len(), auth_mounts.len());

        for (vol, mount) in auth_vols.iter().zip(auth_mounts.iter()) {
            assert_eq!(vol.name, mount.name);
            assert_eq!(vol.secret_name, Some("test-project-secrets".to_string()));
        }
    }

    #[test]
    fn build_volumes_match_volume_mounts_for_additional_mounts() {
        let mut config = minimal_config();
        config.workspace.additional_mounts.push(MountSpec {
            source: "/host/tmp".to_string(),
            target: "/tmp/data".to_string(),
            read_only: false,
        });

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        let mount_vols: Vec<_> = av.volumes.iter().filter(|v| v.name.starts_with("mount-")).collect();
        let mount_mounts: Vec<_> = av.volume_mounts.iter().filter(|v| v.name.starts_with("mount-")).collect();

        assert_eq!(mount_vols.len(), 1);
        assert_eq!(mount_mounts.len(), 1);
        assert_eq!(mount_vols[0].name, mount_mounts[0].name);
        assert!(mount_vols[0].empty_dir);
        assert!(mount_vols[0].pvc_claim_name.is_none());
        assert!(mount_vols[0].secret_name.is_none());
    }

    #[test]
    fn build_volumes_all_types_combined() {
        let mut config = minimal_config();

        // Named volume
        config.volumes.insert(
            "cache".to_string(),
            crate::config::project::VolumeMount {
                target: "/cache".to_string(),
            },
        );

        // Auth volume (OAuth produces file mounts)
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Oauth,
        });

        // Additional mount
        config.workspace.additional_mounts.push(MountSpec {
            source: "/host/scratch".to_string(),
            target: "/scratch".to_string(),
            read_only: true,
        });

        let infra_env = IndexMap::new();
        let av = build(&config, AgentType::Claude, &infra_env);

        // Every volume_mount must have a matching volume definition by name
        assert_eq!(av.volumes.len(), av.volume_mounts.len());
        for mount in &av.volume_mounts {
            let matching_vol = av.volumes.iter().find(|v| v.name == mount.name);
            assert!(
                matching_vol.is_some(),
                "volume_mount '{}' has no matching volume definition",
                mount.name,
            );
        }
    }
}
