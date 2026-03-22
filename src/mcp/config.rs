use crate::config::project::ProjectConfig;
use indexmap::IndexMap;
use serde::Serialize;

/// Generate .mcp.json content for Claude Code.
pub fn generate_mcp_json(config: &ProjectConfig) -> String {
    let mut mcp_servers: IndexMap<String, McpServerEntry> = IndexMap::new();

    for (name, mcp_config) in &config.mcp {
        let args = if let Some(ref cmd) = mcp_config.command {
            // Construct: docker run -i --rm [env flags] [volume flags] <image> [command...]
            let mut docker_args = vec![
                "run".to_string(),
                "-i".to_string(),
                "--rm".to_string(),
                "--network".to_string(),
                "host".to_string(),
            ];

            for env_var in &mcp_config.env {
                docker_args.push("-e".to_string());
                docker_args.push(env_var.clone());
            }

            for vol in &mcp_config.volumes {
                docker_args.push("-v".to_string());
                docker_args.push(vol.clone());
            }

            docker_args.push(mcp_config.image.clone());
            docker_args.extend(cmd.iter().cloned());
            docker_args
        } else {
            vec![
                "run".to_string(),
                "-i".to_string(),
                "--rm".to_string(),
                mcp_config.image.clone(),
            ]
        };

        mcp_servers.insert(
            name.clone(),
            McpServerEntry {
                command: "docker".to_string(),
                args,
            },
        );
    }

    let mcp_json = McpConfig {
        mcp_servers,
    };

    serde_json::to_string_pretty(&mcp_json).unwrap_or_else(|_| "{}".to_string())
}

#[derive(Serialize)]
struct McpConfig {
    #[serde(rename = "mcpServers")]
    mcp_servers: IndexMap<String, McpServerEntry>,
}

#[derive(Serialize)]
struct McpServerEntry {
    command: String,
    args: Vec<String>,
}
