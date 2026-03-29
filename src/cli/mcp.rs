use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum McpCommand {
    /// List configured MCP servers
    List,
    /// Add an MCP server
    Add(McpAddArgs),
    /// Remove an MCP server
    Remove(McpRemoveArgs),
}

#[derive(Parser)]
pub struct McpAddArgs {
    /// MCP server name
    pub name: String,

    /// Docker image
    #[arg(long)]
    pub image: String,

    /// Command to run
    #[arg(long)]
    pub command: Option<String>,

    /// Environment variables (KEY=VAL)
    #[arg(long = "env")]
    pub envs: Vec<String>,

    /// Volume mounts
    #[arg(long = "volume")]
    pub volumes: Vec<String>,
}

#[derive(Parser)]
pub struct McpRemoveArgs {
    /// MCP server name to remove
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_global() -> super::super::GlobalOpts {
        super::super::GlobalOpts {
            target_dir: None,
            config: None,
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        }
    }

    #[test]
    fn mcp_list_returns_not_implemented() {
        let cmd = McpCommand::List;
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "Expected 'not yet implemented' error, got: {err_msg}"
        );
    }

    #[test]
    fn mcp_add_returns_not_implemented() {
        let cmd = McpCommand::Add(McpAddArgs {
            name: "test-server".to_string(),
            image: "test:latest".to_string(),
            command: None,
            envs: vec![],
            volumes: vec![],
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "Expected 'not yet implemented' error, got: {err_msg}"
        );
    }

    #[test]
    fn mcp_add_with_all_options_returns_not_implemented() {
        let cmd = McpCommand::Add(McpAddArgs {
            name: "test-server".to_string(),
            image: "test:latest".to_string(),
            command: Some("serve".to_string()),
            envs: vec!["KEY=VAL".to_string()],
            volumes: vec!["/host:/container".to_string()],
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "Expected 'not yet implemented' error, got: {err_msg}"
        );
    }

    #[test]
    fn mcp_remove_returns_not_implemented() {
        let cmd = McpCommand::Remove(McpRemoveArgs {
            name: "test-server".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "Expected 'not yet implemented' error, got: {err_msg}"
        );
    }
}

pub fn run(cmd: &McpCommand, _global: &super::GlobalOpts) -> crate::error::Result<()> {
    match cmd {
        McpCommand::List => Err(crate::error::Error::Other(
            "mcp list is not yet implemented".to_string(),
        )),
        McpCommand::Add(_args) => Err(crate::error::Error::Other(
            "mcp add is not yet implemented".to_string(),
        )),
        McpCommand::Remove(_args) => Err(crate::error::Error::Other(
            "mcp remove is not yet implemented".to_string(),
        )),
    }
}
