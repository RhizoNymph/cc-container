use crate::config::project::ProjectConfig;
use indexmap::IndexMap;
use serde::Serialize;

/// Generate .mcp.json content for Claude Code.
pub fn generate_mcp_json(config: &ProjectConfig) -> crate::error::Result<String> {
    let mut mcp_servers: IndexMap<String, McpServerEntry> = IndexMap::new();

    for (name, mcp_config) in &config.mcp {
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

        if let Some(ref cmd) = mcp_config.command {
            docker_args.extend(cmd.iter().cloned());
        }

        let args = docker_args;

        mcp_servers.insert(
            name.clone(),
            McpServerEntry {
                command: "docker".to_string(),
                args,
            },
        );
    }

    let mcp_json = McpConfig { mcp_servers };

    Ok(serde_json::to_string_pretty(&mcp_json)?)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal ProjectConfig from a TOML string.
    fn parse_config(toml_str: &str) -> ProjectConfig {
        toml::from_str(toml_str).expect("valid TOML for test config")
    }

    /// Minimal TOML that satisfies the required ProjectConfig fields.
    const BASE_TOML: &str = r#"
[project]
name = "test"

[agent]
type = "claude"
"#;

    /// Helper: parse the JSON output into a serde_json::Value.
    fn parse_mcp_json(config: &ProjectConfig) -> serde_json::Value {
        let json_str = generate_mcp_json(config).unwrap();
        serde_json::from_str(&json_str).unwrap()
    }

    // ── McpServerConfig deserialization ─────────────────────────────

    #[test]
    fn deserialize_mcp_server_minimal() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        assert_eq!(config.mcp.len(), 1);

        let fetch = config.mcp.get("fetch").unwrap();
        assert_eq!(fetch.image, "mcp/fetch:latest");
        assert!(fetch.command.is_none());
        assert!(fetch.env.is_empty());
        assert!(fetch.volumes.is_empty());
        assert!(fetch.port.is_none());
    }

    #[test]
    fn deserialize_mcp_server_all_fields() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
command = ["node", "server.js"]
env = ["API_KEY=${MCP_API_KEY}", "DEBUG=true"]
volumes = ["/data:/data", "/config:/config:ro"]
port = 8000
"#;
        let config = parse_config(toml_str);
        let fetch = config.mcp.get("fetch").unwrap();

        assert_eq!(fetch.image, "mcp/fetch:latest");
        assert_eq!(
            fetch.command.as_ref().unwrap(),
            &vec!["node".to_string(), "server.js".to_string()]
        );
        assert_eq!(fetch.env.len(), 2);
        assert_eq!(fetch.env[0], "API_KEY=${MCP_API_KEY}");
        assert_eq!(fetch.env[1], "DEBUG=true");
        assert_eq!(fetch.volumes.len(), 2);
        assert_eq!(fetch.volumes[0], "/data:/data");
        assert_eq!(fetch.volumes[1], "/config:/config:ro");
        assert_eq!(fetch.port, Some(8000));
    }

    #[test]
    fn deserialize_multiple_mcp_servers() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"

[mcp.memory]
image = "mcp/memory:latest"
command = ["python", "-m", "mcp_memory"]

[mcp.search]
image = "mcp/search:v2"
port = 9000
"#;
        let config = parse_config(toml_str);
        assert_eq!(config.mcp.len(), 3);
        assert!(config.mcp.contains_key("fetch"));
        assert!(config.mcp.contains_key("memory"));
        assert!(config.mcp.contains_key("search"));
    }

    #[test]
    fn deserialize_empty_mcp_section() {
        let config = parse_config(BASE_TOML);
        assert!(config.mcp.is_empty());
    }

    #[test]
    fn deserialize_mcp_env_defaults_to_empty_vec() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.svc]
image = "img:latest"
"#;
        let config = parse_config(toml_str);
        let svc = config.mcp.get("svc").unwrap();
        assert!(svc.env.is_empty());
    }

    #[test]
    fn deserialize_mcp_volumes_defaults_to_empty_vec() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.svc]
image = "img:latest"
"#;
        let config = parse_config(toml_str);
        let svc = config.mcp.get("svc").unwrap();
        assert!(svc.volumes.is_empty());
    }

    // ── generate_mcp_json: empty config ─────────────────────────────

    #[test]
    fn generate_json_empty_mcp_returns_empty_servers() {
        let config = parse_config(BASE_TOML);
        let json = parse_mcp_json(&config);

        let servers = json.get("mcpServers").unwrap().as_object().unwrap();
        assert!(servers.is_empty());
    }

    // ── generate_mcp_json: single server, minimal config ────────────

    #[test]
    fn generate_json_minimal_server_has_docker_command() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let server = &json["mcpServers"]["fetch"];
        assert_eq!(server["command"].as_str().unwrap(), "docker");
    }

    #[test]
    fn generate_json_minimal_server_base_args() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["fetch"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        // Should be: run -i --rm --network host <image>
        assert_eq!(
            args,
            vec!["run", "-i", "--rm", "--network", "host", "mcp/fetch:latest"]
        );
    }

    // ── generate_mcp_json: server with env vars ─────────────────────

    #[test]
    fn generate_json_server_with_env_vars() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
env = ["API_KEY=${MCP_API_KEY}", "DEBUG=1"]
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["fetch"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        // Env vars come as -e KEY pairs before the image
        assert!(args.contains(&"-e"));
        assert!(args.contains(&"API_KEY=${MCP_API_KEY}"));
        assert!(args.contains(&"DEBUG=1"));

        // Env flags should appear before the image name
        let image_pos = args.iter().position(|&a| a == "mcp/fetch:latest").unwrap();
        let first_env_pos = args.iter().position(|&a| a == "-e").unwrap();
        assert!(first_env_pos < image_pos);
    }

    #[test]
    fn generate_json_server_with_volumes() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
volumes = ["/data:/data", "/config:/config:ro"]
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["fetch"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        assert!(args.contains(&"-v"));
        assert!(args.contains(&"/data:/data"));
        assert!(args.contains(&"/config:/config:ro"));

        // Volume flags should appear before the image name
        let image_pos = args.iter().position(|&a| a == "mcp/fetch:latest").unwrap();
        let first_vol_pos = args.iter().position(|&a| a == "-v").unwrap();
        assert!(first_vol_pos < image_pos);
    }

    #[test]
    fn generate_json_server_with_command() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
command = ["node", "server.js"]
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["fetch"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        // Command args should appear after the image
        let image_pos = args.iter().position(|&a| a == "mcp/fetch:latest").unwrap();
        assert_eq!(args[image_pos + 1], "node");
        assert_eq!(args[image_pos + 2], "server.js");
    }

    #[test]
    fn generate_json_server_with_all_fields() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
command = ["node", "server.js"]
env = ["API_KEY=${MCP_API_KEY}"]
volumes = ["/data:/data"]
port = 8000
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["fetch"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        // Expected order: run -i --rm --network host -e API_KEY=... -v /data:/data mcp/fetch:latest node server.js
        assert_eq!(args[0], "run");
        assert_eq!(args[1], "-i");
        assert_eq!(args[2], "--rm");
        assert_eq!(args[3], "--network");
        assert_eq!(args[4], "host");

        // Env var flags
        let env_idx = args.iter().position(|&a| a == "-e").unwrap();
        assert_eq!(args[env_idx + 1], "API_KEY=${MCP_API_KEY}");

        // Volume flags
        let vol_idx = args.iter().position(|&a| a == "-v").unwrap();
        assert_eq!(args[vol_idx + 1], "/data:/data");

        // Image then command
        let image_pos = args.iter().position(|&a| a == "mcp/fetch:latest").unwrap();
        assert_eq!(args[image_pos + 1], "node");
        assert_eq!(args[image_pos + 2], "server.js");
    }

    // ── generate_mcp_json: port is not in docker args ───────────────

    #[test]
    fn generate_json_port_not_in_docker_args() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
port = 8000
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["fetch"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        // Port is used for compose, not for mcp.json docker args
        assert!(!args.contains(&"8000"));
        assert!(!args.contains(&"-p"));
    }

    // ── generate_mcp_json: multiple servers ─────────────────────────

    #[test]
    fn generate_json_multiple_servers() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"

[mcp.memory]
image = "mcp/memory:latest"
command = ["python", "server.py"]
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let servers = json["mcpServers"].as_object().unwrap();
        assert_eq!(servers.len(), 2);
        assert!(servers.contains_key("fetch"));
        assert!(servers.contains_key("memory"));
    }

    #[test]
    fn generate_json_multiple_servers_each_has_own_image() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"

[mcp.memory]
image = "mcp/memory:v2"
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let fetch_args: Vec<&str> = json["mcpServers"]["fetch"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let memory_args: Vec<&str> = json["mcpServers"]["memory"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        assert!(fetch_args.contains(&"mcp/fetch:latest"));
        assert!(memory_args.contains(&"mcp/memory:v2"));
    }

    // ── generate_mcp_json: output is valid JSON ─────────────────────

    #[test]
    fn generate_json_output_is_valid_json() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
command = ["node", "server.js"]
env = ["API_KEY=${MCP_API_KEY}"]
volumes = ["/data:/data"]
port = 8000
"#;
        let config = parse_config(toml_str);
        let json_str = generate_mcp_json(&config).unwrap();

        // Should parse without error
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("mcpServers").is_some());
    }

    #[test]
    fn generate_json_uses_mcp_servers_camel_case_key() {
        let config = parse_config(BASE_TOML);
        let json_str = generate_mcp_json(&config).unwrap();

        // The JSON key should be "mcpServers" (camelCase), not "mcp_servers"
        assert!(json_str.contains("mcpServers"));
        assert!(!json_str.contains("mcp_servers"));
    }

    // ── generate_mcp_json: argument ordering ────────────────────────

    #[test]
    fn generate_json_args_order_env_then_volumes_then_image_then_command() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.svc]
image = "img:latest"
command = ["start-server"]
env = ["KEY=val"]
volumes = ["/a:/b"]
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["svc"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        let env_flag_pos = args.iter().position(|&a| a == "-e").unwrap();
        let vol_flag_pos = args.iter().position(|&a| a == "-v").unwrap();
        let image_pos = args.iter().position(|&a| a == "img:latest").unwrap();
        let cmd_pos = args.iter().position(|&a| a == "start-server").unwrap();

        // env flags come before volume flags
        assert!(env_flag_pos < vol_flag_pos);
        // volume flags come before image
        assert!(vol_flag_pos < image_pos);
        // command comes after image
        assert!(image_pos < cmd_pos);
    }

    // ── generate_mcp_json: server without command ───────────────────

    #[test]
    fn generate_json_no_command_means_image_is_last_arg() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["fetch"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        assert_eq!(*args.last().unwrap(), "mcp/fetch:latest");
    }

    // ── generate_mcp_json: multiple env vars each get a flag ────────

    #[test]
    fn generate_json_multiple_env_vars_each_gets_flag() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.svc]
image = "img:latest"
env = ["A=1", "B=2", "C=3"]
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["svc"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        // Each env var should be preceded by -e
        let env_count = args.iter().filter(|&&a| a == "-e").count();
        assert_eq!(env_count, 3);

        // Verify each pair
        for (i, &arg) in args.iter().enumerate() {
            if arg == "-e" {
                let val = args[i + 1];
                assert!(
                    val == "A=1" || val == "B=2" || val == "C=3",
                    "unexpected env value: {val}"
                );
            }
        }
    }

    // ── generate_mcp_json: multiple volumes each get a flag ─────────

    #[test]
    fn generate_json_multiple_volumes_each_gets_flag() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.svc]
image = "img:latest"
volumes = ["/a:/a", "/b:/b"]
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["svc"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        let vol_count = args.iter().filter(|&&a| a == "-v").count();
        assert_eq!(vol_count, 2);
    }

    // ── generate_mcp_json: all servers use host network ─────────────

    #[test]
    fn generate_json_all_servers_use_host_network() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.a]
image = "img-a:latest"

[mcp.b]
image = "img-b:latest"
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        for name in ["a", "b"] {
            let args: Vec<&str> = json["mcpServers"][name]["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();

            assert!(args.contains(&"--network"));
            let net_pos = args.iter().position(|&a| a == "--network").unwrap();
            assert_eq!(args[net_pos + 1], "host");
        }
    }

    // ── generate_mcp_json: all servers use --rm and -i ──────────────

    #[test]
    fn generate_json_all_servers_use_rm_and_interactive() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.svc]
image = "img:latest"
"#;
        let config = parse_config(toml_str);
        let json = parse_mcp_json(&config);

        let args: Vec<&str> = json["mcpServers"]["svc"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        assert!(args.contains(&"--rm"));
        assert!(args.contains(&"-i"));
    }
}
