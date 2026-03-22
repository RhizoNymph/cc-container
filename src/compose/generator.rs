use crate::config::project::{AgentType, ProjectConfig};
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

use super::agent_service;
use super::service_templates;

/// Generate a complete docker-compose Compose struct from project config.
pub fn generate(config: &ProjectConfig) -> Result<dct::Compose> {
    let mut services: IndexMap<String, Option<dct::Service>> = IndexMap::new();
    let mut top_volumes: IndexMap<String, dct::MapOrEmpty<dct::ComposeVolume>> = IndexMap::new();

    // Collect infrastructure service env vars for agent containers
    let mut infra_env: IndexMap<String, String> = IndexMap::new();
    let mut infra_service_names: Vec<String> = Vec::new();

    // Build infrastructure services
    for (name, svc_config) in &config.services {
        if !svc_config.enabled {
            continue;
        }

        let (service, agent_env) = service_templates::build_service(name, svc_config)?;
        services.insert(name.clone(), Some(service));
        infra_env.extend(agent_env);
        infra_service_names.push(name.clone());

        // Collect volume names from service volumes
        // (services reference named volumes like "pgdata-devdb:/var/lib/...")
        // We'll add them to the top-level volumes section
    }

    // Build MCP server services
    for (name, mcp_config) in &config.mcp {
        let service_name = format!("mcp-{name}");

        let mut mcp_env: IndexMap<String, Option<dct::SingleValue>> = IndexMap::new();
        for env_var in &mcp_config.env {
            mcp_env.insert(env_var.clone(), Some(dct::SingleValue::String(format!("${{{env_var}}}"))));
        }

        let mcp_volumes: Vec<dct::Volumes> = mcp_config
            .volumes
            .iter()
            .map(|v| dct::Volumes::Simple(v.clone()))
            .collect();

        let command = mcp_config
            .command
            .as_ref()
            .map(|c| dct::Command::Args(c.clone()));

        let ports = mcp_config
            .port
            .map(|p| dct::Ports::Short(vec![format!("{p}:{p}")]))
            .unwrap_or(dct::Ports::Short(vec![]));

        let svc = dct::Service {
            image: Some(mcp_config.image.clone()),
            command,
            environment: dct::Environment::KvPair(mcp_env),
            volumes: mcp_volumes,
            ports,
            restart: Some("unless-stopped".to_string()),
            ..Default::default()
        };

        services.insert(service_name, Some(svc));
    }

    // Build agent service(s)
    match config.agent.agent_type {
        AgentType::Both => {
            let claude_svc = agent_service::build(
                config,
                AgentType::Claude,
                &infra_env,
                &infra_service_names,
                "Dockerfile.claude",
            );
            let codex_svc = agent_service::build(
                config,
                AgentType::Codex,
                &infra_env,
                &infra_service_names,
                "Dockerfile.codex",
            );
            services.insert("agent-claude".to_string(), Some(claude_svc));
            services.insert("agent-codex".to_string(), Some(codex_svc));
        }
        agent_type => {
            let svc = agent_service::build(
                config,
                agent_type,
                &infra_env,
                &infra_service_names,
                "Dockerfile",
            );
            services.insert("agent".to_string(), Some(svc));
        }
    }

    // Collect all named volumes referenced by services
    for (_name, svc_opt) in &services {
        if let Some(svc) = svc_opt {
            for vol in &svc.volumes {
                if let dct::Volumes::Simple(v) = vol {
                    if let Some((vol_name, _)) = v.split_once(':') {
                        // Only add named volumes (not paths starting with . or /)
                        if !vol_name.starts_with('.') && !vol_name.starts_with('/') && !vol_name.starts_with('~') && !vol_name.contains('$') {
                            top_volumes
                                .entry(vol_name.to_string())
                                .or_insert(dct::MapOrEmpty::Empty);
                        }
                    }
                }
            }
        }
    }

    // Also add user-defined named volumes
    for (name, _vol) in &config.volumes {
        top_volumes
            .entry(name.clone())
            .or_insert(dct::MapOrEmpty::Empty);
    }

    Ok(dct::Compose {
        services: dct::Services(services),
        volumes: dct::TopLevelVolumes(top_volumes),
        ..Default::default()
    })
}
