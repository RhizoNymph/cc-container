use std::path::{Path, PathBuf};

use crate::config::ProjectConfig;
use crate::config::project::AgentType;
use crate::config::validate::ValidationWarning;
use crate::error::Result;
use crate::module::{DockerfileGenerator, ModuleRegistry};

/// A generation target supported by the public library facade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum GenerateTarget {
    Dockerfile,
    Compose,
    Firewall,
    Env,
    Mcp,
    Helm,
}

/// Options for in-memory project generation.
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    /// Targets to generate, in the order they should be emitted.
    pub targets: Vec<GenerateTarget>,
    /// Whether to validate the config before generation.
    pub validate: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            targets: default_targets(),
            validate: true,
        }
    }
}

impl GenerateOptions {
    /// Build options for an explicit target list.
    pub fn targets(targets: impl Into<Vec<GenerateTarget>>) -> Self {
        Self {
            targets: targets.into(),
            ..Self::default()
        }
    }
}

/// The CLI-compatible default target set. Helm is intentionally opt-in.
pub fn default_targets() -> Vec<GenerateTarget> {
    vec![
        GenerateTarget::Dockerfile,
        GenerateTarget::Compose,
        GenerateTarget::Env,
        GenerateTarget::Firewall,
        GenerateTarget::Mcp,
    ]
}

/// One generated UTF-8 file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    /// Path relative to the selected output directory.
    pub path: PathBuf,
    /// Target responsible for producing this file.
    pub target: GenerateTarget,
    /// Complete file contents.
    pub contents: String,
}

/// Result of generating a project.
#[derive(Debug, Clone)]
pub struct GeneratedProject {
    /// Ordered generated files.
    pub files: Vec<GeneratedFile>,
    /// Non-fatal config validation warnings.
    pub warnings: Vec<ValidationWarning>,
}

/// Load an effective project config from disk and generate files in memory.
///
/// This applies the same user-default merge behavior as the CLI.
pub fn generate_from_path(
    config_path: &Path,
    options: GenerateOptions,
) -> Result<GeneratedProject> {
    let config = crate::config::load_effective_config(config_path)?;
    generate(&config, options)
}

/// Generate all requested files in memory from a project config.
///
/// This function does not read from or write to the filesystem.
pub fn generate(config: &ProjectConfig, options: GenerateOptions) -> Result<GeneratedProject> {
    let warnings = if options.validate {
        crate::config::validate::validate_config(config)?
    } else {
        Vec::new()
    };

    let mut files = Vec::new();

    for target in &options.targets {
        match target {
            GenerateTarget::Dockerfile => {
                files.extend(generate_dockerfiles(config)?);

                let firewall_already_targeted = options
                    .targets
                    .iter()
                    .any(|t| matches!(t, GenerateTarget::Firewall));
                if config.firewall.enabled && !firewall_already_targeted {
                    files.push(generate_firewall(config));
                }
            }
            GenerateTarget::Compose => {
                files.push(generate_compose(config)?);
            }
            GenerateTarget::Env => {
                files.push(generate_env(config));
            }
            GenerateTarget::Firewall => {
                if config.firewall.enabled {
                    files.push(generate_firewall(config));
                }
            }
            GenerateTarget::Mcp => {
                if !config.mcp.is_empty() {
                    files.push(generate_mcp(config)?);
                }
            }
            GenerateTarget::Helm => {
                files.extend(generate_helm(config)?);
            }
        }
    }

    Ok(GeneratedProject { files, warnings })
}

/// Generate files and write them to an output directory.
pub fn generate_to_dir(
    config: &ProjectConfig,
    options: GenerateOptions,
    output_dir: &Path,
) -> Result<GeneratedProject> {
    let project = generate(config, options)?;
    write_generated(&project, output_dir)?;
    Ok(project)
}

/// Write generated files to an output directory, creating parent directories.
pub fn write_generated(project: &GeneratedProject, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;
    for file in &project.files {
        write_file(output_dir, &file.path, &file.contents)?;
    }
    Ok(())
}

fn generate_dockerfiles(config: &ProjectConfig) -> Result<Vec<GeneratedFile>> {
    let registry = ModuleRegistry::new();
    let generator = DockerfileGenerator::new(&registry);

    let mut files = Vec::new();
    match config.agent.agent_type {
        AgentType::Both => {
            for (agent, filename) in [
                (AgentType::Claude, "Dockerfile.claude"),
                (AgentType::Codex, "Dockerfile.codex"),
            ] {
                files.push(GeneratedFile {
                    path: PathBuf::from(filename),
                    target: GenerateTarget::Dockerfile,
                    contents: generator.generate(config, agent)?,
                });
            }
        }
        agent_type => {
            files.push(GeneratedFile {
                path: PathBuf::from("Dockerfile"),
                target: GenerateTarget::Dockerfile,
                contents: generator.generate(config, agent_type)?,
            });
        }
    }

    Ok(files)
}

fn generate_compose(config: &ProjectConfig) -> Result<GeneratedFile> {
    let compose = crate::compose::generator::generate(config)?;
    let contents = serde_yaml::to_string(&compose).map_err(crate::error::Error::YamlSerialize)?;
    Ok(GeneratedFile {
        path: PathBuf::from("docker-compose.yml"),
        target: GenerateTarget::Compose,
        contents,
    })
}

fn generate_env(config: &ProjectConfig) -> GeneratedFile {
    GeneratedFile {
        path: PathBuf::from(".env.example"),
        target: GenerateTarget::Env,
        contents: crate::compose::env::generate_env_example(config),
    }
}

fn generate_firewall(config: &ProjectConfig) -> GeneratedFile {
    GeneratedFile {
        path: PathBuf::from("init-firewall.sh"),
        target: GenerateTarget::Firewall,
        contents: crate::firewall::generator::generate(config),
    }
}

fn generate_mcp(config: &ProjectConfig) -> Result<GeneratedFile> {
    Ok(GeneratedFile {
        path: PathBuf::from(".mcp.json"),
        target: GenerateTarget::Mcp,
        contents: crate::mcp::config::generate_mcp_json(config)?,
    })
}

fn generate_helm(config: &ProjectConfig) -> Result<Vec<GeneratedFile>> {
    let chart = crate::helm::chart::generate(config)?;
    let chart_root = PathBuf::from("chart").join(&config.project.name);

    Ok(chart
        .files
        .into_iter()
        .map(|(path, contents)| GeneratedFile {
            path: chart_root.join(path),
            target: GenerateTarget::Helm,
            contents,
        })
        .collect())
}

fn write_file(output_dir: &Path, relative_path: &Path, contents: &str) -> Result<()> {
    let path = output_dir.join(relative_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    Ok(())
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
            helm: HelmConfig::default(),
        }
    }

    fn paths(project: &GeneratedProject) -> Vec<PathBuf> {
        project.files.iter().map(|f| f.path.clone()).collect()
    }

    #[test]
    fn generate_default_targets_excludes_helm() {
        let config = minimal_config();
        let project = generate(&config, GenerateOptions::default()).unwrap();

        let paths = paths(&project);
        assert!(paths.contains(&PathBuf::from("Dockerfile")));
        assert!(paths.contains(&PathBuf::from("docker-compose.yml")));
        assert!(paths.contains(&PathBuf::from(".env.example")));
        assert!(!paths.iter().any(|p| p.starts_with("chart")));
    }

    #[test]
    fn generate_both_agent_outputs_two_dockerfiles() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::ApiKey,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });

        let project = generate(
            &config,
            GenerateOptions::targets(vec![GenerateTarget::Dockerfile]),
        )
        .unwrap();

        assert_eq!(
            paths(&project),
            vec![
                PathBuf::from("Dockerfile.claude"),
                PathBuf::from("Dockerfile.codex")
            ]
        );
    }

    #[test]
    fn generate_firewall_skips_disabled_firewall() {
        let config = minimal_config();
        let project = generate(
            &config,
            GenerateOptions::targets(vec![GenerateTarget::Firewall]),
        )
        .unwrap();

        assert!(project.files.is_empty());
    }

    #[test]
    fn generate_dockerfile_co_generates_enabled_firewall() {
        let mut config = minimal_config();
        config.firewall.enabled = true;
        config.runtime.cap_add.push("NET_ADMIN".to_string());

        let project = generate(
            &config,
            GenerateOptions::targets(vec![GenerateTarget::Dockerfile]),
        )
        .unwrap();

        assert_eq!(
            paths(&project),
            vec![
                PathBuf::from("Dockerfile"),
                PathBuf::from("init-firewall.sh")
            ]
        );
    }

    #[test]
    fn generate_mcp_skips_empty_mcp() {
        let config = minimal_config();
        let project =
            generate(&config, GenerateOptions::targets(vec![GenerateTarget::Mcp])).unwrap();

        assert!(project.files.is_empty());
    }

    #[test]
    fn generate_helm_outputs_chart_files() {
        let config = minimal_config();
        let project = generate(
            &config,
            GenerateOptions::targets(vec![GenerateTarget::Helm]),
        )
        .unwrap();

        let paths = paths(&project);
        assert!(paths.contains(&PathBuf::from("chart/test-project/Chart.yaml")));
        assert!(paths.contains(&PathBuf::from("chart/test-project/values.yaml")));
        assert!(
            paths
                .iter()
                .any(|p| p == &PathBuf::from("chart/test-project/templates/_helpers.tpl"))
        );
    }

    #[test]
    fn write_generated_creates_files_and_parents() {
        let project = GeneratedProject {
            warnings: Vec::new(),
            files: vec![GeneratedFile {
                path: PathBuf::from("nested/test.txt"),
                target: GenerateTarget::Env,
                contents: "hello".to_string(),
            }],
        };
        let dir = tempfile::tempdir().unwrap();

        write_generated(&project, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("nested/test.txt")).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn generate_to_dir_returns_project_and_writes_files() {
        let config = minimal_config();
        let dir = tempfile::tempdir().unwrap();

        let project = generate_to_dir(
            &config,
            GenerateOptions::targets(vec![GenerateTarget::Env]),
            dir.path(),
        )
        .unwrap();

        assert_eq!(project.files.len(), 1);
        assert!(dir.path().join(".env.example").exists());
    }
}
