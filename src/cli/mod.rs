pub mod config_cmd;
pub mod doctor;
pub mod generate;
pub mod init;
pub mod mcp;
pub mod module;
pub mod service;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cc-container",
    version,
    about = "Generate containerized AI coding agent environments (Claude Code / Codex)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub global: GlobalOpts,
}

#[derive(Parser, Debug)]
pub struct GlobalOpts {
    /// Project / target directory
    #[arg(long, global = true)]
    pub target_dir: Option<PathBuf>,

    /// Path to config file (default: <target-dir>/cc-container.toml)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Color mode
    #[arg(long, default_value = "auto", global = true)]
    pub color: ColorMode,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive project initialization
    Init(init::InitArgs),

    /// Generate output files (Dockerfile, docker-compose.yml, etc.)
    Generate(generate::GenerateArgs),

    /// Manage Dockerfile modules
    #[command(subcommand)]
    Module(module::ModuleCommand),

    /// Manage compose service templates
    #[command(subcommand)]
    Service(service::ServiceCommand),

    /// Manage MCP servers
    #[command(subcommand)]
    Mcp(mcp::McpCommand),

    /// Configuration management
    #[command(subcommand)]
    Config(config_cmd::ConfigCommand),

    /// Diagnose common issues
    Doctor(doctor::DoctorArgs),

    /// Generate shell completions
    Completions(CompletionsArgs),
}

#[derive(Parser)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    pub shell: clap_complete::Shell,
}

pub(crate) fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let (key, val) = s
        .split_once('=')
        .ok_or_else(|| format!("invalid key=value pair: {s}"))?;
    Ok((key.to_string(), val.to_string()))
}
