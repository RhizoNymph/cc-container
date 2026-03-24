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
    fn mcp_list_runs_without_error() {
        let cmd = McpCommand::List;
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_add_runs_without_error() {
        let cmd = McpCommand::Add(McpAddArgs {
            name: "test-server".to_string(),
            image: "test:latest".to_string(),
            command: None,
            envs: vec![],
            volumes: vec![],
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_add_with_all_options_runs_without_error() {
        let cmd = McpCommand::Add(McpAddArgs {
            name: "test-server".to_string(),
            image: "test:latest".to_string(),
            command: Some("serve".to_string()),
            envs: vec!["KEY=VAL".to_string()],
            volumes: vec!["/host:/container".to_string()],
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_remove_runs_without_error() {
        let cmd = McpCommand::Remove(McpRemoveArgs {
            name: "test-server".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }
}

pub fn run(cmd: &McpCommand, _global: &super::GlobalOpts) -> crate::error::Result<()> {
    match cmd {
        McpCommand::List => {
            eprintln!("MCP list not yet implemented");
        }
        McpCommand::Add(args) => {
            eprintln!("MCP add '{}' not yet implemented", args.name);
        }
        McpCommand::Remove(args) => {
            eprintln!("MCP remove '{}' not yet implemented", args.name);
        }
    }
    Ok(())
}
