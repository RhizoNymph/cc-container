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
    #[serde(default = "default_base_version")]
    pub base_version: String,
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
            base_version: default_base_version(),
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

fn default_base_version() -> String {
    "24.04".to_string()
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
