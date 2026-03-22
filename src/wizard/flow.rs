use crate::config::project::*;
use indexmap::IndexMap;
use std::path::Path;

use super::prompts;

/// Run the interactive init wizard and return a ProjectConfig.
pub fn run(target: &Path) -> crate::error::Result<ProjectConfig> {
    let default_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    let name = prompts::input_project_name(default_name)
        .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;

    // Agent type
    let agent_idx = prompts::select_agent_type()
        .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;
    let agent_type = match agent_idx {
        0 => AgentType::Claude,
        1 => AgentType::Codex,
        _ => AgentType::Both,
    };

    // Base OS
    let os_idx = prompts::select_base_os()
        .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;
    let (base, base_version) = match os_idx {
        0 => (BaseOs::Ubuntu, "24.04".to_string()),
        1 => (BaseOs::Debian, "bookworm".to_string()),
        _ => (BaseOs::Alpine, "3.21".to_string()),
    };

    // Shell
    let shell_idx = prompts::select_shell()
        .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;
    let shell = match shell_idx {
        0 => ShellType::Bash,
        1 => ShellType::Zsh,
        _ => ShellType::Sh,
    };

    // Auth
    let auth = build_auth(agent_type)?;

    // Languages
    let lang_indices = prompts::select_languages()
        .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;
    let mut modules = IndexMap::new();

    let lang_map = [
        ("node", "22"),
        ("python", "3.12"),
        ("rust", "stable"),
        ("go", "1.23"),
        ("java", "21"),
        ("ruby", "3.3"),
        ("dotnet", "8.0"),
        ("zig", "0.13"),
        ("cpp", ""),
    ];
    for idx in &lang_indices {
        let (name, version) = lang_map[*idx];
        let mut params = toml::map::Map::new();
        if !version.is_empty() {
            params.insert("version".to_string(), toml::Value::String(version.to_string()));
        }
        modules.insert(name.to_string(), toml::Value::Table(params));
    }

    // Ensure node is always present (required by agents)
    if !modules.contains_key("node") {
        let mut params = toml::map::Map::new();
        params.insert("version".to_string(), toml::Value::String("22".to_string()));
        modules.insert("node".to_string(), toml::Value::Table(params));
    }

    // Tools
    let tool_indices = prompts::select_tools()
        .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;
    let tool_map = ["git", "docker-cli", "build-essential"];
    for idx in &tool_indices {
        modules.insert(
            tool_map[*idx].to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
    }

    // Services
    let svc_indices = prompts::select_services()
        .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;
    let mut services = IndexMap::new();
    let svc_map = [
        ("postgres", 5432u16),
        ("mysql", 3306),
        ("mongodb", 27017),
        ("redis", 6379),
        ("rabbitmq", 5672),
        ("kafka", 9092),
        ("elasticsearch", 9200),
        ("meilisearch", 7700),
        ("minio", 9000),
        ("prometheus", 9090),
        ("grafana", 3000),
    ];
    for idx in &svc_indices {
        let (name, port) = svc_map[*idx];
        services.insert(
            name.to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: Some(port),
                extra: IndexMap::new(),
            },
        );
    }

    // Firewall
    let firewall_enabled = prompts::confirm_firewall()
        .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;

    let mut runtime = RuntimeConfig::default();
    if firewall_enabled {
        runtime.cap_add.push("NET_ADMIN".to_string());
        runtime.cap_add.push("NET_RAW".to_string());
    }

    Ok(ProjectConfig {
        project: ProjectMeta {
            name,
            description: None,
        },
        agent: AgentConfig {
            agent_type,
            claude_version: "latest".to_string(),
            codex_version: "latest".to_string(),
        },
        image: ImageConfig {
            base,
            base_version,
            platform: "linux/amd64".to_string(),
            tag: None,
            user: "dev".to_string(),
            shell,
        },
        modules,
        auth,
        firewall: FirewallConfig {
            enabled: firewall_enabled,
            ..Default::default()
        },
        workspace: WorkspaceConfig::default(),
        volumes: IndexMap::new(),
        environment: EnvironmentConfig::default(),
        services,
        mcp: IndexMap::new(),
        runtime,
    })
}

fn build_auth(agent_type: AgentType) -> crate::error::Result<AuthConfig> {
    let claude = match agent_type {
        AgentType::Claude | AgentType::Both => {
            let idx = prompts::select_claude_auth()
                .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;
            let method = match idx {
                0 => ClaudeAuthMethod::ApiKey,
                1 => ClaudeAuthMethod::Oauth,
                2 => ClaudeAuthMethod::Bedrock,
                3 => ClaudeAuthMethod::BedrockApiKey,
                4 => ClaudeAuthMethod::Vertex,
                _ => ClaudeAuthMethod::Proxy,
            };
            Some(ClaudeAuthConfig { method })
        }
        AgentType::Codex => None,
    };

    let codex = match agent_type {
        AgentType::Codex | AgentType::Both => {
            let idx = prompts::select_codex_auth()
                .map_err(|e| crate::error::Error::Other(format!("prompt error: {e}")))?;
            let method = match idx {
                0 => CodexAuthMethod::ApiKey,
                1 => CodexAuthMethod::Oauth,
                2 => CodexAuthMethod::Azure,
                _ => CodexAuthMethod::Custom,
            };
            Some(CodexAuthConfig {
                method,
                azure_endpoint: None,
                custom_env_key: None,
                custom_base_url: None,
            })
        }
        AgentType::Claude => None,
    };

    Ok(AuthConfig { claude, codex })
}
