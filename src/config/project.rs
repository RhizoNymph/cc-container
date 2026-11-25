use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Root project configuration (cc-container.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    pub agent: AgentConfig,
    #[serde(default)]
    pub image: ImageConfig,
    #[serde(default)]
    pub modules: IndexMap<String, toml::Value>,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub firewall: FirewallConfig,
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    #[serde(default)]
    pub volumes: IndexMap<String, VolumeMount>,
    #[serde(default)]
    pub environment: EnvironmentConfig,
    #[serde(default)]
    pub services: IndexMap<String, ServiceConfig>,
    #[serde(default)]
    pub mcp: IndexMap<String, McpServerConfig>,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub helm: HelmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(rename = "type")]
    pub agent_type: AgentType,
    #[serde(default = "default_latest")]
    pub claude_version: String,
    #[serde(default = "default_latest")]
    pub codex_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
    Both,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Claude => write!(f, "claude"),
            AgentType::Codex => write!(f, "codex"),
            AgentType::Both => write!(f, "both"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    #[serde(default = "default_ubuntu")]
    pub base: BaseOs,
    #[serde(default)]
    pub base_version: Option<String>,
    #[serde(default = "default_platform")]
    pub platform: String,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default = "default_shell")]
    pub shell: ShellType,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            base: BaseOs::Ubuntu,
            base_version: None,
            platform: default_platform(),
            tag: None,
            user: default_user(),
            shell: ShellType::Bash,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum BaseOs {
    Ubuntu,
    Debian,
    Alpine,
}

impl std::fmt::Display for BaseOs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseOs::Ubuntu => write!(f, "ubuntu"),
            BaseOs::Debian => write!(f, "debian"),
            BaseOs::Alpine => write!(f, "alpine"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Bash,
    Zsh,
    Sh,
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Bash => write!(f, "bash"),
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Sh => write!(f, "sh"),
        }
    }
}

// --- Auth ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub claude: Option<ClaudeAuthConfig>,
    #[serde(default)]
    pub codex: Option<CodexAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeAuthConfig {
    pub method: ClaudeAuthMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeAuthMethod {
    ApiKey,
    Oauth,
    Bedrock,
    BedrockApiKey,
    Vertex,
    Proxy,
}

impl std::fmt::Display for ClaudeAuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey => write!(f, "api-key"),
            Self::Oauth => write!(f, "oauth"),
            Self::Bedrock => write!(f, "bedrock"),
            Self::BedrockApiKey => write!(f, "bedrock-api-key"),
            Self::Vertex => write!(f, "vertex"),
            Self::Proxy => write!(f, "proxy"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthConfig {
    pub method: CodexAuthMethod,
    #[serde(default)]
    pub azure_endpoint: Option<String>,
    #[serde(default)]
    pub custom_env_key: Option<String>,
    #[serde(default)]
    pub custom_base_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CodexAuthMethod {
    ApiKey,
    Oauth,
    Azure,
    Custom,
}

impl std::fmt::Display for CodexAuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey => write!(f, "api-key"),
            Self::Oauth => write!(f, "oauth"),
            Self::Azure => write!(f, "azure"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

// --- Firewall ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub allowed_cidrs: Vec<String>,
    #[serde(default = "default_true")]
    pub allow_ssh: bool,
    #[serde(default = "default_true")]
    pub allow_dns: bool,
}

impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_domains: Vec::new(),
            allowed_cidrs: Vec::new(),
            allow_ssh: true,
            allow_dns: true,
        }
    }
}

// --- Workspace ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default = "default_mount_path")]
    pub mount_path: String,
    #[serde(default)]
    pub additional_mounts: Vec<MountSpec>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            mount_path: default_mount_path(),
            additional_mounts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountSpec {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub target: String,
}

// --- Environment ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentConfig {
    #[serde(default)]
    pub env_files: Option<EnvFilesConfig>,
    #[serde(flatten)]
    pub vars: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvFilesConfig {
    #[serde(default)]
    pub files: Vec<String>,
}

// --- Services ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(flatten)]
    pub extra: IndexMap<String, toml::Value>,
}

// --- MCP ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub image: String,
    #[serde(default)]
    pub command: Option<Vec<String>>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub volumes: Vec<String>,
    #[serde(default)]
    pub port: Option<u16>,
}

// --- Runtime ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub cap_add: Vec<String>,
    #[serde(default)]
    pub cap_drop: Vec<String>,
    #[serde(default)]
    pub security_opt: Vec<String>,
    #[serde(default)]
    pub memory_limit: Option<String>,
    #[serde(default)]
    pub cpu_limit: Option<String>,
    #[serde(default)]
    pub shm_size: Option<String>,
}

// --- Helm ---

/// Helm chart generation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmConfig {
    /// Container image registry for the agent image (e.g. "ghcr.io/myorg").
    #[serde(default)]
    pub image_registry: Option<String>,
    /// Repository name for the agent image within the registry.
    /// Defaults to the project name.
    #[serde(default)]
    pub image_repository: Option<String>,
    /// Tag for the agent image. Defaults to "latest".
    #[serde(default = "default_latest")]
    pub image_tag: String,
    /// Kubernetes namespace. Defaults to the project name.
    #[serde(default)]
    pub namespace: Option<String>,
    /// Storage class for PVCs. None means use cluster default.
    #[serde(default)]
    pub storage_class: Option<String>,
    /// Default PVC size for stateful services.
    #[serde(default = "default_pvc_size")]
    pub default_pvc_size: String,
    /// Ingress hostname. If set, generates an Ingress resource.
    #[serde(default)]
    pub ingress_host: Option<String>,
    /// Ingress class name (e.g. "nginx", "traefik").
    #[serde(default)]
    pub ingress_class: Option<String>,
}

impl Default for HelmConfig {
    fn default() -> Self {
        Self {
            image_registry: None,
            image_repository: None,
            image_tag: default_latest(),
            namespace: None,
            storage_class: None,
            default_pvc_size: default_pvc_size(),
            ingress_host: None,
            ingress_class: None,
        }
    }
}

fn default_pvc_size() -> String {
    "10Gi".to_string()
}

// --- Default helpers ---

fn default_true() -> bool {
    true
}

fn default_latest() -> String {
    "latest".to_string()
}

fn default_ubuntu() -> BaseOs {
    BaseOs::Ubuntu
}

/// Returns the default Docker tag for the given base OS.
pub fn default_version_for_os(os: BaseOs) -> &'static str {
    match os {
        BaseOs::Ubuntu => "24.04",
        BaseOs::Debian => "bookworm",
        BaseOs::Alpine => "3.21",
    }
}

fn default_platform() -> String {
    "linux/amd64".to_string()
}

fn default_user() -> String {
    "dev".to_string()
}

fn default_shell() -> ShellType {
    ShellType::Bash
}

fn default_mount_path() -> String {
    "/workspace".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────── Minimal valid config ─────────────────────

    const MINIMAL_CONFIG: &str = r#"
[project]
name = "test-project"

[agent]
type = "claude"
"#;

    #[test]
    fn parse_minimal_config() {
        let config: ProjectConfig = toml::from_str(MINIMAL_CONFIG).unwrap();
        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.agent.agent_type, AgentType::Claude);
    }

    #[test]
    fn minimal_config_defaults() {
        let config: ProjectConfig = toml::from_str(MINIMAL_CONFIG).unwrap();
        // Agent defaults
        assert_eq!(config.agent.claude_version, "latest");
        assert_eq!(config.agent.codex_version, "latest");
        // Image defaults
        assert_eq!(config.image.base, BaseOs::Ubuntu);
        assert!(config.image.base_version.is_none());
        assert_eq!(config.image.platform, "linux/amd64");
        assert!(config.image.tag.is_none());
        assert_eq!(config.image.user, "dev");
        assert_eq!(config.image.shell, ShellType::Bash);
        // Firewall defaults
        assert!(!config.firewall.enabled);
        assert!(config.firewall.allowed_domains.is_empty());
        assert!(config.firewall.allowed_cidrs.is_empty());
        assert!(config.firewall.allow_ssh);
        assert!(config.firewall.allow_dns);
        // Workspace defaults
        assert_eq!(config.workspace.mount_path, "/workspace");
        assert!(config.workspace.additional_mounts.is_empty());
        // Empty collections
        assert!(config.modules.is_empty());
        assert!(config.volumes.is_empty());
        assert!(config.services.is_empty());
        assert!(config.mcp.is_empty());
        // Runtime defaults
        assert!(config.runtime.cap_add.is_empty());
        assert!(config.runtime.cap_drop.is_empty());
        assert!(config.runtime.security_opt.is_empty());
        assert!(config.runtime.memory_limit.is_none());
        assert!(config.runtime.cpu_limit.is_none());
        assert!(config.runtime.shm_size.is_none());
    }

    // ───────────────────── Agent types ─────────────────────

    #[test]
    fn parse_agent_type_codex() {
        let toml_str = r#"
[project]
name = "codex-proj"
[agent]
type = "codex"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.agent.agent_type, AgentType::Codex);
    }

    #[test]
    fn parse_agent_type_both() {
        let toml_str = r#"
[project]
name = "both-proj"
[agent]
type = "both"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.agent.agent_type, AgentType::Both);
    }

    #[test]
    fn invalid_agent_type_fails() {
        let toml_str = r#"
[project]
name = "bad"
[agent]
type = "invalid"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn agent_type_display() {
        assert_eq!(AgentType::Claude.to_string(), "claude");
        assert_eq!(AgentType::Codex.to_string(), "codex");
        assert_eq!(AgentType::Both.to_string(), "both");
    }

    // ───────────────────── Image config ─────────────────────

    #[test]
    fn parse_image_config_all_fields() {
        let toml_str = r#"
[project]
name = "img-test"
[agent]
type = "claude"
[image]
base = "debian"
base_version = "bookworm"
platform = "linux/arm64"
tag = "my-tag"
user = "admin"
shell = "zsh"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.image.base, BaseOs::Debian);
        assert_eq!(config.image.base_version.as_deref(), Some("bookworm"));
        assert_eq!(config.image.platform, "linux/arm64");
        assert_eq!(config.image.tag.as_deref(), Some("my-tag"));
        assert_eq!(config.image.user, "admin");
        assert_eq!(config.image.shell, ShellType::Zsh);
    }

    #[test]
    fn parse_base_os_alpine() {
        let toml_str = r#"
[project]
name = "alp"
[agent]
type = "claude"
[image]
base = "alpine"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.image.base, BaseOs::Alpine);
    }

    #[test]
    fn invalid_base_os_fails() {
        let toml_str = r#"
[project]
name = "bad"
[agent]
type = "claude"
[image]
base = "centos"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_shell_type_fails() {
        let toml_str = r#"
[project]
name = "bad"
[agent]
type = "claude"
[image]
shell = "fish"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn base_os_display() {
        assert_eq!(BaseOs::Ubuntu.to_string(), "ubuntu");
        assert_eq!(BaseOs::Debian.to_string(), "debian");
        assert_eq!(BaseOs::Alpine.to_string(), "alpine");
    }

    #[test]
    fn shell_type_display() {
        assert_eq!(ShellType::Bash.to_string(), "bash");
        assert_eq!(ShellType::Zsh.to_string(), "zsh");
        assert_eq!(ShellType::Sh.to_string(), "sh");
    }

    #[test]
    fn image_config_default_impl() {
        let img = ImageConfig::default();
        assert_eq!(img.base, BaseOs::Ubuntu);
        assert!(img.base_version.is_none());
        assert_eq!(img.platform, "linux/amd64");
        assert!(img.tag.is_none());
        assert_eq!(img.user, "dev");
        assert_eq!(img.shell, ShellType::Bash);
    }

    // ───────────────────── default_version_for_os ─────────────────────

    #[test]
    fn default_version_for_all_os() {
        assert_eq!(default_version_for_os(BaseOs::Ubuntu), "24.04");
        assert_eq!(default_version_for_os(BaseOs::Debian), "bookworm");
        assert_eq!(default_version_for_os(BaseOs::Alpine), "3.21");
    }

    // ───────────────────── Auth config ─────────────────────

    #[test]
    fn parse_auth_claude_api_key() {
        let toml_str = r#"
[project]
name = "auth"
[agent]
type = "claude"
[auth.claude]
method = "api-key"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let claude = config.auth.claude.unwrap();
        assert_eq!(claude.method, ClaudeAuthMethod::ApiKey);
    }

    #[test]
    fn parse_auth_claude_all_methods() {
        for (method_str, expected) in [
            ("api-key", ClaudeAuthMethod::ApiKey),
            ("oauth", ClaudeAuthMethod::Oauth),
            ("bedrock", ClaudeAuthMethod::Bedrock),
            ("bedrock-api-key", ClaudeAuthMethod::BedrockApiKey),
            ("vertex", ClaudeAuthMethod::Vertex),
            ("proxy", ClaudeAuthMethod::Proxy),
        ] {
            let toml_str = format!(
                r#"
[project]
name = "auth-test"
[agent]
type = "claude"
[auth.claude]
method = "{method_str}"
"#
            );
            let config: ProjectConfig = toml::from_str(&toml_str).unwrap();
            assert_eq!(config.auth.claude.unwrap().method, expected);
        }
    }

    #[test]
    fn parse_auth_codex_with_extra_fields() {
        let toml_str = r#"
[project]
name = "codex-auth"
[agent]
type = "codex"
[auth.codex]
method = "azure"
azure_endpoint = "https://my-resource.openai.azure.com"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let codex = config.auth.codex.unwrap();
        assert_eq!(codex.method, CodexAuthMethod::Azure);
        assert_eq!(
            codex.azure_endpoint.as_deref(),
            Some("https://my-resource.openai.azure.com")
        );
    }

    #[test]
    fn parse_auth_codex_custom_method() {
        let toml_str = r#"
[project]
name = "codex-custom"
[agent]
type = "codex"
[auth.codex]
method = "custom"
custom_env_key = "MY_API_KEY"
custom_base_url = "https://proxy.example.com"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let codex = config.auth.codex.unwrap();
        assert_eq!(codex.method, CodexAuthMethod::Custom);
        assert_eq!(codex.custom_env_key.as_deref(), Some("MY_API_KEY"));
        assert_eq!(
            codex.custom_base_url.as_deref(),
            Some("https://proxy.example.com")
        );
    }

    #[test]
    fn invalid_auth_method_fails() {
        let toml_str = r#"
[project]
name = "bad"
[agent]
type = "claude"
[auth.claude]
method = "password"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn auth_config_default_is_none() {
        let config: ProjectConfig = toml::from_str(MINIMAL_CONFIG).unwrap();
        assert!(config.auth.claude.is_none());
        assert!(config.auth.codex.is_none());
    }

    #[test]
    fn claude_auth_method_display() {
        assert_eq!(ClaudeAuthMethod::ApiKey.to_string(), "api-key");
        assert_eq!(ClaudeAuthMethod::Oauth.to_string(), "oauth");
        assert_eq!(ClaudeAuthMethod::Bedrock.to_string(), "bedrock");
        assert_eq!(
            ClaudeAuthMethod::BedrockApiKey.to_string(),
            "bedrock-api-key"
        );
        assert_eq!(ClaudeAuthMethod::Vertex.to_string(), "vertex");
        assert_eq!(ClaudeAuthMethod::Proxy.to_string(), "proxy");
    }

    #[test]
    fn codex_auth_method_display() {
        assert_eq!(CodexAuthMethod::ApiKey.to_string(), "api-key");
        assert_eq!(CodexAuthMethod::Oauth.to_string(), "oauth");
        assert_eq!(CodexAuthMethod::Azure.to_string(), "azure");
        assert_eq!(CodexAuthMethod::Custom.to_string(), "custom");
    }

    // ───────────────────── Firewall config ─────────────────────

    #[test]
    fn parse_firewall_config() {
        let toml_str = r#"
[project]
name = "fw"
[agent]
type = "claude"
[firewall]
enabled = true
allowed_domains = ["github.com", "api.anthropic.com"]
allowed_cidrs = ["10.0.0.0/8", "172.16.0.0/12"]
allow_ssh = false
allow_dns = false
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert!(config.firewall.enabled);
        assert_eq!(
            config.firewall.allowed_domains,
            vec!["github.com", "api.anthropic.com"]
        );
        assert_eq!(
            config.firewall.allowed_cidrs,
            vec!["10.0.0.0/8", "172.16.0.0/12"]
        );
        assert!(!config.firewall.allow_ssh);
        assert!(!config.firewall.allow_dns);
    }

    #[test]
    fn firewall_default_impl() {
        let fw = FirewallConfig::default();
        assert!(!fw.enabled);
        assert!(fw.allowed_domains.is_empty());
        assert!(fw.allowed_cidrs.is_empty());
        assert!(fw.allow_ssh);
        assert!(fw.allow_dns);
    }

    // ───────────────────── Workspace config ─────────────────────

    #[test]
    fn parse_workspace_with_mounts() {
        let toml_str = r#"
[project]
name = "ws"
[agent]
type = "claude"
[workspace]
mount_path = "/code"
[[workspace.additional_mounts]]
source = "/host/data"
target = "/container/data"
read_only = true
[[workspace.additional_mounts]]
source = "/host/cache"
target = "/container/cache"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.workspace.mount_path, "/code");
        assert_eq!(config.workspace.additional_mounts.len(), 2);
        assert_eq!(config.workspace.additional_mounts[0].source, "/host/data");
        assert_eq!(
            config.workspace.additional_mounts[0].target,
            "/container/data"
        );
        assert!(config.workspace.additional_mounts[0].read_only);
        assert!(!config.workspace.additional_mounts[1].read_only);
    }

    #[test]
    fn workspace_default_impl() {
        let ws = WorkspaceConfig::default();
        assert_eq!(ws.mount_path, "/workspace");
        assert!(ws.additional_mounts.is_empty());
    }

    // ───────────────────── Volumes ─────────────────────

    #[test]
    fn parse_volumes() {
        let toml_str = r#"
[project]
name = "vol"
[agent]
type = "claude"
[volumes]
data = { target = "/data" }
logs = { target = "/var/log/app" }
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.volumes.len(), 2);
        assert_eq!(config.volumes["data"].target, "/data");
        assert_eq!(config.volumes["logs"].target, "/var/log/app");
    }

    // ───────────────────── Environment config ─────────────────────

    #[test]
    fn parse_environment_vars() {
        let toml_str = r#"
[project]
name = "env"
[agent]
type = "claude"
[environment]
MY_VAR = "hello"
ANOTHER = "world"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.environment.vars["MY_VAR"], "hello");
        assert_eq!(config.environment.vars["ANOTHER"], "world");
    }

    #[test]
    fn parse_environment_with_env_files() {
        let toml_str = r#"
[project]
name = "env-files"
[agent]
type = "claude"
[environment]
MY_VAR = "test"
[environment.env_files]
files = [".env", ".env.local"]
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let env_files = config.environment.env_files.unwrap();
        assert_eq!(env_files.files, vec![".env", ".env.local"]);
        assert_eq!(config.environment.vars["MY_VAR"], "test");
    }

    // ───────────────────── Services ─────────────────────

    #[test]
    fn parse_services() {
        let toml_str = r#"
[project]
name = "svc"
[agent]
type = "claude"
[services.postgres]
enabled = true
version = "16"
port = 5433
[services.redis]
enabled = false
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let pg = &config.services["postgres"];
        assert!(pg.enabled);
        assert_eq!(pg.version.as_deref(), Some("16"));
        assert_eq!(pg.port, Some(5433));
        let redis = &config.services["redis"];
        assert!(!redis.enabled);
    }

    #[test]
    fn service_enabled_defaults_to_true() {
        let toml_str = r#"
[project]
name = "svc-def"
[agent]
type = "claude"
[services.postgres]
version = "15"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert!(config.services["postgres"].enabled);
    }

    // ───────────────────── MCP config ─────────────────────

    #[test]
    fn parse_mcp_servers() {
        let toml_str = r#"
[project]
name = "mcp"
[agent]
type = "claude"
[mcp.my-server]
image = "ghcr.io/example/mcp-server:latest"
command = ["serve", "--port", "8080"]
env = ["API_KEY"]
volumes = ["/data:/data"]
port = 8080
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let mcp = &config.mcp["my-server"];
        assert_eq!(mcp.image, "ghcr.io/example/mcp-server:latest");
        assert_eq!(mcp.command.as_ref().unwrap(), &["serve", "--port", "8080"]);
        assert_eq!(mcp.env, vec!["API_KEY"]);
        assert_eq!(mcp.volumes, vec!["/data:/data"]);
        assert_eq!(mcp.port, Some(8080));
    }

    #[test]
    fn mcp_server_minimal() {
        let toml_str = r#"
[project]
name = "mcp-min"
[agent]
type = "claude"
[mcp.simple]
image = "my-mcp:latest"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let mcp = &config.mcp["simple"];
        assert_eq!(mcp.image, "my-mcp:latest");
        assert!(mcp.command.is_none());
        assert!(mcp.env.is_empty());
        assert!(mcp.volumes.is_empty());
        assert!(mcp.port.is_none());
    }

    // ───────────────────── Runtime config ─────────────────────

    #[test]
    fn parse_runtime_config() {
        let toml_str = r#"
[project]
name = "rt"
[agent]
type = "claude"
[runtime]
cap_add = ["NET_ADMIN", "SYS_PTRACE"]
cap_drop = ["ALL"]
security_opt = ["no-new-privileges"]
memory_limit = "4g"
cpu_limit = "2"
shm_size = "1g"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.runtime.cap_add, vec!["NET_ADMIN", "SYS_PTRACE"]);
        assert_eq!(config.runtime.cap_drop, vec!["ALL"]);
        assert_eq!(config.runtime.security_opt, vec!["no-new-privileges"]);
        assert_eq!(config.runtime.memory_limit.as_deref(), Some("4g"));
        assert_eq!(config.runtime.cpu_limit.as_deref(), Some("2"));
        assert_eq!(config.runtime.shm_size.as_deref(), Some("1g"));
    }

    // ───────────────────── Modules ─────────────────────

    #[test]
    fn parse_modules_with_params() {
        let toml_str = r#"
[project]
name = "mods"
[agent]
type = "claude"
[modules]
nodejs = { version = "20" }
python = { version = "3.12", venv = true }
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.modules.len(), 2);
        let node = config.modules["nodejs"].as_table().unwrap();
        assert_eq!(node["version"].as_str(), Some("20"));
        let py = config.modules["python"].as_table().unwrap();
        assert_eq!(py["version"].as_str(), Some("3.12"));
        assert_eq!(py["venv"].as_bool(), Some(true));
    }

    #[test]
    fn parse_modules_empty_table() {
        let toml_str = r#"
[project]
name = "mods-empty"
[agent]
type = "claude"
[modules]
rust = {}
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.modules.len(), 1);
        assert!(config.modules["rust"].as_table().unwrap().is_empty());
    }

    // ───────────────────── Full config ─────────────────────

    #[test]
    fn parse_full_config() {
        let toml_str = r#"
[project]
name = "full-project"
description = "A comprehensive test"

[agent]
type = "both"
claude_version = "1.0.0"
codex_version = "2.0.0"

[image]
base = "debian"
base_version = "bookworm"
platform = "linux/arm64"
tag = "full-test"
user = "coder"
shell = "zsh"

[modules]
nodejs = { version = "20" }
python = { version = "3.12" }

[auth.claude]
method = "api-key"
[auth.codex]
method = "oauth"

[firewall]
enabled = true
allowed_domains = ["github.com"]
allowed_cidrs = ["10.0.0.0/8"]

[workspace]
mount_path = "/src"

[volumes]
data = { target = "/data" }

[environment]
ENV = "production"

[services.postgres]
enabled = true
version = "16"
port = 5432

[runtime]
cap_add = ["NET_ADMIN"]
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "full-project");
        assert_eq!(
            config.project.description.as_deref(),
            Some("A comprehensive test")
        );
        assert_eq!(config.agent.agent_type, AgentType::Both);
        assert_eq!(config.agent.claude_version, "1.0.0");
        assert_eq!(config.agent.codex_version, "2.0.0");
        assert_eq!(config.image.base, BaseOs::Debian);
        assert_eq!(config.image.user, "coder");
        assert_eq!(config.image.shell, ShellType::Zsh);
        assert_eq!(config.modules.len(), 2);
        assert!(config.auth.claude.is_some());
        assert!(config.auth.codex.is_some());
        assert!(config.firewall.enabled);
        assert_eq!(config.workspace.mount_path, "/src");
        assert_eq!(config.volumes.len(), 1);
        assert_eq!(config.environment.vars["ENV"], "production");
        assert!(config.services["postgres"].enabled);
        assert_eq!(config.runtime.cap_add, vec!["NET_ADMIN"]);
    }

    // ───────────────────── Error cases ─────────────────────

    #[test]
    fn missing_project_section_fails() {
        let toml_str = r#"
[agent]
type = "claude"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn missing_agent_section_fails() {
        let toml_str = r#"
[project]
name = "no-agent"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn missing_project_name_fails() {
        let toml_str = r#"
[project]
description = "no name"
[agent]
type = "claude"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn missing_agent_type_fails() {
        let toml_str = r#"
[project]
name = "no-type"
[agent]
claude_version = "latest"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_top_level_section_fails() {
        // deny_unknown_fields on ProjectConfig should reject unknown top-level tables
        let toml_str = r#"
[project]
name = "bad"
[agent]
type = "claude"
[bogus_section]
key = "value"
"#;
        let result: Result<ProjectConfig, _> = toml::from_str(toml_str);
        assert!(
            result.is_err(),
            "deny_unknown_fields should reject unknown top-level sections"
        );
    }

    #[test]
    fn empty_config_fails() {
        let result: Result<ProjectConfig, _> = toml::from_str("");
        assert!(result.is_err());
    }

    // ───────────────────── Helm config ─────────────────────

    #[test]
    fn helm_config_defaults() {
        let config: ProjectConfig = toml::from_str(MINIMAL_CONFIG).unwrap();
        assert!(config.helm.image_registry.is_none());
        assert!(config.helm.image_repository.is_none());
        assert_eq!(config.helm.image_tag, "latest");
        assert!(config.helm.namespace.is_none());
        assert!(config.helm.storage_class.is_none());
        assert_eq!(config.helm.default_pvc_size, "10Gi");
        assert!(config.helm.ingress_host.is_none());
        assert!(config.helm.ingress_class.is_none());
    }

    #[test]
    fn parse_helm_config_all_fields() {
        let toml_str = r#"
[project]
name = "helm-test"
[agent]
type = "claude"
[helm]
image_registry = "ghcr.io/myorg"
image_repository = "my-agent"
image_tag = "sha-abc123"
namespace = "dev-agents"
storage_class = "ssd"
default_pvc_size = "50Gi"
ingress_host = "agent.dev.internal"
ingress_class = "nginx"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.helm.image_registry.as_deref(), Some("ghcr.io/myorg"));
        assert_eq!(config.helm.image_repository.as_deref(), Some("my-agent"));
        assert_eq!(config.helm.image_tag, "sha-abc123");
        assert_eq!(config.helm.namespace.as_deref(), Some("dev-agents"));
        assert_eq!(config.helm.storage_class.as_deref(), Some("ssd"));
        assert_eq!(config.helm.default_pvc_size, "50Gi");
        assert_eq!(
            config.helm.ingress_host.as_deref(),
            Some("agent.dev.internal")
        );
        assert_eq!(config.helm.ingress_class.as_deref(), Some("nginx"));
    }

    #[test]
    fn parse_helm_config_partial() {
        let toml_str = r#"
[project]
name = "helm-partial"
[agent]
type = "claude"
[helm]
image_registry = "docker.io/myteam"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.helm.image_registry.as_deref(),
            Some("docker.io/myteam")
        );
        assert_eq!(config.helm.image_tag, "latest");
        assert_eq!(config.helm.default_pvc_size, "10Gi");
        assert!(config.helm.namespace.is_none());
    }

    #[test]
    fn helm_config_default_impl() {
        let helm = HelmConfig::default();
        assert!(helm.image_registry.is_none());
        assert!(helm.image_repository.is_none());
        assert_eq!(helm.image_tag, "latest");
        assert!(helm.namespace.is_none());
        assert!(helm.storage_class.is_none());
        assert_eq!(helm.default_pvc_size, "10Gi");
        assert!(helm.ingress_host.is_none());
        assert!(helm.ingress_class.is_none());
    }
}
