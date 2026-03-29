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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // --- parse_key_val helper ---

    #[test]
    fn parse_key_val_valid() {
        let (k, v) = parse_key_val("foo=bar").unwrap();
        assert_eq!(k, "foo");
        assert_eq!(v, "bar");
    }

    #[test]
    fn parse_key_val_with_equals_in_value() {
        let (k, v) = parse_key_val("key=a=b").unwrap();
        assert_eq!(k, "key");
        assert_eq!(v, "a=b");
    }

    #[test]
    fn parse_key_val_empty_value() {
        let (k, v) = parse_key_val("key=").unwrap();
        assert_eq!(k, "key");
        assert_eq!(v, "");
    }

    #[test]
    fn parse_key_val_no_equals() {
        let err = parse_key_val("noequals").unwrap_err();
        assert!(err.contains("invalid key=value pair"));
    }

    // --- CLI parsing: top-level subcommands ---

    #[test]
    fn parse_init_bare() {
        let cli = Cli::try_parse_from(["cc-container", "init"]).unwrap();
        assert!(matches!(cli.command, Commands::Init(_)));
    }

    #[test]
    fn parse_init_with_template() {
        let cli = Cli::try_parse_from(["cc-container", "init", "--template", "claude"]).unwrap();
        if let Commands::Init(args) = &cli.command {
            assert!(args.template.is_some());
        } else {
            panic!("expected Init command");
        }
    }

    #[test]
    fn parse_init_no_interactive() {
        let cli = Cli::try_parse_from(["cc-container", "init", "--no-interactive"]).unwrap();
        if let Commands::Init(args) = &cli.command {
            assert!(args.no_interactive);
        } else {
            panic!("expected Init command");
        }
    }

    #[test]
    fn parse_init_with_agent() {
        let cli = Cli::try_parse_from(["cc-container", "init", "--agent", "codex"]).unwrap();
        if let Commands::Init(args) = &cli.command {
            assert!(args.agent.is_some());
        } else {
            panic!("expected Init command");
        }
    }

    #[test]
    fn parse_generate_bare() {
        let cli = Cli::try_parse_from(["cc-container", "generate"]).unwrap();
        assert!(matches!(cli.command, Commands::Generate(_)));
    }

    #[test]
    fn parse_generate_dry_run() {
        let cli = Cli::try_parse_from(["cc-container", "generate", "--dry-run"]).unwrap();
        if let Commands::Generate(args) = &cli.command {
            assert!(args.dry_run);
        } else {
            panic!("expected Generate command");
        }
    }

    #[test]
    fn parse_generate_output() {
        let cli =
            Cli::try_parse_from(["cc-container", "generate", "--output", "/tmp/out"]).unwrap();
        if let Commands::Generate(args) = &cli.command {
            assert_eq!(
                args.output.as_deref(),
                Some(std::path::Path::new("/tmp/out"))
            );
        } else {
            panic!("expected Generate command");
        }
    }

    #[test]
    fn parse_generate_only_dockerfile() {
        let cli =
            Cli::try_parse_from(["cc-container", "generate", "--only", "dockerfile"]).unwrap();
        if let Commands::Generate(args) = &cli.command {
            let targets = args.only.as_ref().unwrap();
            assert_eq!(targets.len(), 1);
            assert!(matches!(targets[0], generate::GenerateTarget::Dockerfile));
        } else {
            panic!("expected Generate command");
        }
    }

    #[test]
    fn parse_generate_only_multiple() {
        let cli = Cli::try_parse_from([
            "cc-container",
            "generate",
            "--only",
            "dockerfile,compose,env",
        ])
        .unwrap();
        if let Commands::Generate(args) = &cli.command {
            let targets = args.only.as_ref().unwrap();
            assert_eq!(targets.len(), 3);
        } else {
            panic!("expected Generate command");
        }
    }

    // --- Module subcommands ---

    #[test]
    fn parse_module_list() {
        let cli = Cli::try_parse_from(["cc-container", "module", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Module(module::ModuleCommand::List(_))
        ));
    }

    #[test]
    fn parse_module_list_with_category() {
        let cli =
            Cli::try_parse_from(["cc-container", "module", "list", "--category", "lang"]).unwrap();
        if let Commands::Module(module::ModuleCommand::List(args)) = &cli.command {
            assert_eq!(args.category.as_deref(), Some("lang"));
        } else {
            panic!("expected Module List command");
        }
    }

    #[test]
    fn parse_module_info() {
        let cli = Cli::try_parse_from(["cc-container", "module", "info", "node"]).unwrap();
        if let Commands::Module(module::ModuleCommand::Info(args)) = &cli.command {
            assert_eq!(args.name, "node");
        } else {
            panic!("expected Module Info command");
        }
    }

    #[test]
    fn parse_module_add() {
        let cli = Cli::try_parse_from(["cc-container", "module", "add", "node", "git"]).unwrap();
        if let Commands::Module(module::ModuleCommand::Add(args)) = &cli.command {
            assert_eq!(args.names, vec!["node", "git"]);
        } else {
            panic!("expected Module Add command");
        }
    }

    #[test]
    fn parse_module_add_with_params() {
        let cli = Cli::try_parse_from([
            "cc-container",
            "module",
            "add",
            "node",
            "--with",
            "version=22",
        ])
        .unwrap();
        if let Commands::Module(module::ModuleCommand::Add(args)) = &cli.command {
            assert_eq!(args.names, vec!["node"]);
            assert_eq!(args.params.len(), 1);
            assert_eq!(args.params[0], ("version".to_string(), "22".to_string()));
        } else {
            panic!("expected Module Add command");
        }
    }

    #[test]
    fn parse_module_remove() {
        let cli = Cli::try_parse_from(["cc-container", "module", "remove", "git"]).unwrap();
        if let Commands::Module(module::ModuleCommand::Remove(args)) = &cli.command {
            assert_eq!(args.names, vec!["git"]);
        } else {
            panic!("expected Module Remove command");
        }
    }

    #[test]
    fn parse_module_create() {
        let cli =
            Cli::try_parse_from(["cc-container", "module", "create", "--name", "mymod"]).unwrap();
        if let Commands::Module(module::ModuleCommand::Create(args)) = &cli.command {
            assert_eq!(args.name, "mymod");
            assert!(args.dir.is_none());
        } else {
            panic!("expected Module Create command");
        }
    }

    #[test]
    fn parse_module_create_with_dir() {
        let cli = Cli::try_parse_from([
            "cc-container",
            "module",
            "create",
            "--name",
            "mymod",
            "--dir",
            "/tmp/mods",
        ])
        .unwrap();
        if let Commands::Module(module::ModuleCommand::Create(args)) = &cli.command {
            assert_eq!(args.dir.as_deref(), Some(std::path::Path::new("/tmp/mods")));
        } else {
            panic!("expected Module Create command");
        }
    }

    // --- Service subcommands ---

    #[test]
    fn parse_service_list() {
        let cli = Cli::try_parse_from(["cc-container", "service", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Service(service::ServiceCommand::List(_))
        ));
    }

    #[test]
    fn parse_service_list_with_category() {
        let cli =
            Cli::try_parse_from(["cc-container", "service", "list", "--category", "database"])
                .unwrap();
        if let Commands::Service(service::ServiceCommand::List(args)) = &cli.command {
            assert_eq!(args.category.as_deref(), Some("database"));
        } else {
            panic!("expected Service List command");
        }
    }

    #[test]
    fn parse_service_info() {
        let cli = Cli::try_parse_from(["cc-container", "service", "info", "postgres"]).unwrap();
        if let Commands::Service(service::ServiceCommand::Info(args)) = &cli.command {
            assert_eq!(args.name, "postgres");
        } else {
            panic!("expected Service Info command");
        }
    }

    #[test]
    fn parse_service_add() {
        let cli =
            Cli::try_parse_from(["cc-container", "service", "add", "postgres", "redis"]).unwrap();
        if let Commands::Service(service::ServiceCommand::Add(args)) = &cli.command {
            assert_eq!(args.names, vec!["postgres", "redis"]);
        } else {
            panic!("expected Service Add command");
        }
    }

    #[test]
    fn parse_service_add_with_params() {
        let cli = Cli::try_parse_from([
            "cc-container",
            "service",
            "add",
            "postgres",
            "--with",
            "port=5433",
        ])
        .unwrap();
        if let Commands::Service(service::ServiceCommand::Add(args)) = &cli.command {
            assert_eq!(args.params.len(), 1);
            assert_eq!(args.params[0], ("port".to_string(), "5433".to_string()));
        } else {
            panic!("expected Service Add command");
        }
    }

    #[test]
    fn parse_service_remove() {
        let cli = Cli::try_parse_from(["cc-container", "service", "remove", "redis"]).unwrap();
        if let Commands::Service(service::ServiceCommand::Remove(args)) = &cli.command {
            assert_eq!(args.names, vec!["redis"]);
        } else {
            panic!("expected Service Remove command");
        }
    }

    // --- MCP subcommands ---

    #[test]
    fn parse_mcp_list() {
        let cli = Cli::try_parse_from(["cc-container", "mcp", "list"]).unwrap();
        assert!(matches!(cli.command, Commands::Mcp(mcp::McpCommand::List)));
    }

    #[test]
    fn parse_mcp_add() {
        let cli = Cli::try_parse_from([
            "cc-container",
            "mcp",
            "add",
            "my-server",
            "--image",
            "myimg:latest",
        ])
        .unwrap();
        if let Commands::Mcp(mcp::McpCommand::Add(args)) = &cli.command {
            assert_eq!(args.name, "my-server");
            assert_eq!(args.image, "myimg:latest");
            assert!(args.command.is_none());
        } else {
            panic!("expected Mcp Add command");
        }
    }

    #[test]
    fn parse_mcp_add_with_all_options() {
        let cli = Cli::try_parse_from([
            "cc-container",
            "mcp",
            "add",
            "my-server",
            "--image",
            "myimg:latest",
            "--command",
            "serve",
            "--env",
            "KEY=VAL",
            "--volume",
            "/host:/container",
        ])
        .unwrap();
        if let Commands::Mcp(mcp::McpCommand::Add(args)) = &cli.command {
            assert_eq!(args.name, "my-server");
            assert_eq!(args.image, "myimg:latest");
            assert_eq!(args.command.as_deref(), Some("serve"));
            assert_eq!(args.envs, vec!["KEY=VAL"]);
            assert_eq!(args.volumes, vec!["/host:/container"]);
        } else {
            panic!("expected Mcp Add command");
        }
    }

    #[test]
    fn parse_mcp_remove() {
        let cli = Cli::try_parse_from(["cc-container", "mcp", "remove", "my-server"]).unwrap();
        if let Commands::Mcp(mcp::McpCommand::Remove(args)) = &cli.command {
            assert_eq!(args.name, "my-server");
        } else {
            panic!("expected Mcp Remove command");
        }
    }

    // --- Config subcommands ---

    #[test]
    fn parse_config_show() {
        let cli = Cli::try_parse_from(["cc-container", "config", "show"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Config(config_cmd::ConfigCommand::Show(_))
        ));
    }

    #[test]
    fn parse_config_show_json() {
        let cli =
            Cli::try_parse_from(["cc-container", "config", "show", "--format", "json"]).unwrap();
        if let Commands::Config(config_cmd::ConfigCommand::Show(args)) = &cli.command {
            assert!(matches!(args.format, config_cmd::ConfigFormat::Json));
        } else {
            panic!("expected Config Show command");
        }
    }

    #[test]
    fn parse_config_show_yaml() {
        let cli =
            Cli::try_parse_from(["cc-container", "config", "show", "--format", "yaml"]).unwrap();
        if let Commands::Config(config_cmd::ConfigCommand::Show(args)) = &cli.command {
            assert!(matches!(args.format, config_cmd::ConfigFormat::Yaml));
        } else {
            panic!("expected Config Show command");
        }
    }

    #[test]
    fn parse_config_validate() {
        let cli = Cli::try_parse_from(["cc-container", "config", "validate"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Config(config_cmd::ConfigCommand::Validate)
        ));
    }

    #[test]
    fn parse_config_set() {
        let cli =
            Cli::try_parse_from(["cc-container", "config", "set", "agent.type", "codex"]).unwrap();
        if let Commands::Config(config_cmd::ConfigCommand::Set(args)) = &cli.command {
            assert_eq!(args.key, "agent.type");
            assert_eq!(args.value, "codex");
        } else {
            panic!("expected Config Set command");
        }
    }

    #[test]
    fn parse_config_get() {
        let cli = Cli::try_parse_from(["cc-container", "config", "get", "project.name"]).unwrap();
        if let Commands::Config(config_cmd::ConfigCommand::Get(args)) = &cli.command {
            assert_eq!(args.key, "project.name");
        } else {
            panic!("expected Config Get command");
        }
    }

    #[test]
    fn parse_config_edit() {
        let cli = Cli::try_parse_from(["cc-container", "config", "edit"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Config(config_cmd::ConfigCommand::Edit)
        ));
    }

    // --- Doctor command ---

    #[test]
    fn parse_doctor_verbose() {
        let cli = Cli::try_parse_from(["cc-container", "doctor"]).unwrap();
        // -v now works via global verbose
        let cli = Cli::try_parse_from(["cc-container", "-v", "doctor"]).unwrap();
        assert_eq!(cli.global.verbose, 1);
    }

    // --- Completions command ---

    #[test]
    fn parse_completions_bash() {
        let cli = Cli::try_parse_from(["cc-container", "completions", "bash"]).unwrap();
        if let Commands::Completions(args) = &cli.command {
            assert_eq!(args.shell, clap_complete::Shell::Bash);
        } else {
            panic!("expected Completions command");
        }
    }

    #[test]
    fn parse_completions_zsh() {
        let cli = Cli::try_parse_from(["cc-container", "completions", "zsh"]).unwrap();
        if let Commands::Completions(args) = &cli.command {
            assert_eq!(args.shell, clap_complete::Shell::Zsh);
        } else {
            panic!("expected Completions command");
        }
    }

    #[test]
    fn parse_completions_fish() {
        let cli = Cli::try_parse_from(["cc-container", "completions", "fish"]).unwrap();
        if let Commands::Completions(args) = &cli.command {
            assert_eq!(args.shell, clap_complete::Shell::Fish);
        } else {
            panic!("expected Completions command");
        }
    }

    // --- Global args ---

    #[test]
    fn parse_global_target_dir() {
        let cli =
            Cli::try_parse_from(["cc-container", "--target-dir", "/tmp/proj", "generate"]).unwrap();
        assert_eq!(
            cli.global.target_dir.as_deref(),
            Some(std::path::Path::new("/tmp/proj"))
        );
    }

    #[test]
    fn parse_global_config() {
        let cli =
            Cli::try_parse_from(["cc-container", "--config", "/tmp/cfg.toml", "generate"]).unwrap();
        assert_eq!(
            cli.global.config.as_deref(),
            Some(std::path::Path::new("/tmp/cfg.toml"))
        );
    }

    #[test]
    fn parse_global_verbose_single() {
        let cli = Cli::try_parse_from(["cc-container", "-v", "generate"]).unwrap();
        assert_eq!(cli.global.verbose, 1);
    }

    #[test]
    fn parse_global_verbose_double() {
        let cli = Cli::try_parse_from(["cc-container", "-vv", "generate"]).unwrap();
        assert_eq!(cli.global.verbose, 2);
    }

    #[test]
    fn parse_global_verbose_triple() {
        let cli = Cli::try_parse_from(["cc-container", "-vvv", "generate"]).unwrap();
        assert_eq!(cli.global.verbose, 3);
    }

    #[test]
    fn parse_global_quiet() {
        let cli = Cli::try_parse_from(["cc-container", "-q", "generate"]).unwrap();
        assert!(cli.global.quiet);
    }

    #[test]
    fn parse_global_color_never() {
        let cli = Cli::try_parse_from(["cc-container", "--color", "never", "generate"]).unwrap();
        assert!(matches!(cli.global.color, ColorMode::Never));
    }

    #[test]
    fn parse_global_color_always() {
        let cli = Cli::try_parse_from(["cc-container", "--color", "always", "generate"]).unwrap();
        assert!(matches!(cli.global.color, ColorMode::Always));
    }

    #[test]
    fn parse_global_color_default_auto() {
        let cli = Cli::try_parse_from(["cc-container", "generate"]).unwrap();
        assert!(matches!(cli.global.color, ColorMode::Auto));
    }

    #[test]
    fn parse_global_opts_after_subcommand() {
        // Global opts can appear after the subcommand
        let cli = Cli::try_parse_from(["cc-container", "generate", "--target-dir", "/tmp", "-v"])
            .unwrap();
        assert_eq!(
            cli.global.target_dir.as_deref(),
            Some(std::path::Path::new("/tmp"))
        );
        assert_eq!(cli.global.verbose, 1);
    }

    // --- Errors ---

    #[test]
    fn parse_no_subcommand_errors() {
        let result = Cli::try_parse_from(["cc-container"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_unknown_subcommand_errors() {
        let result = Cli::try_parse_from(["cc-container", "nonexistent"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_module_info_missing_name_errors() {
        let result = Cli::try_parse_from(["cc-container", "module", "info"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_mcp_add_missing_image_errors() {
        let result = Cli::try_parse_from(["cc-container", "mcp", "add", "server"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_config_set_missing_value_errors() {
        let result = Cli::try_parse_from(["cc-container", "config", "set", "key"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_completions_missing_shell_errors() {
        let result = Cli::try_parse_from(["cc-container", "completions"]);
        assert!(result.is_err());
    }
}
