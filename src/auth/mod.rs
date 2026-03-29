pub mod claude;
pub mod codex;

use indexmap::IndexMap;

/// Volume mount needed for auth.
#[derive(Debug, Clone)]
pub struct AuthVolume {
    pub source: String,
    pub target: String,
    pub read_only: bool,
}

/// Auth requirements for a compose service.
#[derive(Debug, Clone, Default)]
pub struct AuthRequirements {
    /// Environment variables to set (key -> value or ${REF}).
    pub env_vars: IndexMap<String, String>,
    /// Volume mounts needed for credential files.
    pub volumes: Vec<AuthVolume>,
    /// Lines to add to .env.example.
    pub env_example_lines: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::{
        ClaudeAuthConfig, ClaudeAuthMethod, CodexAuthConfig, CodexAuthMethod,
    };

    // ── AuthRequirements Default ─────────────────────────────────────

    #[test]
    fn auth_requirements_default_is_empty() {
        let req = AuthRequirements::default();
        assert!(req.env_vars.is_empty());
        assert!(req.volumes.is_empty());
        assert!(req.env_example_lines.is_empty());
    }

    // ── AuthVolume struct ────────────────────────────────────────────

    #[test]
    fn auth_volume_clone_preserves_fields() {
        let vol = AuthVolume {
            source: "/host/path".to_string(),
            target: "/container/path".to_string(),
            read_only: true,
        };
        let cloned = vol.clone();
        assert_eq!(cloned.source, "/host/path");
        assert_eq!(cloned.target, "/container/path");
        assert!(cloned.read_only);
    }

    #[test]
    fn auth_volume_read_only_can_be_false() {
        let vol = AuthVolume {
            source: "src".to_string(),
            target: "tgt".to_string(),
            read_only: false,
        };
        assert!(!vol.read_only);
    }

    // ── Claude dispatch ──────────────────────────────────────────────

    #[test]
    fn claude_dispatch_api_key() {
        let cfg = ClaudeAuthConfig {
            method: ClaudeAuthMethod::ApiKey,
        };
        let req = claude::requirements(&cfg, "dev");
        assert!(req.env_vars.contains_key("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn claude_dispatch_oauth() {
        let cfg = ClaudeAuthConfig {
            method: ClaudeAuthMethod::Oauth,
        };
        let req = claude::requirements(&cfg, "dev");
        assert_eq!(req.volumes.len(), 1);
        assert!(req.env_vars.is_empty());
    }

    #[test]
    fn claude_dispatch_bedrock() {
        let cfg = ClaudeAuthConfig {
            method: ClaudeAuthMethod::Bedrock,
        };
        let req = claude::requirements(&cfg, "dev");
        assert!(req.env_vars.contains_key("CLAUDE_CODE_USE_BEDROCK"));
        assert!(req.env_vars.contains_key("AWS_ACCESS_KEY_ID"));
    }

    #[test]
    fn claude_dispatch_bedrock_api_key() {
        let cfg = ClaudeAuthConfig {
            method: ClaudeAuthMethod::BedrockApiKey,
        };
        let req = claude::requirements(&cfg, "dev");
        assert!(req.env_vars.contains_key("CLAUDE_CODE_USE_BEDROCK"));
        assert!(req.env_vars.contains_key("AWS_BEARER_TOKEN_BEDROCK"));
    }

    #[test]
    fn claude_dispatch_vertex() {
        let cfg = ClaudeAuthConfig {
            method: ClaudeAuthMethod::Vertex,
        };
        let req = claude::requirements(&cfg, "dev");
        assert!(req.env_vars.contains_key("CLAUDE_CODE_USE_VERTEX"));
        assert_eq!(req.volumes.len(), 1);
    }

    #[test]
    fn claude_dispatch_proxy() {
        let cfg = ClaudeAuthConfig {
            method: ClaudeAuthMethod::Proxy,
        };
        let req = claude::requirements(&cfg, "dev");
        assert!(req.env_vars.contains_key("ANTHROPIC_AUTH_TOKEN"));
        assert!(req.env_vars.contains_key("ANTHROPIC_BASE_URL"));
    }

    // ── Codex dispatch ───────────────────────────────────────────────

    fn codex_config(method: CodexAuthMethod) -> CodexAuthConfig {
        CodexAuthConfig {
            method,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        }
    }

    #[test]
    fn codex_dispatch_api_key() {
        let req = codex::requirements(&codex_config(CodexAuthMethod::ApiKey), "dev");
        assert!(req.env_vars.contains_key("OPENAI_API_KEY"));
    }

    #[test]
    fn codex_dispatch_oauth() {
        let req = codex::requirements(&codex_config(CodexAuthMethod::Oauth), "dev");
        assert_eq!(req.volumes.len(), 1);
        assert!(req.env_vars.is_empty());
    }

    #[test]
    fn codex_dispatch_azure() {
        let req = codex::requirements(&codex_config(CodexAuthMethod::Azure), "dev");
        assert!(req.env_vars.contains_key("AZURE_OPENAI_API_KEY"));
    }

    #[test]
    fn codex_dispatch_custom() {
        let req = codex::requirements(&codex_config(CodexAuthMethod::Custom), "dev");
        assert!(req.env_vars.contains_key("CUSTOM_API_KEY"));
    }

    // ── Claude and Codex produce distinct env vars ───────────────────

    #[test]
    fn claude_and_codex_api_key_use_different_env_vars() {
        let claude_req = claude::requirements(
            &ClaudeAuthConfig {
                method: ClaudeAuthMethod::ApiKey,
            },
            "dev",
        );
        let codex_req = codex::requirements(&codex_config(CodexAuthMethod::ApiKey), "dev");

        // They must not share any env var keys
        for key in claude_req.env_vars.keys() {
            assert!(
                !codex_req.env_vars.contains_key(key),
                "env var {} should not be shared between claude and codex api-key auth",
                key
            );
        }
    }

    #[test]
    fn claude_and_codex_oauth_mount_different_paths() {
        let claude_req = claude::requirements(
            &ClaudeAuthConfig {
                method: ClaudeAuthMethod::Oauth,
            },
            "dev",
        );
        let codex_req = codex::requirements(&codex_config(CodexAuthMethod::Oauth), "dev");

        assert_ne!(
            claude_req.volumes[0].source, codex_req.volumes[0].source,
            "Claude and Codex OAuth should mount from different host paths"
        );
        assert_ne!(
            claude_req.volumes[0].target, codex_req.volumes[0].target,
            "Claude and Codex OAuth should mount to different container paths"
        );
    }
}
