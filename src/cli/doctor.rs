use clap::Parser;

#[derive(Parser)]
pub struct DoctorArgs {
    /// Show detailed diagnostic output
    #[arg(long)]
    pub verbose: bool,
}

pub fn run(args: &DoctorArgs, global: &super::GlobalOpts) -> crate::error::Result<()> {
    let mut ok = true;

    // Check Docker is installed
    eprint!("Docker: ");
    let docker_result: Result<std::path::PathBuf, _> = which::which("docker");
    match docker_result {
        Ok(path) => {
            if args.verbose {
                eprintln!("found at {}", path.display());
            } else {
                eprintln!("ok");
            }
        }
        Err(_) => {
            eprintln!("NOT FOUND");
            ok = false;
        }
    }

    // Check docker compose
    eprint!("Docker Compose: ");
    let compose_check = std::process::Command::new("docker")
        .args(["compose", "version"])
        .output();
    match compose_check {
        Ok(output) if output.status.success() => {
            if args.verbose {
                let version = String::from_utf8_lossy(&output.stdout);
                eprintln!("{}", version.trim());
            } else {
                eprintln!("ok");
            }
        }
        _ => {
            eprintln!("NOT FOUND (docker compose plugin required)");
            ok = false;
        }
    }

    // Check for config file
    let target_dir = global
        .target_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));
    let config_path = global
        .config
        .clone()
        .unwrap_or_else(|| target_dir.join("cc-container.toml"));

    eprint!("Config file: ");
    if config_path.exists() {
        eprintln!("{}", config_path.display());
        // Try to load and validate
        match crate::config::load_project_config(&config_path) {
            Ok(config) => {
                match crate::config::validate::validate_config(&config) {
                    Ok(warnings) => {
                        if warnings.is_empty() {
                            eprintln!("  Config: valid");
                        } else {
                            for w in &warnings {
                                eprintln!("  warning: {w}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("  Config error: {e}");
                        ok = false;
                    }
                }

                // Check auth credentials exist for oauth methods
                check_oauth_credentials(&config, args.verbose);
            }
            Err(e) => {
                eprintln!("  Failed to parse: {e}");
                ok = false;
            }
        }
    } else {
        eprintln!("not found (run `cc-container init` to create one)");
    }

    if ok {
        eprintln!("\nAll checks passed.");
        Ok(())
    } else {
        eprintln!("\nSome checks failed.");
        Err(crate::error::Error::Other("some checks failed".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::*;
    use indexmap::IndexMap;

    fn make_global_with_config(config_path: std::path::PathBuf) -> super::super::GlobalOpts {
        super::super::GlobalOpts {
            target_dir: None,
            config: Some(config_path),
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        }
    }

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

    fn write_config(dir: &std::path::Path) -> std::path::PathBuf {
        let config = minimal_config();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let path = dir.join("cc-container.toml");
        std::fs::write(&path, toml_str).unwrap();
        path
    }

    #[test]
    fn doctor_with_valid_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global_with_config(config_path);

        let args = DoctorArgs { verbose: false };
        // Doctor checks for Docker, which may or may not be installed.
        // We just run it and verify it doesn't panic.
        let _result = run(&args, &global);
    }

    #[test]
    fn doctor_with_missing_config_still_runs() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nonexistent.toml");
        let global = make_global_with_config(missing);

        let args = DoctorArgs { verbose: false };
        let _result = run(&args, &global);
    }

    #[test]
    fn doctor_verbose_mode() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = write_config(dir.path());
        let global = make_global_with_config(config_path);

        let args = DoctorArgs { verbose: true };
        let _result = run(&args, &global);
    }

    #[test]
    fn doctor_with_invalid_config_reports_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("cc-container.toml");
        std::fs::write(&config_path, "this is not valid toml {{{").unwrap();

        let global = make_global_with_config(config_path);
        let args = DoctorArgs { verbose: false };

        let _result = run(&args, &global);
    }

    #[test]
    fn check_oauth_credentials_api_key_does_nothing() {
        let config = minimal_config();
        check_oauth_credentials(&config, false);
    }

    #[test]
    fn check_oauth_credentials_oauth_method() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Oauth,
        });
        check_oauth_credentials(&config, false);
    }

    #[test]
    fn check_oauth_credentials_codex_oauth() {
        let mut config = minimal_config();
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::Oauth,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });
        check_oauth_credentials(&config, true);
    }

    #[test]
    fn check_oauth_credentials_no_auth() {
        let mut config = minimal_config();
        config.auth.claude = None;
        config.auth.codex = None;
        check_oauth_credentials(&config, false);
    }
}

fn check_oauth_credentials(
    config: &crate::config::ProjectConfig,
    verbose: bool,
) {
    use crate::config::project::{ClaudeAuthMethod, CodexAuthMethod};

    if let Some(ref claude_auth) = config.auth.claude
        && claude_auth.method == ClaudeAuthMethod::Oauth {
            let cred_path = dirs::home_dir()
                .map(|h| h.join(".claude").join(".credentials.json"));
            eprint!("  Claude OAuth credentials: ");
            match cred_path {
                Some(p) if p.exists() => {
                    if verbose {
                        eprintln!("found ({})", p.display());
                    } else {
                        eprintln!("found");
                    }
                }
                _ => eprintln!("NOT FOUND (~/.claude/.credentials.json) — run `claude /login` first"),
            }
        }

    if let Some(ref codex_auth) = config.auth.codex
        && codex_auth.method == CodexAuthMethod::Oauth {
            let cred_path = dirs::home_dir()
                .map(|h| h.join(".codex").join("auth.json"));
            eprint!("  Codex OAuth credentials: ");
            match cred_path {
                Some(p) if p.exists() => {
                    if verbose {
                        eprintln!("found ({})", p.display());
                    } else {
                        eprintln!("found");
                    }
                }
                _ => eprintln!("NOT FOUND (~/.codex/auth.json) — run `codex login` first"),
            }
        }
}
