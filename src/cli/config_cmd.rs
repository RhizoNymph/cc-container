use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show effective merged configuration
    Show(ConfigShowArgs),
    /// Validate configuration files
    Validate,
    /// Set a configuration value
    Set(ConfigSetArgs),
    /// Get a configuration value
    Get(ConfigGetArgs),
    /// Open config in $EDITOR
    Edit,
}

#[derive(Parser)]
pub struct ConfigShowArgs {
    /// Output format
    #[arg(long, default_value = "toml")]
    pub format: ConfigFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
}

#[derive(Parser)]
pub struct ConfigSetArgs {
    /// Config key (dotted path, e.g. "agent.type")
    pub key: String,
    /// Value to set
    pub value: String,
}

#[derive(Parser)]
pub struct ConfigGetArgs {
    /// Config key (dotted path)
    pub key: String,
}

pub fn run(cmd: &ConfigCommand, global: &super::GlobalOpts) -> crate::error::Result<()> {
    let target_dir = global
        .target_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));

    let config_path = global
        .config
        .clone()
        .unwrap_or_else(|| target_dir.join("cc-container.toml"));

    match cmd {
        ConfigCommand::Show(args) => {
            let config = crate::config::load_effective_config(&config_path)?;
            let output = match args.format {
                ConfigFormat::Toml => {
                    toml::to_string_pretty(&config).map_err(crate::error::Error::TomlSerialize)?
                }
                ConfigFormat::Json => serde_json::to_string_pretty(&config)
                    .map_err(crate::error::Error::JsonSerialize)?,
                ConfigFormat::Yaml => serde_yaml::to_string(&config)
                    .map_err(crate::error::Error::YamlSerialize)?,
            };
            println!("{output}");
        }
        ConfigCommand::Validate => {
            let config = crate::config::load_effective_config(&config_path)?;
            let warnings = crate::config::validate::validate_config(&config)?;
            if warnings.is_empty() {
                eprintln!("Configuration is valid.");
            } else {
                for w in &warnings {
                    eprintln!("warning: {w}");
                }
            }
        }
        ConfigCommand::Set(args) => {
            eprintln!("Config set {}={} not yet implemented", args.key, args.value);
        }
        ConfigCommand::Get(args) => {
            eprintln!("Config get {} not yet implemented", args.key);
        }
        ConfigCommand::Edit => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let parts: Vec<&str> = editor.split_whitespace().collect();
            if parts.is_empty() {
                return Err(crate::error::Error::Other("EDITOR is empty".to_string()));
            }
            let status = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .arg(&config_path)
                .status()?;
            if !status.success() {
                return Err(crate::error::Error::Other(format!(
                    "editor '{}' exited with non-zero status",
                    editor
                )));
            }
        }
    }
    Ok(())
}
