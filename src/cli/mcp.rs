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
