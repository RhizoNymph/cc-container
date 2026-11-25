use dialoguer::{Confirm, FuzzySelect, Input, MultiSelect};

pub fn select_agent_type() -> dialoguer::Result<usize> {
    FuzzySelect::new()
        .with_prompt("Which AI agent(s)?")
        .items(&["Claude Code", "OpenAI Codex", "Both"])
        .default(0)
        .interact()
}

pub fn select_base_os() -> dialoguer::Result<usize> {
    FuzzySelect::new()
        .with_prompt("Base OS")
        .items(&["Ubuntu 24.04", "Debian Bookworm", "Alpine 3.21"])
        .default(0)
        .interact()
}

pub fn select_shell() -> dialoguer::Result<usize> {
    FuzzySelect::new()
        .with_prompt("Default shell")
        .items(&["bash", "zsh", "sh"])
        .default(0)
        .interact()
}

pub fn select_claude_auth() -> dialoguer::Result<usize> {
    FuzzySelect::new()
        .with_prompt("Claude Code auth method")
        .items(&[
            "API Key (ANTHROPIC_API_KEY)",
            "OAuth (mount host credentials)",
            "AWS Bedrock (standard creds)",
            "AWS Bedrock API Key",
            "Google Vertex AI",
            "Proxy / Gateway",
        ])
        .default(0)
        .interact()
}

pub fn select_codex_auth() -> dialoguer::Result<usize> {
    FuzzySelect::new()
        .with_prompt("Codex CLI auth method")
        .items(&[
            "API Key (OPENAI_API_KEY)",
            "OAuth (mount host credentials)",
            "Azure OpenAI",
            "Custom provider",
        ])
        .default(0)
        .interact()
}

pub fn select_languages() -> dialoguer::Result<Vec<usize>> {
    MultiSelect::new()
        .with_prompt("Language toolchains to include (space to select)")
        .items(&[
            "Node.js (required for agents)",
            "Python",
            "Rust",
            "Go",
            "Java",
            "Ruby",
            ".NET",
            "Zig",
            "C/C++",
        ])
        .defaults(&[true, false, false, false, false, false, false, false, false])
        .interact()
}

pub fn select_tools() -> dialoguer::Result<Vec<usize>> {
    MultiSelect::new()
        .with_prompt("Additional tools (space to select)")
        .items(&["Git", "Docker CLI", "Build tools (gcc, make)"])
        .defaults(&[true, false, false])
        .interact()
}

pub fn select_services() -> dialoguer::Result<Vec<usize>> {
    MultiSelect::new()
        .with_prompt("Infrastructure services (space to select)")
        .items(&[
            "PostgreSQL",
            "MySQL",
            "MongoDB",
            "Redis",
            "RabbitMQ",
            "Kafka (Redpanda)",
            "Elasticsearch",
            "Meilisearch",
            "MinIO (S3)",
            "Prometheus",
            "Grafana",
        ])
        .interact()
}

pub fn confirm_firewall() -> dialoguer::Result<bool> {
    Confirm::new()
        .with_prompt("Enable network firewall (domain whitelisting)?")
        .default(true)
        .interact()
}

pub fn input_project_name(default: &str) -> dialoguer::Result<String> {
    Input::new()
        .with_prompt("Project name")
        .default(default.to_string())
        .interact_text()
}
