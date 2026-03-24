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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::*;
    use indexmap::IndexMap;

    fn minimal_config() -> ProjectConfig {
        ProjectConfig {
            project: ProjectMeta {
                name: "test-project".to_string(),
                description: None,
            },
            agent: AgentConfig {
                agent_type: AgentType::Claude,
                claude_version: "latest".to_string(),
                codex_version: "latest".to_string(),
            },
            image: ImageConfig::default(),
            modules: IndexMap::new(),
            auth: AuthConfig {
                claude: Some(ClaudeAuthConfig {
                    method: ClaudeAuthMethod::ApiKey,
                }),
                codex: None,
            },
            firewall: FirewallConfig::default(),
            workspace: WorkspaceConfig::default(),
            volumes: IndexMap::new(),
            environment: EnvironmentConfig::default(),
            services: IndexMap::new(),
            mcp: IndexMap::new(),
            runtime: RuntimeConfig::default(),
        }
    }

    fn write_config(dir: &std::path::Path) -> std::path::PathBuf {
        let config = minimal_config();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let path = dir.join("cc-container.toml");
        std::fs::write(&path, toml_str).unwrap();
        path
    }

    fn make_global(config_path: std::path::PathBuf) -> super::super::GlobalOpts {
        super::super::GlobalOpts {
            target_dir: None,
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        }
    }

    #[test]
    fn config_show_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global(config_path);

        let cmd = ConfigCommand::Show(ConfigShowArgs {
            format: ConfigFormat::Toml,
        });
        let result = run(&cmd, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn config_show_json() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global(config_path);

        let cmd = ConfigCommand::Show(ConfigShowArgs {
            format: ConfigFormat::Json,
        });
        let result = run(&cmd, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn config_show_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global(config_path);

        let cmd = ConfigCommand::Show(ConfigShowArgs {
            format: ConfigFormat::Yaml,
        });
        let result = run(&cmd, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn config_show_errors_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nonexistent.toml");
        let global = make_global(missing);

        let cmd = ConfigCommand::Show(ConfigShowArgs {
            format: ConfigFormat::Toml,
        });
        let result = run(&cmd, &global);
        assert!(result.is_err());
    }

    #[test]
    fn config_validate_on_valid_config() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global(config_path);

        let cmd = ConfigCommand::Validate;
        let result = run(&cmd, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn config_validate_errors_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nonexistent.toml");
        let global = make_global(missing);

        let cmd = ConfigCommand::Validate;
        let result = run(&cmd, &global);
        assert!(result.is_err());
    }

    #[test]
    fn config_set_runs_without_error() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global(config_path);

        let cmd = ConfigCommand::Set(ConfigSetArgs {
            key: "agent.type".to_string(),
            value: "codex".to_string(),
        });
        let result = run(&cmd, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn config_get_runs_without_error() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global(config_path);

        let cmd = ConfigCommand::Get(ConfigGetArgs {
            key: "project.name".to_string(),
        });
        let result = run(&cmd, &global);
        assert!(result.is_ok());
    }

    #[test]
    fn config_edit_fails_with_nonexistent_editor() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global(config_path);

        // Set EDITOR to a nonexistent binary
        // SAFETY: test-only env var manipulation
        unsafe { std::env::set_var("EDITOR", "nonexistent-editor-binary-12345") };
        let cmd = ConfigCommand::Edit;
        let result = run(&cmd, &global);
        assert!(result.is_err());
        // Clean up
        unsafe { std::env::remove_var("EDITOR") };
    }

    #[test]
    fn config_show_invalid_toml_errors() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("cc-container.toml");
        std::fs::write(&config_path, "invalid toml {{{{").unwrap();
        let global = make_global(config_path);

        let cmd = ConfigCommand::Show(ConfigShowArgs {
            format: ConfigFormat::Toml,
        });
        let result = run(&cmd, &global);
        assert!(result.is_err());
    }
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
