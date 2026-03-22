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

    let config = crate::config::load_project_config(&config_path)?;

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
                // Co-generate files the Dockerfile COPYs
                if config.firewall.enabled {
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
    let content = crate::mcp::config::generate_mcp_json(config);
    write_output(output_dir, ".mcp.json", &content, dry_run)
}
