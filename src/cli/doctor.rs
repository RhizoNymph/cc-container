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
