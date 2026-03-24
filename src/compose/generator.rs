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
    let mut db_url_sources: Vec<(String, String)> = Vec::new(); // (service_name, url_value)

    for (name, svc_config) in &config.services {
        if !svc_config.enabled {
            continue;
        }

        let (service, agent_env) = service_templates::build_service(name, svc_config)?;
        services.insert(name.clone(), Some(service));
        infra_service_names.push(name.clone());

        for (key, val) in agent_env {
            if key == "DATABASE_URL" {
                db_url_sources.push((name.clone(), val));
            } else {
                infra_env.insert(key, val);
            }
        }
    }

    // Handle DATABASE_URL: if only one database, use DATABASE_URL directly.
    // If multiple, use service-specific names and warn.
    match db_url_sources.len() {
        0 => {}
        1 => {
            let (_name, url) = db_url_sources.into_iter().next().unwrap();
            infra_env.insert("DATABASE_URL".to_string(), url);
        }
        _ => {
            eprintln!("warning: multiple services set DATABASE_URL; using service-specific env vars");
            for (name, url) in &db_url_sources {
                let specific_key = format!("{}_URL", name.to_uppercase());
                infra_env.insert(specific_key, url.clone());
            }
            // Also set DATABASE_URL to the first one as a convenience default
            let (name, url) = &db_url_sources[0];
            infra_env.insert("DATABASE_URL".to_string(), url.clone());
            eprintln!("  DATABASE_URL defaults to {} ({})", name, url);
        }
    }

    // Collect volume names from service volumes
    // (services reference named volumes like "pgdata-devdb:/var/lib/...")
    // We'll add them to the top-level volumes section

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
                if let dct::Volumes::Simple(v) = vol
                    && let Some((vol_name, _)) = v.split_once(':') {
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
