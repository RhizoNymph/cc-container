use clap::Parser;
use std::path::PathBuf;

pub use crate::generate::GenerateTarget;
use crate::generate::{GenerateOptions, GeneratedProject};

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

pub fn run(args: &GenerateArgs, global: &super::GlobalOpts) -> crate::error::Result<()> {
    let target_dir = global
        .target_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));

    let config_path = global
        .config
        .clone()
        .unwrap_or_else(|| target_dir.join("cc-container.toml"));

    let output_dir = args.output.clone().unwrap_or(target_dir);
    let options = GenerateOptions {
        targets: args
            .only
            .clone()
            .unwrap_or_else(crate::generate::default_targets),
        validate: true,
    };

    let project = crate::generate::generate_from_path(&config_path, options)?;

    for warning in &project.warnings {
        eprintln!("warning: {warning}");
    }

    if args.dry_run {
        print_generated(&project);
    } else {
        crate::generate::write_generated(&project, &output_dir)?;
        for file in &project.files {
            eprintln!("  wrote {}", output_dir.join(&file.path).display());
        }
        eprintln!("Generated files in {}", output_dir.display());
    }

    Ok(())
}

fn print_generated(project: &GeneratedProject) {
    for file in &project.files {
        println!("=== {} ===", file.path.display());
        println!("{}", file.contents);
    }
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
            helm: HelmConfig::default(),
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
        assert!(!out_dir.path().join("init-firewall.sh").exists());
    }

    #[test]
    fn generate_firewall_created_when_enabled() {
        let dir = tempfile::tempdir().unwrap();

        let mut config = minimal_config();
        config.firewall.enabled = true;
        config.runtime.cap_add.push("NET_ADMIN".to_string());
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
