use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct GenerateArgs {
    /// Output directory (default: target-dir or CWD)
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Print generated files to stdout instead of writing
    #[arg(long)]
    pub dry_run: bool,

    /// Only generate specific file types
    #[arg(long, value_delimiter = ',')]
    pub only: Option<Vec<GenerateTarget>>,

    /// Show diff against existing files before writing
    #[arg(long)]
    pub diff: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum GenerateTarget {
    Dockerfile,
    Compose,
    Firewall,
    Env,
    Mcp,
}

pub fn run(args: &GenerateArgs, global: &super::GlobalOpts) -> crate::error::Result<()> {
    let target_dir = global
        .target_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));

    let config_path = global
        .config
        .clone()
        .unwrap_or_else(|| target_dir.join("cc-container.toml"));

    let config = crate::config::load_effective_config(&config_path)?;

    // Validate config
    let warnings = crate::config::validate::validate_config(&config)?;
    for w in &warnings {
        eprintln!("warning: {w}");
    }

    let output_dir = args.output.clone().unwrap_or(target_dir);
    std::fs::create_dir_all(&output_dir)?;

    let targets = args.only.clone().unwrap_or_else(|| {
        vec![
            GenerateTarget::Dockerfile,
            GenerateTarget::Compose,
            GenerateTarget::Env,
            GenerateTarget::Firewall,
            GenerateTarget::Mcp,
        ]
    });

    for target in &targets {
        match target {
            GenerateTarget::Dockerfile => {
                generate_dockerfile(&config, &output_dir, args.dry_run)?;
                // Co-generate files the Dockerfile COPYs, but only if they
                // aren't already in the target list (avoids double output)
                let firewall_already_targeted = targets.iter().any(|t| matches!(t, GenerateTarget::Firewall));
                if config.firewall.enabled && !firewall_already_targeted {
                    generate_firewall(&config, &output_dir, args.dry_run)?;
                }
            }
            GenerateTarget::Compose => {
                generate_compose(&config, &output_dir, args.dry_run)?;
            }
            GenerateTarget::Env => {
                generate_env(&config, &output_dir, args.dry_run)?;
            }
            GenerateTarget::Firewall => {
                if config.firewall.enabled {
                    generate_firewall(&config, &output_dir, args.dry_run)?;
                }
            }
            GenerateTarget::Mcp => {
                if !config.mcp.is_empty() {
                    generate_mcp(&config, &output_dir, args.dry_run)?;
                }
            }
        }
    }

    if !args.dry_run {
        eprintln!("Generated files in {}", output_dir.display());
    }

    Ok(())
}

fn generate_dockerfile(
    config: &crate::config::ProjectConfig,
    output_dir: &std::path::Path,
    dry_run: bool,
) -> crate::error::Result<()> {
    use crate::config::project::AgentType;
    use crate::module::{DockerfileGenerator, ModuleRegistry};

    let registry = ModuleRegistry::new();

    // Load user modules from project directory if present
    // (registry.load_user_modules is available but we skip for now)

    let generator = DockerfileGenerator::new(&registry);

    match config.agent.agent_type {
        AgentType::Both => {
            // Generate two separate Dockerfiles
            for (agent, filename) in [
                (AgentType::Claude, "Dockerfile.claude"),
                (AgentType::Codex, "Dockerfile.codex"),
            ] {
                let content = generator.generate(config, agent)?;
                write_output(output_dir, filename, &content, dry_run)?;
            }
        }
        agent_type => {
            let content = generator.generate(config, agent_type)?;
            write_output(output_dir, "Dockerfile", &content, dry_run)?;
        }
    }

    Ok(())
}

fn write_output(
    output_dir: &std::path::Path,
    filename: &str,
    content: &str,
    dry_run: bool,
) -> crate::error::Result<()> {
    if dry_run {
        println!("=== {} ===", filename);
        println!("{content}");
    } else {
        let path = output_dir.join(filename);
        std::fs::write(&path, content)?;
        eprintln!("  wrote {}", path.display());
    }
    Ok(())
}

fn generate_compose(
    config: &crate::config::ProjectConfig,
    output_dir: &std::path::Path,
    dry_run: bool,
) -> crate::error::Result<()> {
    let compose = crate::compose::generator::generate(config)?;
    let yaml = serde_yaml::to_string(&compose)
        .map_err(crate::error::Error::YamlSerialize)?;
    write_output(output_dir, "docker-compose.yml", &yaml, dry_run)
}

fn generate_env(
    config: &crate::config::ProjectConfig,
    output_dir: &std::path::Path,
    dry_run: bool,
) -> crate::error::Result<()> {
    let content = crate::compose::env::generate_env_example(config);
    write_output(output_dir, ".env.example", &content, dry_run)
}

fn generate_firewall(
    config: &crate::config::ProjectConfig,
    output_dir: &std::path::Path,
    dry_run: bool,
) -> crate::error::Result<()> {
    let content = crate::firewall::generator::generate(config);
    write_output(output_dir, "init-firewall.sh", &content, dry_run)
}

fn generate_mcp(
    config: &crate::config::ProjectConfig,
    output_dir: &std::path::Path,
    dry_run: bool,
) -> crate::error::Result<()> {
    let content = crate::mcp::config::generate_mcp_json(config)?;
    write_output(output_dir, ".mcp.json", &content, dry_run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::*;
    use indexmap::IndexMap;

    /// Build a minimal valid ProjectConfig for testing.
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

    /// Write a minimal config file and return path.
    fn write_config(dir: &std::path::Path) -> std::path::PathBuf {
        let config = minimal_config();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let path = dir.join("cc-container.toml");
        std::fs::write(&path, toml_str).unwrap();
        path
    }

    // --- write_output tests ---

    #[test]
    fn write_output_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        write_output(dir.path(), "test.txt", "hello", false).unwrap();
        let content = std::fs::read_to_string(dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn write_output_dry_run_does_not_create_file() {
        let dir = tempfile::tempdir().unwrap();
        write_output(dir.path(), "test.txt", "hello", true).unwrap();
        assert!(!dir.path().join("test.txt").exists());
    }

    // --- run() integration tests ---

    #[test]
    fn generate_creates_dockerfile() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let out_dir = tempfile::tempdir().unwrap();

        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: false,
            only: Some(vec![GenerateTarget::Dockerfile]),
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(out_dir.path().join("Dockerfile").exists());
    }

    #[test]
    fn generate_creates_compose_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let out_dir = tempfile::tempdir().unwrap();

        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: false,
            only: Some(vec![GenerateTarget::Compose]),
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(out_dir.path().join("docker-compose.yml").exists());
    }

    #[test]
    fn generate_creates_env_example() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let out_dir = tempfile::tempdir().unwrap();

        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: false,
            only: Some(vec![GenerateTarget::Env]),
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(out_dir.path().join(".env.example").exists());
    }

    #[test]
    fn generate_firewall_skipped_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let out_dir = tempfile::tempdir().unwrap();

        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: false,
            only: Some(vec![GenerateTarget::Firewall]),
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        // Firewall is disabled by default, so no file should be created
        assert!(!out_dir.path().join("init-firewall.sh").exists());
    }

    #[test]
    fn generate_firewall_created_when_enabled() {
        let dir = tempfile::tempdir().unwrap();

        let mut config = minimal_config();
        config.firewall.enabled = true;
        config.firewall.allowed_domains = vec!["api.anthropic.com".to_string()];
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let config_path = dir.path().join("cc-container.toml");
        std::fs::write(&config_path, toml_str).unwrap();

        let out_dir = tempfile::tempdir().unwrap();

        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: false,
            only: Some(vec![GenerateTarget::Firewall]),
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(out_dir.path().join("init-firewall.sh").exists());
    }

    #[test]
    fn generate_mcp_skipped_when_empty() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let out_dir = tempfile::tempdir().unwrap();

        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: false,
            only: Some(vec![GenerateTarget::Mcp]),
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(!out_dir.path().join(".mcp.json").exists());
    }

    #[test]
    fn generate_all_targets_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let out_dir = tempfile::tempdir().unwrap();

        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: false,
            only: None, // default: all targets
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(out_dir.path().join("Dockerfile").exists());
        assert!(out_dir.path().join("docker-compose.yml").exists());
        assert!(out_dir.path().join(".env.example").exists());
    }

    #[test]
    fn generate_dry_run_creates_no_files() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let out_dir = tempfile::tempdir().unwrap();

        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: true,
            only: None,
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        // No files should be written in dry-run mode
        let entries: Vec<_> = std::fs::read_dir(out_dir.path()).unwrap().collect();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn generate_errors_on_missing_config() {
        let dir = tempfile::tempdir().unwrap();
        let missing_config = dir.path().join("nonexistent.toml");

        let args = GenerateArgs {
            output: None,
            dry_run: false,
            only: None,
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(missing_config),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        let result = run(&args, &global);
        assert!(result.is_err());
    }

    #[test]
    fn generate_both_agent_creates_two_dockerfiles() {
        let dir = tempfile::tempdir().unwrap();

        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::ApiKey,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let config_path = dir.path().join("cc-container.toml");
        std::fs::write(&config_path, toml_str).unwrap();

        let out_dir = tempfile::tempdir().unwrap();
        let args = GenerateArgs {
            output: Some(out_dir.path().to_path_buf()),
            dry_run: false,
            only: Some(vec![GenerateTarget::Dockerfile]),
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(out_dir.path().join("Dockerfile.claude").exists());
        assert!(out_dir.path().join("Dockerfile.codex").exists());
    }

    #[test]
    fn generate_creates_output_dir_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let out_dir = dir.path().join("nested").join("output");

        let args = GenerateArgs {
            output: Some(out_dir.clone()),
            dry_run: false,
            only: Some(vec![GenerateTarget::Env]),
            diff: false,
        };
        let global = super::super::GlobalOpts {
            target_dir: Some(dir.path().to_path_buf()),
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        };

        run(&args, &global).unwrap();
        assert!(out_dir.join(".env.example").exists());
    }
}
