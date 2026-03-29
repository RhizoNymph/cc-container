use crate::config::project::AgentType;
use clap::Parser;

#[derive(Parser)]
pub struct InitArgs {
    /// Starter template
    #[arg(long)]
    pub template: Option<InitTemplate>,

    /// Agent type
    #[arg(long)]
    pub agent: Option<AgentType>,

    /// Skip interactive prompts, use defaults
    #[arg(long)]
    pub no_interactive: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum InitTemplate {
    Claude,
    Codex,
    Both,
    Minimal,
}

pub fn run(args: &InitArgs, global: &super::GlobalOpts) -> crate::error::Result<()> {
    let target = global
        .target_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));

    // Check for existing config BEFORE running wizard
    let config_path = target.join("cc-container.toml");
    if config_path.exists() {
        eprintln!("Config file already exists: {}", config_path.display());
        return Err(crate::error::Error::Other(
            "cc-container.toml already exists. Remove it first or use a different directory."
                .to_string(),
        ));
    }

    let config = if let Some(ref template) = args.template {
        generate_template_config(&target, template)
    } else if args.no_interactive {
        let agent_type = args.agent.unwrap_or(AgentType::Claude);
        generate_default_config(&target, agent_type)
    } else {
        crate::wizard::flow::run(&target)?
    };

    std::fs::create_dir_all(&target)?;
    let toml_str = toml::to_string_pretty(&config).map_err(crate::error::Error::TomlSerialize)?;
    std::fs::write(&config_path, toml_str)?;
    eprintln!("Created {}", config_path.display());

    Ok(())
}

fn generate_default_config(
    target: &std::path::Path,
    agent_type: AgentType,
) -> crate::config::project::ProjectConfig {
    use crate::config::project::*;
    use indexmap::IndexMap;

    let project_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    let auth = match agent_type {
        AgentType::Claude => AuthConfig {
            claude: Some(ClaudeAuthConfig {
                method: ClaudeAuthMethod::ApiKey,
            }),
            codex: None,
        },
        AgentType::Codex => AuthConfig {
            claude: None,
            codex: Some(CodexAuthConfig {
                method: CodexAuthMethod::ApiKey,
                azure_endpoint: None,
                custom_env_key: None,
                custom_base_url: None,
            }),
        },
        AgentType::Both => AuthConfig {
            claude: Some(ClaudeAuthConfig {
                method: ClaudeAuthMethod::ApiKey,
            }),
            codex: Some(CodexAuthConfig {
                method: CodexAuthMethod::ApiKey,
                azure_endpoint: None,
                custom_env_key: None,
                custom_base_url: None,
            }),
        },
    };

    // Add node module by default (required by both agents)
    let mut modules = IndexMap::new();
    let mut node_params = toml::map::Map::new();
    node_params.insert("version".to_string(), toml::Value::String("22".to_string()));
    modules.insert("node".to_string(), toml::Value::Table(node_params));
    modules.insert("git".to_string(), toml::Value::Table(toml::map::Map::new()));

    ProjectConfig {
        project: ProjectMeta {
            name: project_name,
            description: None,
        },
        agent: AgentConfig {
            agent_type,
            claude_version: "latest".to_string(),
            codex_version: "latest".to_string(),
        },
        image: ImageConfig::default(),
        modules,
        auth,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::AgentType;

    // --- generate_default_config tests ---

    #[test]
    fn default_config_claude_agent() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_default_config(dir.path(), AgentType::Claude);

        assert_eq!(config.agent.agent_type, AgentType::Claude);
        assert!(config.auth.claude.is_some());
        assert!(config.auth.codex.is_none());
        assert!(config.modules.contains_key("node"));
        assert!(config.modules.contains_key("git"));
    }

    #[test]
    fn default_config_codex_agent() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_default_config(dir.path(), AgentType::Codex);

        assert_eq!(config.agent.agent_type, AgentType::Codex);
        assert!(config.auth.claude.is_none());
        assert!(config.auth.codex.is_some());
    }

    #[test]
    fn default_config_both_agents() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_default_config(dir.path(), AgentType::Both);

        assert_eq!(config.agent.agent_type, AgentType::Both);
        assert!(config.auth.claude.is_some());
        assert!(config.auth.codex.is_some());
    }

    #[test]
    fn default_config_uses_dir_name_as_project_name() {
        let dir = tempfile::tempdir().unwrap();
        let expected_name = dir
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap()
            .to_string();
        let config = generate_default_config(dir.path(), AgentType::Claude);
        assert_eq!(config.project.name, expected_name);
    }

    #[test]
    fn default_config_node_version_is_22() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_default_config(dir.path(), AgentType::Claude);
        let node = config.modules.get("node").unwrap();
        if let toml::Value::Table(params) = node {
            assert_eq!(params.get("version").unwrap().as_str().unwrap(), "22");
        } else {
            panic!("expected node module to be a table");
        }
    }

    #[test]
    fn default_config_agent_versions_are_latest() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_default_config(dir.path(), AgentType::Claude);
        assert_eq!(config.agent.claude_version, "latest");
        assert_eq!(config.agent.codex_version, "latest");
    }

    // --- generate_template_config tests ---

    #[test]
    fn template_config_claude() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_template_config(dir.path(), &InitTemplate::Claude);

        assert_eq!(config.agent.agent_type, AgentType::Claude);
        assert!(config.auth.claude.is_some());
        assert!(config.auth.codex.is_none());
        // Claude template includes services (node + git)
        assert!(config.modules.contains_key("node"));
        assert!(config.modules.contains_key("git"));
    }

    #[test]
    fn template_config_codex() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_template_config(dir.path(), &InitTemplate::Codex);

        assert_eq!(config.agent.agent_type, AgentType::Codex);
        assert!(config.auth.claude.is_none());
        assert!(config.auth.codex.is_some());
    }

    #[test]
    fn template_config_both() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_template_config(dir.path(), &InitTemplate::Both);

        assert_eq!(config.agent.agent_type, AgentType::Both);
        assert!(config.auth.claude.is_some());
        assert!(config.auth.codex.is_some());
    }

    #[test]
    fn template_config_minimal_no_git() {
        let dir = tempfile::tempdir().unwrap();
        let config = generate_template_config(dir.path(), &InitTemplate::Minimal);

        assert_eq!(config.agent.agent_type, AgentType::Claude);
        assert!(config.modules.contains_key("node"));
        // Minimal does not include git
        assert!(!config.modules.contains_key("git"));
    }

    // --- run() integration tests ---

    #[test]
    fn run_no_interactive_creates_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let args = InitArgs {
            template: None,
            agent: None,
            no_interactive: true,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: None,
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();

        let config_path = dir.path().join("cc-container.toml");
        assert!(config_path.exists());

        // Verify the config can be parsed back
        let content = std::fs::read_to_string(&config_path).unwrap();
        let _config: crate::config::project::ProjectConfig = toml::from_str(&content).unwrap();
    }

    #[test]
    fn run_with_template_creates_config() {
        let dir = tempfile::tempdir().unwrap();
        let args = InitArgs {
            template: Some(InitTemplate::Minimal),
            agent: None,
            no_interactive: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: None,
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();

        let config_path = dir.path().join("cc-container.toml");
        assert!(config_path.exists());
    }

    #[test]
    fn run_errors_when_config_exists() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("cc-container.toml");
        std::fs::write(&config_path, "# existing").unwrap();

        let args = InitArgs {
            template: Some(InitTemplate::Claude),
            agent: None,
            no_interactive: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: None,
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        let result = run(&args, &global);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("already exists"));
    }

    #[test]
    fn run_no_interactive_with_codex_agent() {
        let dir = tempfile::tempdir().unwrap();
        let args = InitArgs {
            template: None,
            agent: Some(AgentType::Codex),
            no_interactive: true,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: None,
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();

        let content = std::fs::read_to_string(dir.path().join("cc-container.toml")).unwrap();
        let config: crate::config::project::ProjectConfig = toml::from_str(&content).unwrap();
        assert_eq!(config.agent.agent_type, AgentType::Codex);
    }

    #[test]
    fn run_creates_target_directory_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("nested").join("project");

        let args = InitArgs {
            template: Some(InitTemplate::Claude),
            agent: None,
            no_interactive: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(sub.clone()),
            config: None,
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(sub.join("cc-container.toml").exists());
    }
}

fn generate_template_config(
    target: &std::path::Path,
    template: &InitTemplate,
) -> crate::config::project::ProjectConfig {
    use crate::config::project::*;
    use indexmap::IndexMap;

    let project_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    let (agent_type, include_services) = match template {
        InitTemplate::Claude => (AgentType::Claude, true),
        InitTemplate::Codex => (AgentType::Codex, true),
        InitTemplate::Both => (AgentType::Both, true),
        InitTemplate::Minimal => (AgentType::Claude, false),
    };

    let auth = match agent_type {
        AgentType::Claude => AuthConfig {
            claude: Some(ClaudeAuthConfig {
                method: ClaudeAuthMethod::ApiKey,
            }),
            codex: None,
        },
        AgentType::Codex => AuthConfig {
            claude: None,
            codex: Some(CodexAuthConfig {
                method: CodexAuthMethod::ApiKey,
                azure_endpoint: None,
                custom_env_key: None,
                custom_base_url: None,
            }),
        },
        AgentType::Both => AuthConfig {
            claude: Some(ClaudeAuthConfig {
                method: ClaudeAuthMethod::ApiKey,
            }),
            codex: Some(CodexAuthConfig {
                method: CodexAuthMethod::ApiKey,
                azure_endpoint: None,
                custom_env_key: None,
                custom_base_url: None,
            }),
        },
    };

    let mut modules = IndexMap::new();
    let mut node_params = toml::map::Map::new();
    node_params.insert("version".to_string(), toml::Value::String("22".to_string()));
    modules.insert("node".to_string(), toml::Value::Table(node_params));

    if include_services {
        modules.insert("git".to_string(), toml::Value::Table(toml::map::Map::new()));
    }

    ProjectConfig {
        project: ProjectMeta {
            name: project_name,
            description: None,
        },
        agent: AgentConfig {
            agent_type,
            claude_version: "latest".to_string(),
            codex_version: "latest".to_string(),
        },
        image: ImageConfig::default(),
        modules,
        auth,
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
