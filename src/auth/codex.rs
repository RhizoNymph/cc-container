use super::{AuthRequirements, AuthVolume};
use crate::config::project::{CodexAuthConfig, CodexAuthMethod};
use indexmap::IndexMap;

/// Build auth requirements for Codex CLI based on the configured method.
pub fn requirements(auth: &CodexAuthConfig, container_user: &str) -> AuthRequirements {
    match auth.method {
        CodexAuthMethod::ApiKey => api_key_requirements(),
        CodexAuthMethod::Oauth => oauth_requirements(container_user),
        CodexAuthMethod::Azure => azure_requirements(auth),
        CodexAuthMethod::Custom => custom_requirements(auth),
    }
}

fn api_key_requirements() -> AuthRequirements {
    AuthRequirements {
        env_vars: IndexMap::from([(
            "OPENAI_API_KEY".to_string(),
            "${OPENAI_API_KEY}".to_string(),
        )]),
        volumes: vec![],
        env_example_lines: vec![
            "# Codex CLI - API Key auth".to_string(),
            "OPENAI_API_KEY=your-openai-api-key-here".to_string(),
        ],
    }
}

fn oauth_requirements(container_user: &str) -> AuthRequirements {
    AuthRequirements {
        env_vars: IndexMap::new(),
        volumes: vec![AuthVolume {
            source: "${HOME}/.codex/auth.json".to_string(),
            target: format!("/home/{}/.codex/auth.json", container_user),
            read_only: true,
        }],
        env_example_lines: vec![
            "# Codex CLI - OAuth auth".to_string(),
            "# No env vars needed. Credentials mounted from host ~/.codex/auth.json".to_string(),
            "# Run `codex login` on your host machine first.".to_string(),
        ],
    }
}

fn azure_requirements(auth: &CodexAuthConfig) -> AuthRequirements {
    let mut env_vars = IndexMap::from([(
        "AZURE_OPENAI_API_KEY".to_string(),
        "${AZURE_OPENAI_API_KEY}".to_string(),
    )]);

    let mut env_lines = vec![
        "# Codex CLI - Azure OpenAI auth".to_string(),
        "AZURE_OPENAI_API_KEY=your-azure-openai-api-key".to_string(),
    ];

    if let Some(ref endpoint) = auth.azure_endpoint {
        env_vars.insert(
            "AZURE_OPENAI_ENDPOINT".to_string(),
            endpoint.clone(),
        );
        env_lines.push(format!("# Azure endpoint: {}", endpoint));
    }

    AuthRequirements {
        env_vars,
        volumes: vec![],
        env_example_lines: env_lines,
    }
}

fn custom_requirements(auth: &CodexAuthConfig) -> AuthRequirements {
    let env_key = auth
        .custom_env_key
        .clone()
        .unwrap_or_else(|| "CUSTOM_API_KEY".to_string());

    let mut env_vars = IndexMap::from([(
        env_key.clone(),
        format!("${{{}}}", env_key),
    )]);

    if let Some(ref base_url) = auth.custom_base_url {
        env_vars.insert("OPENAI_BASE_URL".to_string(), base_url.clone());
    }

    let mut env_lines = vec![
        "# Codex CLI - Custom provider auth".to_string(),
        format!("{}=your-api-key-here", env_key),
    ];

    if let Some(ref base_url) = auth.custom_base_url {
        env_lines.push(format!("# Custom endpoint: {}", base_url));
    }

    AuthRequirements {
        env_vars,
        volumes: vec![],
        env_example_lines: env_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::{CodexAuthConfig, CodexAuthMethod};

    fn make_config(method: CodexAuthMethod) -> CodexAuthConfig {
        CodexAuthConfig {
            method,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        }
    }

    // ── API Key ──────────────────────────────────────────────────────

    #[test]
    fn api_key_sets_openai_api_key_env() {
        let cfg = make_config(CodexAuthMethod::ApiKey);
        let req = requirements(&cfg, "developer");

        assert_eq!(req.env_vars.len(), 1);
        assert_eq!(
            req.env_vars.get("OPENAI_API_KEY").unwrap(),
            "${OPENAI_API_KEY}"
        );
    }

    #[test]
    fn api_key_has_no_volumes() {
        let req = requirements(&make_config(CodexAuthMethod::ApiKey), "dev");
        assert!(req.volumes.is_empty());
    }

    #[test]
    fn api_key_env_example_mentions_openai() {
        let req = requirements(&make_config(CodexAuthMethod::ApiKey), "dev");
        assert!(req
            .env_example_lines
            .iter()
            .any(|l| l.contains("OPENAI_API_KEY")));
    }

    #[test]
    fn api_key_ignores_container_user() {
        let req_a = requirements(&make_config(CodexAuthMethod::ApiKey), "alice");
        let req_b = requirements(&make_config(CodexAuthMethod::ApiKey), "bob");
        assert_eq!(req_a.env_vars, req_b.env_vars);
    }

    // ── OAuth ────────────────────────────────────────────────────────

    #[test]
    fn oauth_has_no_env_vars() {
        let req = requirements(&make_config(CodexAuthMethod::Oauth), "developer");
        assert!(req.env_vars.is_empty());
    }

    #[test]
    fn oauth_mounts_auth_json() {
        let req = requirements(&make_config(CodexAuthMethod::Oauth), "developer");
        assert_eq!(req.volumes.len(), 1);

        let vol = &req.volumes[0];
        assert_eq!(vol.source, "${HOME}/.codex/auth.json");
        assert_eq!(vol.target, "/home/developer/.codex/auth.json");
        assert!(vol.read_only);
    }

    #[test]
    fn oauth_volume_target_uses_container_user() {
        let req = requirements(&make_config(CodexAuthMethod::Oauth), "charlie");
        assert_eq!(
            req.volumes[0].target,
            "/home/charlie/.codex/auth.json"
        );
    }

    #[test]
    fn oauth_env_example_mentions_no_env_vars() {
        let req = requirements(&make_config(CodexAuthMethod::Oauth), "dev");
        assert!(req
            .env_example_lines
            .iter()
            .any(|l| l.contains("No env vars needed")));
    }

    // ── Azure ────────────────────────────────────────────────────────

    #[test]
    fn azure_sets_azure_api_key() {
        let cfg = make_config(CodexAuthMethod::Azure);
        let req = requirements(&cfg, "dev");

        assert_eq!(
            req.env_vars.get("AZURE_OPENAI_API_KEY").unwrap(),
            "${AZURE_OPENAI_API_KEY}"
        );
    }

    #[test]
    fn azure_has_no_volumes() {
        let req = requirements(&make_config(CodexAuthMethod::Azure), "dev");
        assert!(req.volumes.is_empty());
    }

    #[test]
    fn azure_without_endpoint_has_one_env_var() {
        let cfg = make_config(CodexAuthMethod::Azure);
        let req = requirements(&cfg, "dev");
        assert_eq!(req.env_vars.len(), 1);
    }

    #[test]
    fn azure_with_endpoint_adds_env_var() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Azure,
            azure_endpoint: Some("https://my-resource.openai.azure.com".to_string()),
            custom_env_key: None,
            custom_base_url: None,
        };
        let req = requirements(&cfg, "dev");

        assert_eq!(req.env_vars.len(), 2);
        assert_eq!(
            req.env_vars.get("AZURE_OPENAI_ENDPOINT").unwrap(),
            "https://my-resource.openai.azure.com"
        );
    }

    #[test]
    fn azure_with_endpoint_includes_it_in_env_example() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Azure,
            azure_endpoint: Some("https://my-resource.openai.azure.com".to_string()),
            custom_env_key: None,
            custom_base_url: None,
        };
        let req = requirements(&cfg, "dev");
        assert!(req
            .env_example_lines
            .iter()
            .any(|l| l.contains("https://my-resource.openai.azure.com")));
    }

    #[test]
    fn azure_ignores_container_user() {
        let req_a = requirements(&make_config(CodexAuthMethod::Azure), "alice");
        let req_b = requirements(&make_config(CodexAuthMethod::Azure), "bob");
        assert_eq!(req_a.env_vars, req_b.env_vars);
    }

    #[test]
    fn azure_ignores_custom_fields() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Azure,
            azure_endpoint: None,
            custom_env_key: Some("MY_KEY".to_string()),
            custom_base_url: Some("https://custom.example.com".to_string()),
        };
        let req = requirements(&cfg, "dev");
        // custom_env_key and custom_base_url should not leak into Azure auth
        assert!(req.env_vars.get("MY_KEY").is_none());
        assert!(req.env_vars.get("OPENAI_BASE_URL").is_none());
    }

    // ── Custom ───────────────────────────────────────────────────────

    #[test]
    fn custom_defaults_env_key_to_custom_api_key() {
        let cfg = make_config(CodexAuthMethod::Custom);
        let req = requirements(&cfg, "dev");

        assert_eq!(req.env_vars.len(), 1);
        assert_eq!(
            req.env_vars.get("CUSTOM_API_KEY").unwrap(),
            "${CUSTOM_API_KEY}"
        );
    }

    #[test]
    fn custom_with_env_key_uses_it() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Custom,
            azure_endpoint: None,
            custom_env_key: Some("MY_PROVIDER_KEY".to_string()),
            custom_base_url: None,
        };
        let req = requirements(&cfg, "dev");

        assert_eq!(req.env_vars.len(), 1);
        assert_eq!(
            req.env_vars.get("MY_PROVIDER_KEY").unwrap(),
            "${MY_PROVIDER_KEY}"
        );
        // The default key should not be present
        assert!(req.env_vars.get("CUSTOM_API_KEY").is_none());
    }

    #[test]
    fn custom_without_base_url_has_no_openai_base_url() {
        let cfg = make_config(CodexAuthMethod::Custom);
        let req = requirements(&cfg, "dev");
        assert!(req.env_vars.get("OPENAI_BASE_URL").is_none());
    }

    #[test]
    fn custom_with_base_url_adds_openai_base_url() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Custom,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: Some("https://my-llm.example.com/v1".to_string()),
        };
        let req = requirements(&cfg, "dev");

        assert_eq!(req.env_vars.len(), 2);
        assert_eq!(
            req.env_vars.get("OPENAI_BASE_URL").unwrap(),
            "https://my-llm.example.com/v1"
        );
    }

    #[test]
    fn custom_with_all_fields_set() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Custom,
            azure_endpoint: None,
            custom_env_key: Some("OLLAMA_KEY".to_string()),
            custom_base_url: Some("http://localhost:11434/v1".to_string()),
        };
        let req = requirements(&cfg, "dev");

        assert_eq!(req.env_vars.len(), 2);
        assert_eq!(
            req.env_vars.get("OLLAMA_KEY").unwrap(),
            "${OLLAMA_KEY}"
        );
        assert_eq!(
            req.env_vars.get("OPENAI_BASE_URL").unwrap(),
            "http://localhost:11434/v1"
        );
    }

    #[test]
    fn custom_env_example_uses_custom_key_name() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Custom,
            azure_endpoint: None,
            custom_env_key: Some("TOGETHER_API_KEY".to_string()),
            custom_base_url: None,
        };
        let req = requirements(&cfg, "dev");
        assert!(req
            .env_example_lines
            .iter()
            .any(|l| l.contains("TOGETHER_API_KEY")));
    }

    #[test]
    fn custom_env_example_includes_base_url_when_set() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Custom,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: Some("https://api.together.xyz/v1".to_string()),
        };
        let req = requirements(&cfg, "dev");
        assert!(req
            .env_example_lines
            .iter()
            .any(|l| l.contains("https://api.together.xyz/v1")));
    }

    #[test]
    fn custom_has_no_volumes() {
        let cfg = CodexAuthConfig {
            method: CodexAuthMethod::Custom,
            azure_endpoint: None,
            custom_env_key: Some("KEY".to_string()),
            custom_base_url: Some("http://localhost:8080".to_string()),
        };
        let req = requirements(&cfg, "dev");
        assert!(req.volumes.is_empty());
    }

    #[test]
    fn custom_ignores_container_user() {
        let cfg = make_config(CodexAuthMethod::Custom);
        let req_a = requirements(&cfg, "alice");
        let cfg2 = make_config(CodexAuthMethod::Custom);
        let req_b = requirements(&cfg2, "bob");
        assert_eq!(req_a.env_vars, req_b.env_vars);
    }

    // ── Env example lines non-empty for all methods ──────────────────

    #[test]
    fn all_methods_produce_env_example_lines() {
        for method in [
            CodexAuthMethod::ApiKey,
            CodexAuthMethod::Oauth,
            CodexAuthMethod::Azure,
            CodexAuthMethod::Custom,
        ] {
            let req = requirements(&make_config(method), "dev");
            assert!(
                !req.env_example_lines.is_empty(),
                "method {:?} should produce env_example_lines",
                method
            );
        }
    }
}
