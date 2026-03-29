use crate::config::project::ProjectConfig;
use crate::error::Result;
use crate::helm::types::*;
use indexmap::IndexMap;

/// Build the complete `HelmValues` struct from a project config.
///
/// This orchestrates the value builders from sibling modules to assemble
/// the full values.yaml content for the generated Helm chart.
pub fn build(config: &ProjectConfig) -> Result<HelmValues> {
    let namespace = config
        .helm
        .namespace
        .clone()
        .unwrap_or_else(|| config.project.name.clone());

    // Build infrastructure services and collect agent env vars
    let mut services = IndexMap::new();
    let mut infra_env: IndexMap<String, String> = IndexMap::new();
    let mut db_url_sources: Vec<(String, String)> = Vec::new();

    for (name, svc_config) in &config.services {
        if !svc_config.enabled {
            continue;
        }

        let (svc_values, agent_env) = crate::helm::service_values::build_service(name, svc_config)?;
        services.insert(name.clone(), svc_values);

        for (key, val) in agent_env {
            if key == "DATABASE_URL" {
                db_url_sources.push((name.clone(), val));
            } else {
                infra_env.insert(key, val);
            }
        }
    }

    // Handle DATABASE_URL: if only one database, use DATABASE_URL directly.
    // If multiple, use service-specific names and set DATABASE_URL to the first.
    match db_url_sources.len() {
        0 => {}
        1 => {
            let (_name, url) = db_url_sources.into_iter().next().expect("checked len == 1");
            infra_env.insert("DATABASE_URL".to_string(), url);
        }
        _ => {
            for (name, url) in &db_url_sources {
                let specific_key = format!("{}_URL", name.to_uppercase());
                infra_env.insert(specific_key, url.clone());
            }
            // Default DATABASE_URL to the first one
            let (_name, url) = &db_url_sources[0];
            infra_env.insert("DATABASE_URL".to_string(), url.clone());
        }
    }

    // Build agent values
    let agent = crate::helm::agent_values::build(config, config.agent.agent_type, &infra_env);

    // Build network policy
    let network_policy = crate::helm::network_policy::build(config);

    // Build secrets
    let secrets = crate::helm::secrets::build(config);

    // Build MCP values
    let mut mcp = IndexMap::new();
    for (name, mcp_config) in &config.mcp {
        mcp.insert(
            name.clone(),
            McpValues {
                image: mcp_config.image.clone(),
                command: mcp_config.command.clone(),
                env_from_secret: mcp_config.env.clone(),
                ports: mcp_config
                    .port
                    .map(|p| {
                        vec![PortSpec {
                            name: "mcp".to_string(),
                            container_port: p,
                            protocol: "TCP".to_string(),
                        }]
                    })
                    .unwrap_or_default(),
            },
        );
    }

    // Build ingress
    let ingress = config.helm.ingress_host.as_ref().map(|host| IngressValues {
        class_name: config
            .helm
            .ingress_class
            .clone()
            .unwrap_or_else(|| "nginx".to_string()),
        host: host.clone(),
    });

    Ok(HelmValues {
        project_name: config.project.name.clone(),
        namespace,
        agent,
        services,
        mcp,
        network_policy,
        secrets,
        ingress,
    })
}
