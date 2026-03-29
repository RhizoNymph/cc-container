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

    #[test]
    fn test_build_minimal_agent() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        // Build step should be simple "."
        match &svc.build_ {
            Some(dct::BuildStep::Simple(ctx)) => assert_eq!(ctx, "."),
            _ => panic!("Expected simple build step"),
        }

        // Should have working_dir set
        assert_eq!(svc.working_dir, Some("/workspace".to_string()));

        // stdin_open and tty should be true
        assert!(svc.stdin_open);
        assert!(svc.tty);

        // restart policy
        assert_eq!(svc.restart, Some("unless-stopped".to_string()));

        // Default env_file should be ".env"
        match &svc.env_file {
            Some(dct::StringOrList::Simple(s)) => assert_eq!(s, ".env"),
            _ => panic!("Expected simple .env env_file"),
        }
    }

    #[test]
    fn test_build_advanced_build_step() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile.claude");

        match &svc.build_ {
            Some(dct::BuildStep::Advanced(adv)) => {
                assert_eq!(adv.context, ".");
                assert_eq!(adv.dockerfile, Some("Dockerfile.claude".to_string()));
            }
            _ => panic!("Expected advanced build step"),
        }
    }

    #[test]
    fn test_build_with_infra_env() {
        let config = minimal_config();
        let infra_env = IndexMap::from([
            ("DATABASE_URL".to_string(), "postgres://dev:pw@postgres:5432/devdb".to_string()),
            ("REDIS_URL".to_string(), "redis://redis:6379".to_string()),
        ]);
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        if let dct::Environment::KvPair(env) = &svc.environment {
            assert!(env.contains_key("DATABASE_URL"));
            assert!(env.contains_key("REDIS_URL"));
        } else {
            panic!("Expected KvPair environment");
        }
    }

    #[test]
    fn test_build_with_depends_on() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let depends_on = vec!["postgres".to_string(), "redis".to_string()];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        if let dct::DependsOnOptions::Conditional(deps) = &svc.depends_on {
            assert_eq!(deps.len(), 2);
            assert!(deps.contains_key("postgres"));
            assert!(deps.contains_key("redis"));
        } else {
            panic!("Expected conditional depends_on");
        }
    }

    #[test]
    fn test_build_with_empty_depends_on() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        if let dct::DependsOnOptions::Conditional(deps) = &svc.depends_on {
            assert!(deps.is_empty());
        } else {
            panic!("Expected conditional depends_on");
        }
    }

    #[test]
    fn test_build_with_user_env_vars() {
        let mut config = minimal_config();
        config.environment.vars.insert("MY_VAR".to_string(), "my_value".to_string());
        config.environment.vars.insert("ANOTHER".to_string(), "val2".to_string());

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        if let dct::Environment::KvPair(env) = &svc.environment {
            assert_eq!(
                env.get("MY_VAR"),
                Some(&Some(dct::SingleValue::String("my_value".to_string())))
            );
            assert_eq!(
                env.get("ANOTHER"),
                Some(&Some(dct::SingleValue::String("val2".to_string())))
            );
        } else {
            panic!("Expected KvPair environment");
        }
    }

    #[test]
    fn test_build_with_user_volumes() {
        let mut config = minimal_config();
        config.volumes.insert(
            "cache-vol".to_string(),
            VolumeMount {
                target: "/cache".to_string(),
            },
        );

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        let vol_strs: Vec<String> = svc
            .volumes
            .iter()
            .filter_map(|v| match v {
                dct::Volumes::Simple(s) => Some(s.clone()),
                _ => None,
            })
            .collect();

        assert!(vol_strs.contains(&"cache-vol:/cache".to_string()));
    }

    #[test]
    fn test_build_workspace_mount() {
        let mut config = minimal_config();
        config.workspace.mount_path = "/app".to_string();

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        let vol_strs: Vec<String> = svc
            .volumes
            .iter()
            .filter_map(|v| match v {
                dct::Volumes::Simple(s) => Some(s.clone()),
                _ => None,
            })
            .collect();

        assert!(vol_strs.contains(&"./:/app".to_string()));
        assert_eq!(svc.working_dir, Some("/app".to_string()));
    }

    #[test]
    fn test_build_with_additional_mounts() {
        let mut config = minimal_config();
        config.workspace.additional_mounts.push(MountSpec {
            source: "/host/data".to_string(),
            target: "/container/data".to_string(),
            read_only: false,
        });
        config.workspace.additional_mounts.push(MountSpec {
            source: "/host/config".to_string(),
            target: "/container/config".to_string(),
            read_only: true,
        });

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        let vol_strs: Vec<String> = svc
            .volumes
            .iter()
            .filter_map(|v| match v {
                dct::Volumes::Simple(s) => Some(s.clone()),
                _ => None,
            })
            .collect();

        assert!(vol_strs.contains(&"/host/data:/container/data".to_string()));
        assert!(vol_strs.contains(&"/host/config:/container/config:ro".to_string()));
    }

    #[test]
    fn test_build_with_claude_api_key_auth() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::ApiKey,
        });

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        if let dct::Environment::KvPair(env) = &svc.environment {
            assert!(env.contains_key("ANTHROPIC_API_KEY"));
        } else {
            panic!("Expected KvPair environment");
        }
    }

    #[test]
    fn test_build_with_claude_oauth_auth() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Oauth,
        });

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        // OAuth mounts a credentials file
        let vol_strs: Vec<String> = svc
            .volumes
            .iter()
            .filter_map(|v| match v {
                dct::Volumes::Simple(s) => Some(s.clone()),
                _ => None,
            })
            .collect();

        let has_cred_mount = vol_strs.iter().any(|v| v.contains(".credentials.json"));
        assert!(has_cred_mount, "Should have credential volume mount");
    }

    #[test]
    fn test_build_with_codex_api_key_auth() {
        let mut config = minimal_config();
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::ApiKey,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Codex, &infra_env, &depends_on, "Dockerfile");

        if let dct::Environment::KvPair(env) = &svc.environment {
            assert!(env.contains_key("OPENAI_API_KEY"));
        } else {
            panic!("Expected KvPair environment");
        }
    }

    #[test]
    fn test_build_with_env_files() {
        let mut config = minimal_config();
        config.environment.env_files = Some(EnvFilesConfig {
            files: vec![".env.local".to_string(), ".env.production".to_string()],
        });

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        match &svc.env_file {
            Some(dct::StringOrList::List(files)) => {
                assert_eq!(files.len(), 2);
                assert!(files.contains(&".env.local".to_string()));
                assert!(files.contains(&".env.production".to_string()));
            }
            _ => panic!("Expected list of env files"),
        }
    }

    #[test]
    fn test_build_with_runtime_caps() {
        let mut config = minimal_config();
        config.runtime.cap_add = vec!["SYS_PTRACE".to_string()];
        config.runtime.cap_drop = vec!["NET_RAW".to_string()];
        config.runtime.security_opt = vec!["no-new-privileges:true".to_string()];
        config.runtime.memory_limit = Some("4g".to_string());
        config.runtime.shm_size = Some("2g".to_string());

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        assert_eq!(svc.cap_add, vec!["SYS_PTRACE".to_string()]);
        assert_eq!(svc.cap_drop, vec!["NET_RAW".to_string()]);
        assert_eq!(svc.security_opt, vec!["no-new-privileges:true".to_string()]);
        assert_eq!(svc.mem_limit, Some("4g".to_string()));
        assert_eq!(svc.shm_size, Some("2g".to_string()));
    }

    #[test]
    fn test_build_with_cpu_limit() {
        let mut config = minimal_config();
        config.runtime.cpu_limit = Some("2.0".to_string());

        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        let deploy = svc.deploy.unwrap();
        let resources = deploy.resources.unwrap();
        let limits = resources.limits.unwrap();
        assert_eq!(limits.cpus, Some("2.0".to_string()));
    }

    #[test]
    fn test_build_without_cpu_limit() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        assert!(svc.deploy.is_none());
    }

    #[test]
    fn test_build_no_auth_configured() {
        let config = minimal_config();
        let infra_env = IndexMap::new();
        let depends_on: Vec<String> = vec![];

        let svc = build(&config, AgentType::Claude, &infra_env, &depends_on, "Dockerfile");

        if let dct::Environment::KvPair(env) = &svc.environment {
            assert!(!env.contains_key("ANTHROPIC_API_KEY"));
            assert!(!env.contains_key("OPENAI_API_KEY"));
        } else {
            panic!("Expected KvPair environment");
        }
    }
}
