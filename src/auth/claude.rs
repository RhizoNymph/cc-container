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
            source: "~/.claude/.credentials.json".to_string(),
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
            (
                "CLAUDE_CODE_USE_BEDROCK".to_string(),
                "1".to_string(),
            ),
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
            (
                "AWS_REGION".to_string(),
                "${AWS_REGION}".to_string(),
            ),
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
            (
                "CLAUDE_CODE_USE_BEDROCK".to_string(),
                "1".to_string(),
            ),
            (
                "AWS_BEARER_TOKEN_BEDROCK".to_string(),
                "${AWS_BEARER_TOKEN_BEDROCK}".to_string(),
            ),
            (
                "AWS_REGION".to_string(),
                "${AWS_REGION}".to_string(),
            ),
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
            (
                "CLAUDE_CODE_USE_VERTEX".to_string(),
                "1".to_string(),
            ),
            (
                "GOOGLE_APPLICATION_CREDENTIALS".to_string(),
                format!("/home/{}/.config/gcloud/application_default_credentials.json", container_user),
            ),
            (
                "ANTHROPIC_VERTEX_PROJECT_ID".to_string(),
                "${ANTHROPIC_VERTEX_PROJECT_ID}".to_string(),
            ),
        ]),
        volumes: vec![AuthVolume {
            source: "${GOOGLE_APPLICATION_CREDENTIALS}".to_string(),
            target: format!("/home/{}/.config/gcloud/application_default_credentials.json", container_user),
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
