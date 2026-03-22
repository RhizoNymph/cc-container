use indexmap::IndexMap;
use minijinja::Environment;

use super::registry::ModuleRegistry;
use super::resolver::ModuleResolver;
use crate::config::project::{AgentType, ProjectConfig};
use crate::error::{Error, Result};

/// Generates a complete Dockerfile from project config and module templates.
pub struct DockerfileGenerator<'a> {
    registry: &'a ModuleRegistry,
}

impl<'a> DockerfileGenerator<'a> {
    pub fn new(registry: &'a ModuleRegistry) -> Self {
        Self { registry }
    }

    /// Generate a Dockerfile for the given agent type.
    /// When agent_type is Both, call this twice with Claude and Codex separately.
    pub fn generate(
        &self,
        config: &ProjectConfig,
        agent_type: AgentType,
    ) -> Result<String> {
        let mut modules = config.modules.clone();

        // Determine which base module to add based on image.base
        let base_name = config.image.base.to_string();
        if !modules.contains_key(&base_name) {
            let mut base_params = toml::map::Map::new();
            base_params.insert(
                "version".to_string(),
                toml::Value::String(config.image.base_version.clone()),
            );
            modules.insert(base_name.clone(), toml::Value::Table(base_params));
        }

        // Add user-setup module
        if !modules.contains_key("user-setup") {
            let mut params = toml::map::Map::new();
            params.insert(
                "username".to_string(),
                toml::Value::String(config.image.user.clone()),
            );
            params.insert(
                "shell".to_string(),
                toml::Value::String(format!("/bin/{}", config.image.shell)),
            );
            modules.insert("user-setup".to_string(), toml::Value::Table(params));
        }

        // Add agent module
        match agent_type {
            AgentType::Claude => {
                if !modules.contains_key("claude-code") {
                    let mut params = toml::map::Map::new();
                    params.insert(
                        "version".to_string(),
                        toml::Value::String(config.agent.claude_version.clone()),
                    );
                    modules.insert("claude-code".to_string(), toml::Value::Table(params));
                }
            }
            AgentType::Codex => {
                if !modules.contains_key("codex-cli") {
                    let mut params = toml::map::Map::new();
                    params.insert(
                        "version".to_string(),
                        toml::Value::String(config.agent.codex_version.clone()),
                    );
                    modules.insert("codex-cli".to_string(), toml::Value::Table(params));
                }
            }
            AgentType::Both => {
                // Should not be called with Both — caller must split
                return Err(Error::Other(
                    "generate() must be called with Claude or Codex, not Both".to_string(),
                ));
            }
        }

        // Add firewall module if enabled
        if config.firewall.enabled && !modules.contains_key("firewall") {
            modules.insert(
                "firewall".to_string(),
                toml::Value::Table(toml::map::Map::new()),
            );
        }

        // Resolve ordering
        let resolver = ModuleResolver::new(self.registry);
        let ordered = resolver.resolve(&modules)?;

        // Render each module template
        let base_os = config.image.base.to_string();
        let mut env = Environment::new();
        let mut dockerfile = String::new();

        // Add custom pre_agent snippet tracking
        let custom_pre = modules
            .get("custom")
            .and_then(|v| v.get("pre_agent"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let custom_post = modules
            .get("custom")
            .and_then(|v| v.get("post_agent"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut agent_rendered = false;

        for module_name in &ordered {
            if module_name == "custom" {
                continue; // handled via pre/post injection
            }

            let entry = self.registry.get(module_name).ok_or_else(|| {
                Error::ModuleNotFound(module_name.clone())
            })?;

            // Get user params for this module, merged with defaults
            let user_params = modules
                .get(module_name)
                .cloned()
                .unwrap_or(toml::Value::Table(toml::map::Map::new()));
            let params = merge_with_defaults(&entry.definition, &user_params);

            // Insert pre_agent custom snippet before agent modules
            let is_agent = entry.definition.module.category
                == super::definition::ModuleCategory::Agent;
            if is_agent && !agent_rendered {
                if let Some(ref pre) = custom_pre {
                    dockerfile.push_str(pre.trim());
                    dockerfile.push('\n');
                    dockerfile.push('\n');
                }
                agent_rendered = true;
            }

            // Render template
            let template_name = format!("{}.dockerfile.j2", module_name);
            env.add_template_owned(template_name.clone(), entry.template.clone())
                .map_err(|e| Error::TemplateRender(format!("{}: {}", module_name, e)))?;

            let tmpl = env
                .get_template(&template_name)
                .map_err(|e| Error::TemplateRender(format!("{}: {}", module_name, e)))?;

            let rendered = tmpl
                .render(minijinja::context! {
                    params => params,
                    base_os => &base_os,
                    image => {
                        minijinja::context! {
                            user => &config.image.user,
                            shell => config.image.shell.to_string(),
                        }
                    },
                })
                .map_err(|e| Error::TemplateRender(format!("{}: {}", module_name, e)))?;

            dockerfile.push_str(&rendered);
            dockerfile.push('\n');
        }

        // Add custom post_agent snippet
        if let Some(ref post) = custom_post {
            dockerfile.push('\n');
            dockerfile.push_str(post.trim());
            dockerfile.push('\n');
        }

        // Switch to non-root user and set workdir
        dockerfile.push('\n');
        dockerfile.push_str(&format!("USER {}\n", config.image.user));
        dockerfile.push_str(&format!("WORKDIR {}\n", config.workspace.mount_path));

        Ok(dockerfile)
    }
}

/// Merge user-provided params with module defaults.
fn merge_with_defaults(
    definition: &super::definition::ModuleDefinition,
    user_params: &toml::Value,
) -> toml::Value {
    let mut result = toml::map::Map::new();

    // Start with defaults
    for (key, param_def) in &definition.module.parameters {
        if let Some(ref default) = param_def.default {
            result.insert(key.clone(), default.clone());
        }
    }

    // Override with user params
    if let toml::Value::Table(user_table) = user_params {
        for (key, value) in user_table {
            result.insert(key.clone(), value.clone());
        }
    }

    toml::Value::Table(result)
}
