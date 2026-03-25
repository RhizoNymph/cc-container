use minijinja::Environment;

use super::registry::ModuleRegistry;
use super::resolver::ModuleResolver;
use crate::config::project::{AgentType, ProjectConfig, default_version_for_os};
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
    pub fn generate(&self, config: &ProjectConfig, agent_type: AgentType) -> Result<String> {
        let mut modules = config.modules.clone();

        // Determine which base module to add based on image.base
        let base_name = config.image.base.to_string();
        if !modules.contains_key(&base_name) {
            let mut base_params = toml::map::Map::new();
            let version = config
                .image
                .base_version
                .clone()
                .unwrap_or_else(|| default_version_for_os(config.image.base).to_string());
            base_params.insert("version".to_string(), toml::Value::String(version));
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
                toml::Value::String(config.image.shell.to_string()),
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

        // Extract custom pre/post agent snippets before resolving
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

        if modules.contains_key("custom") && custom_pre.is_none() && custom_post.is_none() {
            return Err(Error::Other(
                "[modules.custom] is present but has no 'pre_agent' or 'post_agent' field. \
                 Add at least one to include custom Dockerfile instructions."
                    .to_string(),
            ));
        }

        // Remove custom from modules before resolving (it's not a real module)
        modules.shift_remove("custom");

        // Resolve ordering
        let resolver = ModuleResolver::new(self.registry);
        let ordered = resolver.resolve(&modules)?;

        // Render each module template
        let base_os = config.image.base.to_string();
        let mut env = Environment::new();
        let mut dockerfile = String::new();

        let mut agent_rendered = false;

        for module_name in &ordered {
            if module_name == "custom" {
                continue; // handled via pre/post injection
            }

            let entry = self
                .registry
                .get(module_name)
                .ok_or_else(|| Error::ModuleNotFound(module_name.clone()))?;

            // Get user params for this module, merged with defaults
            let user_params = modules
                .get(module_name)
                .cloned()
                .unwrap_or(toml::Value::Table(toml::map::Map::new()));
            let params = merge_with_defaults(module_name, &entry.definition, &user_params)?;

            // Insert pre_agent custom snippet before agent modules
            let is_agent =
                entry.definition.module.category == super::definition::ModuleCategory::Agent;
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

/// Merge user-provided params with module defaults and validate types/allowed values.
fn merge_with_defaults(
    module_name: &str,
    definition: &super::definition::ModuleDefinition,
    user_params: &toml::Value,
) -> Result<toml::Value> {
    use super::definition::ParamType;

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

    // Validate each parameter that has a definition
    for (key, param_def) in &definition.module.parameters {
        if let Some(value) = result.get(key) {
            // Check type matches declared param_type
            let type_ok = match param_def.param_type {
                ParamType::String => value.is_str(),
                ParamType::Bool => matches!(value, toml::Value::Boolean(_)),
                ParamType::Int => value.is_integer(),
                ParamType::List => value.is_array(),
            };
            if !type_ok {
                return Err(Error::InvalidParameter {
                    module: module_name.to_string(),
                    param: key.clone(),
                    reason: format!("expected type {:?}, got {:?}", param_def.param_type, value),
                });
            }

            // Check allowed_values constraint
            if let Some(ref allowed) = param_def.allowed_values
                && !allowed.contains(value)
            {
                return Err(Error::InvalidParameter {
                    module: module_name.to_string(),
                    param: key.clone(),
                    reason: format!("value {:?} is not in allowed values: {:?}", value, allowed),
                });
            }
        }
    }

    Ok(toml::Value::Table(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::*;
    use crate::module::registry::ModuleRegistry;
    use indexmap::IndexMap;

    /// Helper to create a minimal ProjectConfig for testing.
    fn minimal_config(agent_type: AgentType) -> ProjectConfig {
        ProjectConfig {
            project: ProjectMeta {
                name: "test-project".to_string(),
                description: None,
            },
            agent: AgentConfig {
                agent_type,
                claude_version: "latest".to_string(),
                codex_version: "latest".to_string(),
            },
            image: ImageConfig {
                base: BaseOs::Ubuntu,
                base_version: None,
                platform: "linux/amd64".to_string(),
                tag: None,
                user: "dev".to_string(),
                shell: ShellType::Bash,
            },
            modules: IndexMap::new(),
            auth: AuthConfig::default(),
            firewall: FirewallConfig::default(),
            workspace: WorkspaceConfig::default(),
            volumes: IndexMap::new(),
            environment: EnvironmentConfig::default(),
            services: IndexMap::new(),
            mcp: IndexMap::new(),
            runtime: RuntimeConfig::default(),
        }
    }

    #[test]
    fn test_generate_claude_dockerfile() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let config = minimal_config(AgentType::Claude);
        let result = generator.generate(&config, AgentType::Claude);
        assert!(result.is_ok(), "Generation failed: {:?}", result.err());

        let dockerfile = result.unwrap();

        // Should contain FROM ubuntu
        assert!(
            dockerfile.contains("FROM ubuntu:"),
            "Should contain FROM ubuntu"
        );

        // Should contain Node.js installation (auto-added via claude-code dependency)
        assert!(
            dockerfile.contains("nodejs") || dockerfile.contains("Node.js"),
            "Should contain Node.js setup"
        );

        // Should contain Claude Code installation
        assert!(
            dockerfile.contains("claude-code"),
            "Should contain claude-code installation"
        );

        // Should contain user setup
        assert!(dockerfile.contains("dev"), "Should contain user dev setup");

        // Should end with USER and WORKDIR
        assert!(dockerfile.contains("USER dev"), "Should set USER to dev");
        assert!(
            dockerfile.contains("WORKDIR /workspace"),
            "Should set WORKDIR to /workspace"
        );
    }

    #[test]
    fn test_generate_codex_dockerfile() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let config = minimal_config(AgentType::Codex);
        let result = generator.generate(&config, AgentType::Codex);
        assert!(result.is_ok(), "Generation failed: {:?}", result.err());

        let dockerfile = result.unwrap();

        assert!(
            dockerfile.contains("FROM ubuntu:"),
            "Should contain FROM ubuntu"
        );
        assert!(
            dockerfile.contains("codex") || dockerfile.contains("Codex"),
            "Should contain codex CLI setup"
        );
    }

    #[test]
    fn test_generate_both_agent_type_errors() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let config = minimal_config(AgentType::Both);
        let result = generator.generate(&config, AgentType::Both);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not Both"),
            "Error should mention Both: {}",
            err_msg
        );
    }

    #[test]
    fn test_generate_with_debian_base() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.image.base = BaseOs::Debian;

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("FROM debian:"),
            "Should contain FROM debian"
        );
    }

    #[test]
    fn test_generate_with_alpine_base() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.image.base = BaseOs::Alpine;

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("FROM alpine:"),
            "Should contain FROM alpine"
        );
    }

    #[test]
    fn test_generate_with_custom_base_version() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.image.base_version = Some("22.04".to_string());

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("FROM ubuntu:22.04"),
            "Should use custom base version 22.04"
        );
    }

    #[test]
    fn test_generate_default_base_version() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let config = minimal_config(AgentType::Claude);
        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("FROM ubuntu:24.04"),
            "Should use default ubuntu version 24.04"
        );
    }

    #[test]
    fn test_generate_with_extra_modules() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        // Add python and git modules
        let mut python_params = toml::map::Map::new();
        python_params.insert(
            "version".to_string(),
            toml::Value::String("3.11".to_string()),
        );
        config
            .modules
            .insert("python".to_string(), toml::Value::Table(python_params));
        config
            .modules
            .insert("git".to_string(), toml::Value::Table(toml::map::Map::new()));

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("Git") || dockerfile.contains("git"),
            "Should contain git setup"
        );
        assert!(
            dockerfile.contains("Python") || dockerfile.contains("python"),
            "Should contain python setup"
        );
    }

    #[test]
    fn test_generate_with_firewall_enabled() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.firewall.enabled = true;

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("iptables") || dockerfile.contains("firewall"),
            "Should contain firewall setup when enabled"
        );
    }

    #[test]
    fn test_generate_firewall_not_included_when_disabled() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let config = minimal_config(AgentType::Claude);
        // firewall.enabled defaults to false

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        // The firewall module should not be present
        assert!(
            !dockerfile.contains("iptables"),
            "Should not contain iptables when firewall is disabled"
        );
    }

    #[test]
    fn test_generate_custom_user() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.image.user = "myuser".to_string();

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("USER myuser"),
            "Should set USER to myuser"
        );
        assert!(
            dockerfile.contains("myuser"),
            "Should reference the custom user"
        );
    }

    #[test]
    fn test_generate_custom_workspace() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.workspace.mount_path = "/home/dev/project".to_string();

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("WORKDIR /home/dev/project"),
            "Should set WORKDIR to custom path"
        );
    }

    #[test]
    fn test_generate_parameter_substitution() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.image.base_version = Some("22.04".to_string());
        config.agent.claude_version = "1.0.5".to_string();

        // Specify a node version
        let mut node_params = toml::map::Map::new();
        node_params.insert("version".to_string(), toml::Value::String("20".to_string()));
        config
            .modules
            .insert("node".to_string(), toml::Value::Table(node_params));

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("22.04"),
            "Should contain ubuntu version 22.04"
        );
        assert!(
            dockerfile.contains("setup_20"),
            "Should contain node version 20 in setup script URL"
        );
        assert!(
            dockerfile.contains("claude-code@1.0.5"),
            "Should contain specific claude-code version"
        );
    }

    #[test]
    fn test_generate_uses_default_params_when_none_specified() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        // Don't specify any node params - should use defaults
        let config = minimal_config(AgentType::Claude);
        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();

        // Default node version is 22
        assert!(
            dockerfile.contains("setup_22"),
            "Should use default node version 22"
        );
    }

    #[test]
    fn test_merge_with_defaults_uses_defaults() {
        let toml_str = r#"
[module]
name = "test"
category = "tool"
description = "test"

[module.parameters]
version = { type = "string", default = "1.0", description = "Version" }
flag = { type = "bool", default = true, description = "Flag" }
"#;
        let def: super::super::definition::ModuleDefinition = toml::from_str(toml_str).unwrap();
        let user_params = toml::Value::Table(toml::map::Map::new());

        let result = merge_with_defaults("test", &def, &user_params).unwrap();
        let table = result.as_table().unwrap();

        assert_eq!(
            table.get("version"),
            Some(&toml::Value::String("1.0".to_string()))
        );
        assert_eq!(table.get("flag"), Some(&toml::Value::Boolean(true)));
    }

    #[test]
    fn test_merge_with_defaults_user_overrides() {
        let toml_str = r#"
[module]
name = "test"
category = "tool"
description = "test"

[module.parameters]
version = { type = "string", default = "1.0", description = "Version" }
flag = { type = "bool", default = true, description = "Flag" }
"#;
        let def: super::super::definition::ModuleDefinition = toml::from_str(toml_str).unwrap();

        let mut user_table = toml::map::Map::new();
        user_table.insert(
            "version".to_string(),
            toml::Value::String("2.0".to_string()),
        );
        let user_params = toml::Value::Table(user_table);

        let result = merge_with_defaults("test", &def, &user_params).unwrap();
        let table = result.as_table().unwrap();

        // version overridden
        assert_eq!(
            table.get("version"),
            Some(&toml::Value::String("2.0".to_string()))
        );
        // flag keeps default
        assert_eq!(table.get("flag"), Some(&toml::Value::Boolean(true)));
    }

    #[test]
    fn test_merge_with_defaults_extra_user_params() {
        let toml_str = r#"
[module]
name = "test"
category = "tool"
description = "test"

[module.parameters]
version = { type = "string", default = "1.0", description = "Version" }
"#;
        let def: super::super::definition::ModuleDefinition = toml::from_str(toml_str).unwrap();

        let mut user_table = toml::map::Map::new();
        user_table.insert(
            "extra_key".to_string(),
            toml::Value::String("extra_val".to_string()),
        );
        let user_params = toml::Value::Table(user_table);

        let result = merge_with_defaults("test", &def, &user_params).unwrap();
        let table = result.as_table().unwrap();

        assert_eq!(
            table.get("version"),
            Some(&toml::Value::String("1.0".to_string()))
        );
        assert_eq!(
            table.get("extra_key"),
            Some(&toml::Value::String("extra_val".to_string()))
        );
    }

    #[test]
    fn test_merge_with_defaults_no_defaults_no_user() {
        let toml_str = r#"
[module]
name = "test"
category = "tool"
description = "test"

[module.parameters]
name = { type = "string", description = "No default" }
"#;
        let def: super::super::definition::ModuleDefinition = toml::from_str(toml_str).unwrap();
        let user_params = toml::Value::Table(toml::map::Map::new());

        let result = merge_with_defaults("test", &def, &user_params).unwrap();
        let table = result.as_table().unwrap();

        // No default, no user override => not present
        assert!(table.get("name").is_none());
    }

    #[test]
    fn test_generate_ordering_base_before_langs() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        let mut python_params = toml::map::Map::new();
        python_params.insert(
            "version".to_string(),
            toml::Value::String("3.12".to_string()),
        );
        config
            .modules
            .insert("python".to_string(), toml::Value::Table(python_params));

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();

        // FROM should appear before any RUN commands for language installs
        let from_pos = dockerfile.find("FROM ubuntu:").unwrap();
        let python_pos = dockerfile
            .find("Python")
            .or(dockerfile.find("python"))
            .unwrap();
        assert!(
            from_pos < python_pos,
            "FROM should come before Python installation"
        );
    }

    #[test]
    fn test_generate_with_custom_shell() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.image.shell = ShellType::Zsh;

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("command -v zsh"),
            "Should use command -v to resolve zsh path"
        );
    }

    // --- Bug 5: Parameter type validation tests ---

    #[test]
    fn test_param_type_mismatch_string_for_bool() {
        let toml_str = r#"
[module]
name = "test-mod"
category = "tool"
description = "test"

[module.parameters]
flag = { type = "bool", default = true, description = "A flag" }
"#;
        let def: super::super::definition::ModuleDefinition = toml::from_str(toml_str).unwrap();

        let mut user_table = toml::map::Map::new();
        user_table.insert(
            "flag".to_string(),
            toml::Value::String("not-a-bool".to_string()),
        );
        let user_params = toml::Value::Table(user_table);

        let result = merge_with_defaults("test-mod", &def, &user_params);
        assert!(
            result.is_err(),
            "Should fail when string provided for bool param"
        );
        let err = result.unwrap_err();
        match &err {
            Error::InvalidParameter {
                module,
                param,
                reason,
            } => {
                assert_eq!(module, "test-mod");
                assert_eq!(param, "flag");
                assert!(
                    reason.contains("bool"),
                    "Reason should mention expected type: {}",
                    reason
                );
            }
            other => panic!("Expected InvalidParameter, got: {:?}", other),
        }
    }

    #[test]
    fn test_param_allowed_values_violation() {
        let toml_str = r#"
[module]
name = "test-mod"
category = "lang"
description = "test"

[module.parameters]
version = { type = "string", default = "22", description = "Version", allowed_values = ["18", "20", "22"] }
"#;
        let def: super::super::definition::ModuleDefinition = toml::from_str(toml_str).unwrap();

        let mut user_table = toml::map::Map::new();
        user_table.insert("version".to_string(), toml::Value::String("99".to_string()));
        let user_params = toml::Value::Table(user_table);

        let result = merge_with_defaults("test-mod", &def, &user_params);
        assert!(
            result.is_err(),
            "Should fail when value not in allowed_values"
        );
        let err = result.unwrap_err();
        match &err {
            Error::InvalidParameter {
                module,
                param,
                reason,
            } => {
                assert_eq!(module, "test-mod");
                assert_eq!(param, "version");
                assert!(
                    reason.contains("allowed"),
                    "Reason should mention allowed values: {}",
                    reason
                );
            }
            other => panic!("Expected InvalidParameter, got: {:?}", other),
        }
    }

    #[test]
    fn test_valid_params_pass_validation() {
        let toml_str = r#"
[module]
name = "test-mod"
category = "lang"
description = "test"

[module.parameters]
version = { type = "string", default = "22", description = "Version", allowed_values = ["18", "20", "22"] }
flag = { type = "bool", default = true, description = "Flag" }
count = { type = "int", default = 3, description = "Count" }
"#;
        let def: super::super::definition::ModuleDefinition = toml::from_str(toml_str).unwrap();

        let mut user_table = toml::map::Map::new();
        user_table.insert("version".to_string(), toml::Value::String("20".to_string()));
        user_table.insert("flag".to_string(), toml::Value::Boolean(false));
        user_table.insert("count".to_string(), toml::Value::Integer(5));
        let user_params = toml::Value::Table(user_table);

        let result = merge_with_defaults("test-mod", &def, &user_params);
        assert!(
            result.is_ok(),
            "Valid params should pass: {:?}",
            result.err()
        );
        let table = result.unwrap();
        let table = table.as_table().unwrap();
        assert_eq!(
            table.get("version"),
            Some(&toml::Value::String("20".to_string()))
        );
        assert_eq!(table.get("flag"), Some(&toml::Value::Boolean(false)));
        assert_eq!(table.get("count"), Some(&toml::Value::Integer(5)));
    }

    // --- Bug 6: Shell uses command -v ---

    #[test]
    fn test_shell_uses_command_v() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        config.image.shell = ShellType::Zsh;

        let dockerfile = generator.generate(&config, AgentType::Claude).unwrap();
        assert!(
            dockerfile.contains("command -v zsh"),
            "Should use 'command -v zsh' to resolve shell path, got:\n{}",
            dockerfile,
        );
    }

    // --- Bug 8: Empty custom module error tests ---

    #[test]
    fn test_custom_module_empty_returns_error() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        // Add empty [modules.custom] with no pre_agent or post_agent
        config.modules.insert(
            "custom".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );

        let result = generator.generate(&config, AgentType::Claude);
        assert!(
            result.is_err(),
            "Empty [modules.custom] should return error"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("pre_agent") && err_msg.contains("post_agent"),
            "Error should mention pre_agent and post_agent: {}",
            err_msg,
        );
    }

    #[test]
    fn test_custom_module_pre_agent_only_works() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        let mut custom_table = toml::map::Map::new();
        custom_table.insert(
            "pre_agent".to_string(),
            toml::Value::String("RUN echo pre".to_string()),
        );
        config
            .modules
            .insert("custom".to_string(), toml::Value::Table(custom_table));

        let result = generator.generate(&config, AgentType::Claude);
        assert!(
            result.is_ok(),
            "Custom with pre_agent should work: {:?}",
            result.err()
        );
        let dockerfile = result.unwrap();
        assert!(
            dockerfile.contains("RUN echo pre"),
            "Should contain pre_agent snippet",
        );
    }

    #[test]
    fn test_custom_module_post_agent_only_works() {
        let registry = ModuleRegistry::new();
        let generator = DockerfileGenerator::new(&registry);

        let mut config = minimal_config(AgentType::Claude);
        let mut custom_table = toml::map::Map::new();
        custom_table.insert(
            "post_agent".to_string(),
            toml::Value::String("RUN echo post".to_string()),
        );
        config
            .modules
            .insert("custom".to_string(), toml::Value::Table(custom_table));

        let result = generator.generate(&config, AgentType::Claude);
        assert!(
            result.is_ok(),
            "Custom with post_agent should work: {:?}",
            result.err()
        );
        let dockerfile = result.unwrap();
        assert!(
            dockerfile.contains("RUN echo post"),
            "Should contain post_agent snippet",
        );
    }
}
