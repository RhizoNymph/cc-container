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
            eprintln!(
                "warning: multiple services set DATABASE_URL; using service-specific env vars"
            );
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
            if let Some((key, value)) = env_var.split_once('=') {
                mcp_env.insert(
                    key.to_string(),
                    Some(dct::SingleValue::String(value.to_string())),
                );
            } else {
                mcp_env.insert(
                    env_var.clone(),
                    Some(dct::SingleValue::String(format!("${{{env_var}}}"))),
                );
            }
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
                    && let Some((vol_name, _)) = v.split_once(':')
                {
                    // Only add named volumes (not paths starting with . or /)
                    if !vol_name.starts_with('.')
                        && !vol_name.starts_with('/')
                        && !vol_name.starts_with('~')
                        && !vol_name.contains('$')
                    {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::*;
    use indexmap::IndexMap;

    /// Helper: build a minimal valid ProjectConfig for testing.
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

    #[test]
    fn test_generate_minimal_config() {
        let config = minimal_config();
        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // Should have exactly one agent service
        assert_eq!(services.len(), 1);
        assert!(services.contains_key("agent"));
    }

    #[test]
    fn test_generate_with_single_service() {
        let mut config = minimal_config();
        config.services.insert(
            "postgres".to_string(),
            ServiceConfig {
                enabled: true,
                version: Some("15".to_string()),
                port: None,
                extra: IndexMap::new(),
            },
        );

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // Should have postgres + agent
        assert_eq!(services.len(), 2);
        assert!(services.contains_key("postgres"));
        assert!(services.contains_key("agent"));

        // Agent should depend on postgres
        let agent = services.get("agent").unwrap().as_ref().unwrap();
        if let dct::DependsOnOptions::Conditional(deps) = &agent.depends_on {
            assert!(deps.contains_key("postgres"));
        } else {
            panic!("Expected conditional depends_on");
        }

        // Top-level volumes should contain pgdata-devdb (default db name)
        let vols = &compose.volumes.0;
        assert!(vols.contains_key("pgdata-devdb"));
    }

    #[test]
    fn test_generate_with_disabled_service() {
        let mut config = minimal_config();
        config.services.insert(
            "redis".to_string(),
            ServiceConfig {
                enabled: false,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // Disabled service should not appear
        assert_eq!(services.len(), 1);
        assert!(!services.contains_key("redis"));
        assert!(services.contains_key("agent"));
    }

    #[test]
    fn test_generate_with_multiple_services() {
        let mut config = minimal_config();
        config.services.insert(
            "redis".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        config.services.insert(
            "memcached".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // Should have redis + memcached + agent
        assert_eq!(services.len(), 3);
        assert!(services.contains_key("redis"));
        assert!(services.contains_key("memcached"));
        assert!(services.contains_key("agent"));

        // Agent should depend on both
        let agent = services.get("agent").unwrap().as_ref().unwrap();
        if let dct::DependsOnOptions::Conditional(deps) = &agent.depends_on {
            assert!(deps.contains_key("redis"));
            assert!(deps.contains_key("memcached"));
        } else {
            panic!("Expected conditional depends_on");
        }
    }

    #[test]
    fn test_generate_both_agent_types() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // Should have two agent services
        assert!(services.contains_key("agent-claude"));
        assert!(services.contains_key("agent-codex"));
        assert!(!services.contains_key("agent"));
    }

    #[test]
    fn test_generate_codex_agent() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        assert_eq!(services.len(), 1);
        assert!(services.contains_key("agent"));

        // Agent service should use simple build step (Dockerfile)
        let agent = services.get("agent").unwrap().as_ref().unwrap();
        match &agent.build_ {
            Some(dct::BuildStep::Simple(ctx)) => assert_eq!(ctx, "."),
            _ => panic!("Expected simple build step for single agent"),
        }
    }

    #[test]
    fn test_generate_with_mcp_service() {
        let mut config = minimal_config();
        config.mcp.insert(
            "github".to_string(),
            McpServerConfig {
                image: "ghcr.io/modelcontextprotocol/github:latest".to_string(),
                command: Some(vec!["node".to_string(), "server.js".to_string()]),
                env: vec!["GITHUB_TOKEN".to_string()],
                volumes: vec!["/tmp/data:/data".to_string()],
                port: Some(3000),
            },
        );

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // MCP service should be named "mcp-github"
        assert!(services.contains_key("mcp-github"));

        let mcp_svc = services.get("mcp-github").unwrap().as_ref().unwrap();
        assert_eq!(
            mcp_svc.image,
            Some("ghcr.io/modelcontextprotocol/github:latest".to_string())
        );
        assert_eq!(mcp_svc.restart, Some("unless-stopped".to_string()));

        // Should have env var referencing ${GITHUB_TOKEN}
        if let dct::Environment::KvPair(env) = &mcp_svc.environment {
            assert!(env.contains_key("GITHUB_TOKEN"));
        } else {
            panic!("Expected KvPair environment");
        }

        // Should have volume
        assert!(!mcp_svc.volumes.is_empty());

        // Should have ports
        match &mcp_svc.ports {
            dct::Ports::Short(ports) => assert!(ports.contains(&"3000:3000".to_string())),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_generate_mcp_without_port() {
        let mut config = minimal_config();
        config.mcp.insert(
            "test".to_string(),
            McpServerConfig {
                image: "test-image:latest".to_string(),
                command: None,
                env: vec![],
                volumes: vec![],
                port: None,
            },
        );

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;
        let mcp_svc = services.get("mcp-test").unwrap().as_ref().unwrap();

        // Port should be an empty short list
        match &mcp_svc.ports {
            dct::Ports::Short(ports) => assert!(ports.is_empty()),
            _ => panic!("Expected empty short ports"),
        }
    }

    #[test]
    fn test_generate_multiple_database_urls() {
        let mut config = minimal_config();
        config.services.insert(
            "postgres".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        config.services.insert(
            "mysql".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        // Both postgres and mysql set DATABASE_URL; generator should handle the conflict
        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        assert!(services.contains_key("postgres"));
        assert!(services.contains_key("mysql"));

        // The agent should still be generated despite the conflict
        let agent = services.get("agent").unwrap().as_ref().unwrap();
        if let dct::Environment::KvPair(env) = &agent.environment {
            // Should have DATABASE_URL set to the first one
            assert!(env.contains_key("DATABASE_URL"));
            // Should also have service-specific URLs
            assert!(env.contains_key("POSTGRES_URL") || env.contains_key("MYSQL_URL"));
        } else {
            panic!("Expected KvPair environment");
        }
    }

    #[test]
    fn test_generate_user_defined_volumes() {
        let mut config = minimal_config();
        config.volumes.insert(
            "mydata".to_string(),
            VolumeMount {
                target: "/data".to_string(),
            },
        );

        let compose = generate(&config).unwrap();
        let vols = &compose.volumes.0;
        assert!(vols.contains_key("mydata"));
    }

    #[test]
    fn test_named_volumes_collected_from_services() {
        let mut config = minimal_config();
        config.services.insert(
            "redis".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let compose = generate(&config).unwrap();
        let vols = &compose.volumes.0;

        // Redis uses "redisdata:/data" so "redisdata" should be in top-level volumes
        assert!(vols.contains_key("redisdata"));
    }

    #[test]
    fn test_path_volumes_not_in_top_level() {
        // Volumes starting with . / ~ or containing $ should NOT be in top-level volumes
        let config = minimal_config();
        let compose = generate(&config).unwrap();
        let vols = &compose.volumes.0;

        // Workspace mount is ./:... which should not be in top-level volumes
        for key in vols.keys() {
            assert!(!key.starts_with('.'));
            assert!(!key.starts_with('/'));
            assert!(!key.starts_with('~'));
            assert!(!key.contains('$'));
        }
    }

    #[test]
    fn test_generate_both_agents_with_services() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;
        config.services.insert(
            "redis".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // Both agents + redis
        assert!(services.contains_key("agent-claude"));
        assert!(services.contains_key("agent-codex"));
        assert!(services.contains_key("redis"));

        // Both agents should depend on redis
        for agent_name in &["agent-claude", "agent-codex"] {
            let agent = services.get(*agent_name).unwrap().as_ref().unwrap();
            if let dct::DependsOnOptions::Conditional(deps) = &agent.depends_on {
                assert!(deps.contains_key("redis"));
            } else {
                panic!("{} should have conditional depends_on", agent_name);
            }
        }

        // Both agents should have REDIS_URL env
        for agent_name in &["agent-claude", "agent-codex"] {
            let agent = services.get(*agent_name).unwrap().as_ref().unwrap();
            if let dct::Environment::KvPair(env) = &agent.environment {
                assert!(
                    env.contains_key("REDIS_URL"),
                    "{} missing REDIS_URL",
                    agent_name
                );
            } else {
                panic!("Expected KvPair environment for {}", agent_name);
            }
        }
    }

    #[test]
    fn test_both_agents_use_advanced_build_step() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // Claude agent should use Dockerfile.claude
        let claude = services.get("agent-claude").unwrap().as_ref().unwrap();
        match &claude.build_ {
            Some(dct::BuildStep::Advanced(adv)) => {
                assert_eq!(adv.context, ".");
                assert_eq!(adv.dockerfile, Some("Dockerfile.claude".to_string()));
            }
            _ => panic!("Expected advanced build step for agent-claude"),
        }

        // Codex agent should use Dockerfile.codex
        let codex = services.get("agent-codex").unwrap().as_ref().unwrap();
        match &codex.build_ {
            Some(dct::BuildStep::Advanced(adv)) => {
                assert_eq!(adv.context, ".");
                assert_eq!(adv.dockerfile, Some("Dockerfile.codex".to_string()));
            }
            _ => panic!("Expected advanced build step for agent-codex"),
        }
    }

    #[test]
    fn test_generate_empty_services_list() {
        let config = minimal_config();
        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // Only agent service
        assert_eq!(services.len(), 1);
    }

    #[test]
    fn test_generate_all_service_types() {
        let mut config = minimal_config();
        let service_names = vec![
            "postgres",
            "mysql",
            "mariadb",
            "mongodb",
            "cockroachdb",
            "redis",
            "memcached",
            "rabbitmq",
            "kafka",
            "nats",
            "elasticsearch",
            "meilisearch",
            "typesense",
            "minio",
            "prometheus",
            "grafana",
            "traefik",
            "nginx",
        ];

        for name in &service_names {
            config.services.insert(
                name.to_string(),
                ServiceConfig {
                    enabled: true,
                    version: None,
                    port: None,
                    extra: IndexMap::new(),
                },
            );
        }

        let compose = generate(&config).unwrap();
        let services = &compose.services.0;

        // All services + agent
        assert_eq!(services.len(), service_names.len() + 1);
        for name in &service_names {
            assert!(services.contains_key(*name), "Missing service: {}", name);
        }
    }

    #[test]
    fn test_mcp_service_without_command() {
        let mut config = minimal_config();
        config.mcp.insert(
            "test".to_string(),
            McpServerConfig {
                image: "test:latest".to_string(),
                command: None,
                env: vec![],
                volumes: vec![],
                port: None,
            },
        );

        let compose = generate(&config).unwrap();
        let mcp = compose
            .services
            .0
            .get("mcp-test")
            .unwrap()
            .as_ref()
            .unwrap();
        assert!(mcp.command.is_none());
    }
}
