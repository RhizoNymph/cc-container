use crate::auth;
use crate::config::project::{AgentType, ProjectConfig};

/// Generate the contents of a .env.example file.
pub fn generate_env_example(config: &ProjectConfig) -> String {
    let mut lines = Vec::new();

    lines.push("# cc-container environment variables".to_string());
    lines.push("# Copy this file to .env and fill in your values.".to_string());
    lines.push(String::new());

    // Auth section
    let container_user = &config.image.user;

    match config.agent.agent_type {
        AgentType::Claude | AgentType::Both => {
            if let Some(ref claude_auth) = config.auth.claude {
                let reqs = auth::claude::requirements(claude_auth, container_user);
                for line in &reqs.env_example_lines {
                    lines.push(line.clone());
                }
                lines.push(String::new());
            }
        }
        _ => {}
    }

    match config.agent.agent_type {
        AgentType::Codex | AgentType::Both => {
            if let Some(ref codex_auth) = config.auth.codex {
                let reqs = auth::codex::requirements(codex_auth, container_user);
                for line in &reqs.env_example_lines {
                    lines.push(line.clone());
                }
                lines.push(String::new());
            }
        }
        _ => {}
    }

    // Service passwords and connection details
    let has_services = config.services.values().any(|s| s.enabled);
    if has_services {
        lines.push("# Service credentials".to_string());

        for (name, svc_config) in &config.services {
            if !svc_config.enabled {
                continue;
            }

            match name.as_str() {
                "postgres" => {
                    let pw_env = svc_config
                        .extra
                        .get("password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("POSTGRES_PASSWORD");
                    lines.push(format!("{pw_env}=changeme"));
                }
                "mysql" => {
                    let pw_env = svc_config
                        .extra
                        .get("password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MYSQL_PASSWORD");
                    let root_pw_env = svc_config
                        .extra
                        .get("root_password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MYSQL_ROOT_PASSWORD");
                    lines.push(format!("{pw_env}=changeme"));
                    lines.push(format!("{root_pw_env}=changeme"));
                }
                "mariadb" => {
                    let pw_env = svc_config
                        .extra
                        .get("password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MARIADB_PASSWORD");
                    let root_pw_env = svc_config
                        .extra
                        .get("root_password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MARIADB_ROOT_PASSWORD");
                    lines.push(format!("{pw_env}=changeme"));
                    lines.push(format!("{root_pw_env}=changeme"));
                }
                "minio" => {
                    lines.push("MINIO_ACCESS_KEY=minioadmin".to_string());
                    lines.push("MINIO_SECRET_KEY=minioadmin".to_string());
                }
                _ => {}
            }
        }
        lines.push(String::new());
    }

    // MCP env vars
    if !config.mcp.is_empty() {
        lines.push("# MCP server credentials".to_string());
        for (_name, mcp_config) in &config.mcp {
            for env_var in &mcp_config.env {
                lines.push(format!("{env_var}=your-value-here"));
            }
        }
        lines.push(String::new());
    }

    lines.join("\n")
}
