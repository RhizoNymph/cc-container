use crate::auth;
use crate::config::project::{AgentType, ProjectConfig};
use docker_compose_types as dct;
use indexmap::IndexMap;

/// Build the agent container service definition.
pub fn build(
    config: &ProjectConfig,
    agent_type: AgentType,
    infra_env: &IndexMap<String, String>,
    depends_on: &[String],
    dockerfile_name: &str,
) -> dct::Service {
    let container_user = &config.image.user;

    // Collect environment variables
    let mut env: IndexMap<String, Option<dct::SingleValue>> = IndexMap::new();

    // Auth env vars
    let auth_reqs = get_auth_requirements(config, agent_type, container_user);
    for (key, val) in &auth_reqs.env_vars {
        env.insert(key.clone(), Some(dct::SingleValue::String(val.clone())));
    }

    // Infrastructure service connection env vars
    for (key, val) in infra_env {
        env.insert(key.clone(), Some(dct::SingleValue::String(val.clone())));
    }

    // User-defined environment variables
    for (key, val) in &config.environment.vars {
        env.insert(key.clone(), Some(dct::SingleValue::String(val.clone())));
    }

    // Volumes
    let mut volumes = Vec::new();

    // Workspace mount
    volumes.push(dct::Volumes::Simple(format!(
        "./:{}",
        config.workspace.mount_path
    )));

    // Named volumes for persistence
    for (name, vol) in &config.volumes {
        volumes.push(dct::Volumes::Simple(format!("{}:{}", name, vol.target)));
    }

    // Auth volume mounts (OAuth credential files, Vertex creds, etc.)
    for auth_vol in &auth_reqs.volumes {
        let mount = if auth_vol.read_only {
            format!("{}:{}:ro", auth_vol.source, auth_vol.target)
        } else {
            format!("{}:{}", auth_vol.source, auth_vol.target)
        };
        volumes.push(dct::Volumes::Simple(mount));
    }

    // Additional workspace mounts
    for mount in &config.workspace.additional_mounts {
        let m = if mount.read_only {
            format!("{}:{}:ro", mount.source, mount.target)
        } else {
            format!("{}:{}", mount.source, mount.target)
        };
        volumes.push(dct::Volumes::Simple(m));
    }

    // Env files
    let env_file = config
        .environment
        .env_files
        .as_ref()
        .map(|ef| dct::StringOrList::List(ef.files.clone()))
        .unwrap_or_else(|| dct::StringOrList::Simple(".env".to_string()));

    // depends_on
    let depends = if depends_on.is_empty() {
        IndexMap::new()
    } else {
        depends_on
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    dct::DependsCondition::service_healthy(),
                )
            })
            .collect()
    };

    // Build step: use AdvancedBuildStep when a non-default Dockerfile is needed
    let build_step = if dockerfile_name == "Dockerfile" {
        dct::BuildStep::Simple(".".to_string())
    } else {
        dct::BuildStep::Advanced(dct::AdvancedBuildStep {
            context: ".".to_string(),
            dockerfile: Some(dockerfile_name.to_string()),
            ..Default::default()
        })
    };

    // Build deploy config if cpu_limit is set
    let deploy = config.runtime.cpu_limit.as_ref().map(|cpu| dct::Deploy {
        resources: Some(dct::Resources {
            limits: Some(dct::Limits {
                cpus: Some(cpu.clone()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    });

    dct::Service {
        build_: Some(build_step),
        environment: dct::Environment::KvPair(env),
        volumes,
        env_file: Some(env_file),
        working_dir: Some(config.workspace.mount_path.clone()),
        stdin_open: true,
        tty: true,
        depends_on: dct::DependsOnOptions::Conditional(depends),
        cap_add: config.runtime.cap_add.clone(),
        cap_drop: config.runtime.cap_drop.clone(),
        security_opt: config.runtime.security_opt.clone(),
        mem_limit: config.runtime.memory_limit.clone(),
        deploy,
        shm_size: config.runtime.shm_size.clone(),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
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
            // Should not happen — caller splits into individual agents
        }
    }

    reqs
}
