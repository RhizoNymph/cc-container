use super::{AuthRequirements, AuthVolume};
use crate::config::project::{ClaudeAuthConfig, ClaudeAuthMethod};
use indexmap::IndexMap;

/// Build auth requirements for Claude Code based on the configured method.
pub fn requirements(auth: &ClaudeAuthConfig, container_user: &str) -> AuthRequirements {
    match auth.method {
        ClaudeAuthMethod::ApiKey => api_key_requirements(),
        ClaudeAuthMethod::Oauth => oauth_requirements(container_user),
        ClaudeAuthMethod::Bedrock => bedrock_requirements(),
        ClaudeAuthMethod::BedrockApiKey => bedrock_api_key_requirements(),
        ClaudeAuthMethod::Vertex => vertex_requirements(container_user),
        ClaudeAuthMethod::Proxy => proxy_requirements(),
    }
}

fn api_key_requirements() -> AuthRequirements {
    AuthRequirements {
        env_vars: IndexMap::from([(
            "ANTHROPIC_API_KEY".to_string(),
            "${ANTHROPIC_API_KEY}".to_string(),
        )]),
        volumes: vec![],
        env_example_lines: vec![
            "# Claude Code - API Key auth".to_string(),
            "ANTHROPIC_API_KEY=your-anthropic-api-key-here".to_string(),
        ],
    }
}

fn oauth_requirements(container_user: &str) -> AuthRequirements {
    AuthRequirements {
        env_vars: IndexMap::new(),
        volumes: vec![AuthVolume {
            source: "${HOME}/.claude/.credentials.json".to_string(),
            target: format!("/home/{}/.claude/.credentials.json", container_user),
            read_only: true,
        }],
        env_example_lines: vec![
            "# Claude Code - OAuth auth".to_string(),
            "# No env vars needed. Credentials mounted from host ~/.claude/.credentials.json"
                .to_string(),
            "# Run `claude /login` on your host machine first.".to_string(),
        ],
    }
}

fn bedrock_requirements() -> AuthRequirements {
    AuthRequirements {
        env_vars: IndexMap::from([
            ("CLAUDE_CODE_USE_BEDROCK".to_string(), "1".to_string()),
            (
                "AWS_ACCESS_KEY_ID".to_string(),
                "${AWS_ACCESS_KEY_ID}".to_string(),
            ),
            (
                "AWS_SECRET_ACCESS_KEY".to_string(),
                "${AWS_SECRET_ACCESS_KEY}".to_string(),
            ),
            (
                "AWS_SESSION_TOKEN".to_string(),
                "${AWS_SESSION_TOKEN:-}".to_string(),
            ),
            ("AWS_REGION".to_string(), "${AWS_REGION}".to_string()),
        ]),
        volumes: vec![],
        env_example_lines: vec![
            "# Claude Code - AWS Bedrock auth".to_string(),
            "AWS_ACCESS_KEY_ID=your-access-key-id".to_string(),
            "AWS_SECRET_ACCESS_KEY=your-secret-access-key".to_string(),
            "AWS_SESSION_TOKEN=optional-session-token".to_string(),
            "AWS_REGION=us-east-1".to_string(),
        ],
    }
}

fn bedrock_api_key_requirements() -> AuthRequirements {
    AuthRequirements {
        env_vars: IndexMap::from([
            ("CLAUDE_CODE_USE_BEDROCK".to_string(), "1".to_string()),
            (
                "AWS_BEARER_TOKEN_BEDROCK".to_string(),
                "${AWS_BEARER_TOKEN_BEDROCK}".to_string(),
            ),
            ("AWS_REGION".to_string(), "${AWS_REGION}".to_string()),
        ]),
        volumes: vec![],
        env_example_lines: vec![
            "# Claude Code - Bedrock API Key auth".to_string(),
            "AWS_BEARER_TOKEN_BEDROCK=your-bedrock-api-key".to_string(),
            "AWS_REGION=us-east-1".to_string(),
        ],
    }
}

fn vertex_requirements(container_user: &str) -> AuthRequirements {
    AuthRequirements {
        env_vars: IndexMap::from([
            ("CLAUDE_CODE_USE_VERTEX".to_string(), "1".to_string()),
            (
                "GOOGLE_APPLICATION_CREDENTIALS".to_string(),
                format!(
                    "/home/{}/.config/gcloud/application_default_credentials.json",
                    container_user
                ),
            ),
            (
                "ANTHROPIC_VERTEX_PROJECT_ID".to_string(),
                "${ANTHROPIC_VERTEX_PROJECT_ID}".to_string(),
            ),
        ]),
        volumes: vec![AuthVolume {
            source: "${GOOGLE_APPLICATION_CREDENTIALS}".to_string(),
            target: format!(
                "/home/{}/.config/gcloud/application_default_credentials.json",
                container_user
            ),
            read_only: true,
        }],
        env_example_lines: vec![
            "# Claude Code - Google Vertex AI auth".to_string(),
            "GOOGLE_APPLICATION_CREDENTIALS=/path/to/credentials.json".to_string(),
            "ANTHROPIC_VERTEX_PROJECT_ID=your-gcp-project-id".to_string(),
        ],
    }
}

fn proxy_requirements() -> AuthRequirements {
    AuthRequirements {
        env_vars: IndexMap::from([
            (
                "ANTHROPIC_AUTH_TOKEN".to_string(),
                "${ANTHROPIC_AUTH_TOKEN}".to_string(),
            ),
            (
                "ANTHROPIC_BASE_URL".to_string(),
                "${ANTHROPIC_BASE_URL}".to_string(),
            ),
        ]),
        volumes: vec![],
        env_example_lines: vec![
            "# Claude Code - Proxy/Gateway auth".to_string(),
            "ANTHROPIC_AUTH_TOKEN=your-proxy-bearer-token".to_string(),
            "ANTHROPIC_BASE_URL=http://your-proxy:4000".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::{ClaudeAuthConfig, ClaudeAuthMethod};

    fn make_config(method: ClaudeAuthMethod) -> ClaudeAuthConfig {
        ClaudeAuthConfig { method }
    }

    // ── API Key ──────────────────────────────────────────────────────

    #[test]
    fn api_key_sets_anthropic_api_key_env() {
        let cfg = make_config(ClaudeAuthMethod::ApiKey);
        let req = requirements(&cfg, "developer");

        assert_eq!(req.env_vars.len(), 1);
        assert_eq!(
            req.env_vars.get("ANTHROPIC_API_KEY").unwrap(),
            "${ANTHROPIC_API_KEY}"
        );
    }

    #[test]
    fn api_key_has_no_volumes() {
        let req = requirements(&make_config(ClaudeAuthMethod::ApiKey), "dev");
        assert!(req.volumes.is_empty());
    }

    #[test]
    fn api_key_env_example_mentions_api_key() {
        let req = requirements(&make_config(ClaudeAuthMethod::ApiKey), "dev");
        assert!(
            req.env_example_lines
                .iter()
                .any(|l| l.contains("ANTHROPIC_API_KEY"))
        );
    }

    // ── OAuth ────────────────────────────────────────────────────────

    #[test]
    fn oauth_has_no_env_vars() {
        let req = requirements(&make_config(ClaudeAuthMethod::Oauth), "developer");
        assert!(req.env_vars.is_empty());
    }

    #[test]
    fn oauth_mounts_credentials_file() {
        let req = requirements(&make_config(ClaudeAuthMethod::Oauth), "developer");
        assert_eq!(req.volumes.len(), 1);

        let vol = &req.volumes[0];
        assert_eq!(vol.source, "${HOME}/.claude/.credentials.json");
        assert_eq!(vol.target, "/home/developer/.claude/.credentials.json");
        assert!(vol.read_only);
    }

    #[test]
    fn oauth_volume_target_uses_container_user() {
        let req = requirements(&make_config(ClaudeAuthMethod::Oauth), "alice");
        assert_eq!(
            req.volumes[0].target,
            "/home/alice/.claude/.credentials.json"
        );
    }

    #[test]
    fn oauth_env_example_mentions_no_env_vars() {
        let req = requirements(&make_config(ClaudeAuthMethod::Oauth), "dev");
        assert!(
            req.env_example_lines
                .iter()
                .any(|l| l.contains("No env vars needed"))
        );
    }

    // ── Bedrock (IAM credentials) ────────────────────────────────────

    #[test]
    fn bedrock_sets_use_bedrock_flag() {
        let req = requirements(&make_config(ClaudeAuthMethod::Bedrock), "dev");
        assert_eq!(req.env_vars.get("CLAUDE_CODE_USE_BEDROCK").unwrap(), "1");
    }

    #[test]
    fn bedrock_sets_aws_credential_env_vars() {
        let req = requirements(&make_config(ClaudeAuthMethod::Bedrock), "dev");
        assert_eq!(
            req.env_vars.get("AWS_ACCESS_KEY_ID").unwrap(),
            "${AWS_ACCESS_KEY_ID}"
        );
        assert_eq!(
            req.env_vars.get("AWS_SECRET_ACCESS_KEY").unwrap(),
            "${AWS_SECRET_ACCESS_KEY}"
        );
        assert_eq!(
            req.env_vars.get("AWS_SESSION_TOKEN").unwrap(),
            "${AWS_SESSION_TOKEN:-}"
        );
        assert_eq!(req.env_vars.get("AWS_REGION").unwrap(), "${AWS_REGION}");
    }

    #[test]
    fn bedrock_has_five_env_vars() {
        let req = requirements(&make_config(ClaudeAuthMethod::Bedrock), "dev");
        assert_eq!(req.env_vars.len(), 5);
    }

    #[test]
    fn bedrock_has_no_volumes() {
        let req = requirements(&make_config(ClaudeAuthMethod::Bedrock), "dev");
        assert!(req.volumes.is_empty());
    }

    #[test]
    fn bedrock_session_token_is_optional_via_default() {
        let req = requirements(&make_config(ClaudeAuthMethod::Bedrock), "dev");
        // The `:-` syntax means empty-default, so the token is optional
        let val = req.env_vars.get("AWS_SESSION_TOKEN").unwrap();
        assert!(
            val.contains(":-"),
            "session token should have an empty default"
        );
    }

    // ── Bedrock API Key ──────────────────────────────────────────────

    #[test]
    fn bedrock_api_key_sets_use_bedrock_flag() {
        let req = requirements(&make_config(ClaudeAuthMethod::BedrockApiKey), "dev");
        assert_eq!(req.env_vars.get("CLAUDE_CODE_USE_BEDROCK").unwrap(), "1");
    }

    #[test]
    fn bedrock_api_key_sets_bearer_token_and_region() {
        let req = requirements(&make_config(ClaudeAuthMethod::BedrockApiKey), "dev");
        assert_eq!(
            req.env_vars.get("AWS_BEARER_TOKEN_BEDROCK").unwrap(),
            "${AWS_BEARER_TOKEN_BEDROCK}"
        );
        assert_eq!(req.env_vars.get("AWS_REGION").unwrap(), "${AWS_REGION}");
    }

    #[test]
    fn bedrock_api_key_has_three_env_vars() {
        let req = requirements(&make_config(ClaudeAuthMethod::BedrockApiKey), "dev");
        assert_eq!(req.env_vars.len(), 3);
    }

    #[test]
    fn bedrock_api_key_has_no_volumes() {
        let req = requirements(&make_config(ClaudeAuthMethod::BedrockApiKey), "dev");
        assert!(req.volumes.is_empty());
    }

    #[test]
    fn bedrock_api_key_does_not_set_iam_credentials() {
        let req = requirements(&make_config(ClaudeAuthMethod::BedrockApiKey), "dev");
        assert!(req.env_vars.get("AWS_ACCESS_KEY_ID").is_none());
        assert!(req.env_vars.get("AWS_SECRET_ACCESS_KEY").is_none());
    }

    // ── Vertex ───────────────────────────────────────────────────────

    #[test]
    fn vertex_sets_use_vertex_flag() {
        let req = requirements(&make_config(ClaudeAuthMethod::Vertex), "dev");
        assert_eq!(req.env_vars.get("CLAUDE_CODE_USE_VERTEX").unwrap(), "1");
    }

    #[test]
    fn vertex_sets_gcloud_credentials_path() {
        let req = requirements(&make_config(ClaudeAuthMethod::Vertex), "developer");
        assert_eq!(
            req.env_vars.get("GOOGLE_APPLICATION_CREDENTIALS").unwrap(),
            "/home/developer/.config/gcloud/application_default_credentials.json"
        );
    }

    #[test]
    fn vertex_sets_project_id_env() {
        let req = requirements(&make_config(ClaudeAuthMethod::Vertex), "dev");
        assert_eq!(
            req.env_vars.get("ANTHROPIC_VERTEX_PROJECT_ID").unwrap(),
            "${ANTHROPIC_VERTEX_PROJECT_ID}"
        );
    }

    #[test]
    fn vertex_mounts_gcloud_credentials() {
        let req = requirements(&make_config(ClaudeAuthMethod::Vertex), "developer");
        assert_eq!(req.volumes.len(), 1);

        let vol = &req.volumes[0];
        assert_eq!(vol.source, "${GOOGLE_APPLICATION_CREDENTIALS}");
        assert_eq!(
            vol.target,
            "/home/developer/.config/gcloud/application_default_credentials.json"
        );
        assert!(vol.read_only);
    }

    #[test]
    fn vertex_volume_target_uses_container_user() {
        let req = requirements(&make_config(ClaudeAuthMethod::Vertex), "bob");
        assert!(req.volumes[0].target.contains("/home/bob/"));
    }

    #[test]
    fn vertex_credentials_path_matches_volume_target() {
        let req = requirements(&make_config(ClaudeAuthMethod::Vertex), "dev");
        let cred_env = req.env_vars.get("GOOGLE_APPLICATION_CREDENTIALS").unwrap();
        let vol_target = &req.volumes[0].target;
        assert_eq!(cred_env, vol_target);
    }

    // ── Proxy ────────────────────────────────────────────────────────

    #[test]
    fn proxy_sets_auth_token_and_base_url() {
        let req = requirements(&make_config(ClaudeAuthMethod::Proxy), "dev");
        assert_eq!(
            req.env_vars.get("ANTHROPIC_AUTH_TOKEN").unwrap(),
            "${ANTHROPIC_AUTH_TOKEN}"
        );
        assert_eq!(
            req.env_vars.get("ANTHROPIC_BASE_URL").unwrap(),
            "${ANTHROPIC_BASE_URL}"
        );
    }

    #[test]
    fn proxy_has_two_env_vars() {
        let req = requirements(&make_config(ClaudeAuthMethod::Proxy), "dev");
        assert_eq!(req.env_vars.len(), 2);
    }

    #[test]
    fn proxy_has_no_volumes() {
        let req = requirements(&make_config(ClaudeAuthMethod::Proxy), "dev");
        assert!(req.volumes.is_empty());
    }

    #[test]
    fn proxy_env_example_mentions_base_url() {
        let req = requirements(&make_config(ClaudeAuthMethod::Proxy), "dev");
        assert!(
            req.env_example_lines
                .iter()
                .any(|l| l.contains("ANTHROPIC_BASE_URL"))
        );
    }

    // ── Cross-method: container_user is irrelevant for non-mount methods ─

    #[test]
    fn api_key_ignores_container_user() {
        let req_a = requirements(&make_config(ClaudeAuthMethod::ApiKey), "alice");
        let req_b = requirements(&make_config(ClaudeAuthMethod::ApiKey), "bob");
        assert_eq!(req_a.env_vars, req_b.env_vars);
        assert!(req_a.volumes.is_empty());
    }

    #[test]
    fn bedrock_ignores_container_user() {
        let req_a = requirements(&make_config(ClaudeAuthMethod::Bedrock), "alice");
        let req_b = requirements(&make_config(ClaudeAuthMethod::Bedrock), "bob");
        assert_eq!(req_a.env_vars, req_b.env_vars);
    }

    #[test]
    fn proxy_ignores_container_user() {
        let req_a = requirements(&make_config(ClaudeAuthMethod::Proxy), "alice");
        let req_b = requirements(&make_config(ClaudeAuthMethod::Proxy), "bob");
        assert_eq!(req_a.env_vars, req_b.env_vars);
    }

    // ── Env example lines are non-empty for every method ─────────────

    #[test]
    fn all_methods_produce_env_example_lines() {
        for method in [
            ClaudeAuthMethod::ApiKey,
            ClaudeAuthMethod::Oauth,
            ClaudeAuthMethod::Bedrock,
            ClaudeAuthMethod::BedrockApiKey,
            ClaudeAuthMethod::Vertex,
            ClaudeAuthMethod::Proxy,
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
