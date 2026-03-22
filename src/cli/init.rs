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

    let config = if args.no_interactive {
        let agent_type = args.agent.unwrap_or(AgentType::Claude);
        generate_default_config(&target, agent_type)
    } else {
        crate::wizard::flow::run(&target)?
    };

    let config_path = target.join("cc-container.toml");
    if config_path.exists() {
        eprintln!(
            "Config file already exists: {}",
            config_path.display()
        );
        return Err(crate::error::Error::Other(
            "cc-container.toml already exists. Remove it first or use a different directory."
                .to_string(),
        ));
    }

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
    }
}
