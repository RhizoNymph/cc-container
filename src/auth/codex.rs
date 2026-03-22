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
