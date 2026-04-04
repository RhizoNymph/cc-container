use crate::config::project::{AgentType, ClaudeAuthMethod, CodexAuthMethod, ProjectConfig};
use crate::helm::types::{SecretKeyRef, SecretsValues};
use indexmap::IndexMap;

/// Build the SecretsValues for the Helm chart from project config.
///
/// Inspects the auth configuration to determine which secret keys are needed
/// for the agent, and scans enabled services for password env vars that should
/// be stored in a K8s Secret.
pub fn build(config: &ProjectConfig) -> SecretsValues {
    let mut auth_keys: Vec<SecretKeyRef> = Vec::new();

    // Collect auth secret keys based on agent type and auth method
    match config.agent.agent_type {
        AgentType::Claude => {
            if let Some(ref claude_auth) = config.auth.claude {
                auth_keys.extend(claude_auth_keys(claude_auth.method));
            }
        }
        AgentType::Codex => {
            if let Some(ref codex_auth) = config.auth.codex {
                auth_keys.extend(codex_auth_keys(
                    codex_auth.method,
                    codex_auth.custom_env_key.as_deref(),
                ));
            }
        }
        AgentType::Both => {
            if let Some(ref claude_auth) = config.auth.claude {
                auth_keys.extend(claude_auth_keys(claude_auth.method));
            }
            if let Some(ref codex_auth) = config.auth.codex {
                auth_keys.extend(codex_auth_keys(
                    codex_auth.method,
                    codex_auth.custom_env_key.as_deref(),
                ));
            }
        }
    }

    // Collect service credential env vars
    let mut service_credentials: IndexMap<String, String> = IndexMap::new();
    for (name, svc_config) in &config.services {
        if !svc_config.enabled {
            continue;
        }
        match name.as_str() {
            "postgres" => {
                service_credentials.insert("POSTGRES_PASSWORD".to_string(), "changeme".to_string());
            }
            "mysql" => {
                service_credentials.insert("MYSQL_PASSWORD".to_string(), "changeme".to_string());
                service_credentials
                    .insert("MYSQL_ROOT_PASSWORD".to_string(), "changeme".to_string());
            }
            "mariadb" => {
                service_credentials.insert("MARIADB_PASSWORD".to_string(), "changeme".to_string());
                service_credentials
                    .insert("MARIADB_ROOT_PASSWORD".to_string(), "changeme".to_string());
            }
            "typesense" => {
                service_credentials.insert("TYPESENSE_API_KEY".to_string(), "changeme".to_string());
            }
            "minio" => {
                service_credentials
                    .insert("MINIO_ACCESS_KEY".to_string(), "minioadmin".to_string());
                service_credentials
                    .insert("MINIO_SECRET_KEY".to_string(), "minioadmin".to_string());
            }
            "grafana" => {
                service_credentials.insert("GRAFANA_PASSWORD".to_string(), "admin".to_string());
            }
            _ => {}
        }
    }

    // Add computed connection URLs for database services
    for (name, svc_config) in &config.services {
        if !svc_config.enabled {
            continue;
        }
        match name.as_str() {
            "postgres" => {
                let db = svc_config
                    .extra
                    .get("database")
                    .and_then(|v| v.as_str())
                    .unwrap_or("devdb");
                let user = svc_config
                    .extra
                    .get("user")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dev");
                service_credentials.insert(
                    "DATABASE_URL".to_string(),
                    format!("postgres://{user}:changeme@postgres:5432/{db}"),
                );
            }
            "mysql" => {
                let db = svc_config
                    .extra
                    .get("database")
                    .and_then(|v| v.as_str())
                    .unwrap_or("devdb");
                let user = svc_config
                    .extra
                    .get("user")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dev");
                // Only add MYSQL_DATABASE_URL if postgres already claims DATABASE_URL
                if service_credentials.contains_key("DATABASE_URL") {
                    service_credentials.insert(
                        "MYSQL_URL".to_string(),
                        format!("mysql://{user}:changeme@mysql:3306/{db}"),
                    );
                } else {
                    service_credentials.insert(
                        "DATABASE_URL".to_string(),
                        format!("mysql://{user}:changeme@mysql:3306/{db}"),
                    );
                }
            }
            "mariadb" => {
                let db = svc_config
                    .extra
                    .get("database")
                    .and_then(|v| v.as_str())
                    .unwrap_or("devdb");
                let user = svc_config
                    .extra
                    .get("user")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dev");
                if service_credentials.contains_key("DATABASE_URL") {
                    service_credentials.insert(
                        "MARIADB_URL".to_string(),
                        format!("mysql://{user}:changeme@mariadb:3306/{db}"),
                    );
                } else {
                    service_credentials.insert(
                        "DATABASE_URL".to_string(),
                        format!("mysql://{user}:changeme@mariadb:3306/{db}"),
                    );
                }
            }
            _ => {}
        }
    }

    SecretsValues {
        auth_keys,
        service_credentials,
    }
}

/// Return the secret key refs needed for a given Claude auth method.
fn claude_auth_keys(method: ClaudeAuthMethod) -> Vec<SecretKeyRef> {
    match method {
        ClaudeAuthMethod::ApiKey => vec![SecretKeyRef {
            key: "ANTHROPIC_API_KEY".to_string(),
            description: "Anthropic API key for Claude Code".to_string(),
        }],
        ClaudeAuthMethod::Oauth => {
            // OAuth uses mounted credential files, no API key secret needed
            vec![]
        }
        ClaudeAuthMethod::Bedrock => vec![
            SecretKeyRef {
                key: "AWS_ACCESS_KEY_ID".to_string(),
                description: "AWS access key ID for Bedrock".to_string(),
            },
            SecretKeyRef {
                key: "AWS_SECRET_ACCESS_KEY".to_string(),
                description: "AWS secret access key for Bedrock".to_string(),
            },
            SecretKeyRef {
                key: "AWS_REGION".to_string(),
                description: "AWS region for Bedrock".to_string(),
            },
        ],
        ClaudeAuthMethod::BedrockApiKey => vec![
            SecretKeyRef {
                key: "AWS_ACCESS_KEY_ID".to_string(),
                description: "AWS access key ID for Bedrock API key auth".to_string(),
            },
            SecretKeyRef {
                key: "AWS_SECRET_ACCESS_KEY".to_string(),
                description: "AWS secret access key for Bedrock API key auth".to_string(),
            },
            SecretKeyRef {
                key: "AWS_REGION".to_string(),
                description: "AWS region for Bedrock API key auth".to_string(),
            },
        ],
        ClaudeAuthMethod::Vertex => vec![SecretKeyRef {
            key: "GOOGLE_APPLICATION_CREDENTIALS".to_string(),
            description: "Google Cloud credentials JSON for Vertex AI".to_string(),
        }],
        ClaudeAuthMethod::Proxy => vec![
            SecretKeyRef {
                key: "ANTHROPIC_BASE_URL".to_string(),
                description: "Proxy/gateway base URL for Claude".to_string(),
            },
            SecretKeyRef {
                key: "ANTHROPIC_API_KEY".to_string(),
                description: "API key for Claude proxy/gateway".to_string(),
            },
        ],
    }
}

/// Return the secret key refs needed for a given Codex auth method.
fn codex_auth_keys(method: CodexAuthMethod, custom_env_key: Option<&str>) -> Vec<SecretKeyRef> {
    match method {
        CodexAuthMethod::ApiKey => vec![SecretKeyRef {
            key: "OPENAI_API_KEY".to_string(),
            description: "OpenAI API key for Codex CLI".to_string(),
        }],
        CodexAuthMethod::Oauth => {
            // OAuth uses mounted credential files
            vec![]
        }
        CodexAuthMethod::Azure => vec![
            SecretKeyRef {
                key: "AZURE_OPENAI_API_KEY".to_string(),
                description: "Azure OpenAI API key".to_string(),
            },
            SecretKeyRef {
                key: "AZURE_OPENAI_ENDPOINT".to_string(),
                description: "Azure OpenAI endpoint URL".to_string(),
            },
        ],
        CodexAuthMethod::Custom => {
            let key = custom_env_key.unwrap_or("OPENAI_API_KEY");
            vec![SecretKeyRef {
                key: key.to_string(),
                description: "Custom provider API key for Codex".to_string(),
            }]
        }
    }
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
            auth: AuthConfig::default(),
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

    // -- No auth --

    #[test]
    fn build_no_auth_empty_keys() {
        let config = minimal_config();
        let sv = build(&config);
        assert!(sv.auth_keys.is_empty());
    }

    // -- Claude auth methods --

    #[test]
    fn build_claude_api_key() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::ApiKey,
        });
        let sv = build(&config);

        assert_eq!(sv.auth_keys.len(), 1);
        assert_eq!(sv.auth_keys[0].key, "ANTHROPIC_API_KEY");
    }

    #[test]
    fn build_claude_oauth_no_keys() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Oauth,
        });
        let sv = build(&config);

        assert!(sv.auth_keys.is_empty());
    }

    #[test]
    fn build_claude_bedrock() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Bedrock,
        });
        let sv = build(&config);

        let keys: Vec<&str> = sv.auth_keys.iter().map(|k| k.key.as_str()).collect();
        assert!(keys.contains(&"AWS_ACCESS_KEY_ID"));
        assert!(keys.contains(&"AWS_SECRET_ACCESS_KEY"));
        assert!(keys.contains(&"AWS_REGION"));
    }

    #[test]
    fn build_claude_bedrock_api_key() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::BedrockApiKey,
        });
        let sv = build(&config);

        let keys: Vec<&str> = sv.auth_keys.iter().map(|k| k.key.as_str()).collect();
        assert!(keys.contains(&"AWS_ACCESS_KEY_ID"));
        assert!(keys.contains(&"AWS_SECRET_ACCESS_KEY"));
        assert!(keys.contains(&"AWS_REGION"));
    }

    #[test]
    fn build_claude_vertex() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Vertex,
        });
        let sv = build(&config);

        assert_eq!(sv.auth_keys.len(), 1);
        assert_eq!(sv.auth_keys[0].key, "GOOGLE_APPLICATION_CREDENTIALS");
    }

    #[test]
    fn build_claude_proxy() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Proxy,
        });
        let sv = build(&config);

        let keys: Vec<&str> = sv.auth_keys.iter().map(|k| k.key.as_str()).collect();
        assert!(keys.contains(&"ANTHROPIC_BASE_URL"));
        assert!(keys.contains(&"ANTHROPIC_API_KEY"));
    }

    // -- Codex auth methods --

    #[test]
    fn build_codex_api_key() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::ApiKey,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });
        let sv = build(&config);

        assert_eq!(sv.auth_keys.len(), 1);
        assert_eq!(sv.auth_keys[0].key, "OPENAI_API_KEY");
    }

    #[test]
    fn build_codex_oauth_no_keys() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::Oauth,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });
        let sv = build(&config);

        assert!(sv.auth_keys.is_empty());
    }

    #[test]
    fn build_codex_azure() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::Azure,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });
        let sv = build(&config);

        let keys: Vec<&str> = sv.auth_keys.iter().map(|k| k.key.as_str()).collect();
        assert!(keys.contains(&"AZURE_OPENAI_API_KEY"));
        assert!(keys.contains(&"AZURE_OPENAI_ENDPOINT"));
    }

    #[test]
    fn build_codex_custom_default_key() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::Custom,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });
        let sv = build(&config);

        assert_eq!(sv.auth_keys.len(), 1);
        assert_eq!(sv.auth_keys[0].key, "OPENAI_API_KEY");
    }

    #[test]
    fn build_codex_custom_with_env_key() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::Custom,
            azure_endpoint: None,
            custom_env_key: Some("MY_PROVIDER_KEY".to_string()),
            custom_base_url: None,
        });
        let sv = build(&config);

        assert_eq!(sv.auth_keys.len(), 1);
        assert_eq!(sv.auth_keys[0].key, "MY_PROVIDER_KEY");
    }

    // -- Both agent type --

    #[test]
    fn build_both_agents_combines_keys() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::ApiKey,
        });
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::ApiKey,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });
        let sv = build(&config);

        let keys: Vec<&str> = sv.auth_keys.iter().map(|k| k.key.as_str()).collect();
        assert!(keys.contains(&"ANTHROPIC_API_KEY"));
        assert!(keys.contains(&"OPENAI_API_KEY"));
    }

    // -- Service credentials --

    #[test]
    fn build_no_services_empty_credentials() {
        let config = minimal_config();
        let sv = build(&config);
        assert!(sv.service_credentials.is_empty());
    }

    #[test]
    fn build_postgres_credentials() {
        let mut config = minimal_config();
        config.services.insert(
            "postgres".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        let sv = build(&config);

        assert_eq!(sv.service_credentials.len(), 2);
        assert_eq!(sv.service_credentials["POSTGRES_PASSWORD"], "changeme");
        assert!(sv.service_credentials.contains_key("DATABASE_URL"));
    }

    #[test]
    fn build_mysql_credentials() {
        let mut config = minimal_config();
        config.services.insert(
            "mysql".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        let sv = build(&config);

        assert_eq!(sv.service_credentials.len(), 3);
        assert_eq!(sv.service_credentials["MYSQL_PASSWORD"], "changeme");
        assert_eq!(sv.service_credentials["MYSQL_ROOT_PASSWORD"], "changeme");
        assert!(sv.service_credentials.contains_key("DATABASE_URL"));
    }

    #[test]
    fn build_mariadb_credentials() {
        let mut config = minimal_config();
        config.services.insert(
            "mariadb".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        let sv = build(&config);

        assert_eq!(sv.service_credentials.len(), 3);
        assert_eq!(sv.service_credentials["MARIADB_PASSWORD"], "changeme");
        assert_eq!(sv.service_credentials["MARIADB_ROOT_PASSWORD"], "changeme");
        assert!(sv.service_credentials.contains_key("DATABASE_URL"));
    }

    #[test]
    fn build_disabled_service_skipped() {
        let mut config = minimal_config();
        config.services.insert(
            "postgres".to_string(),
            ServiceConfig {
                enabled: false,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        let sv = build(&config);

        assert!(sv.service_credentials.is_empty());
    }

    #[test]
    fn build_redis_no_credentials() {
        let mut config = minimal_config();
        config.services.insert(
            "redis".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        let sv = build(&config);

        assert!(sv.service_credentials.is_empty());
    }

    #[test]
    fn build_multiple_services() {
        let mut config = minimal_config();
        config.services.insert(
            "postgres".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        config.services.insert(
            "mysql".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        config.services.insert(
            "redis".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );
        let sv = build(&config);

        // postgres: 1 key + DATABASE_URL, mysql: 2 keys + MYSQL_URL, redis: 0 keys
        assert_eq!(sv.service_credentials.len(), 5);
        assert!(sv.service_credentials.contains_key("POSTGRES_PASSWORD"));
        assert!(sv.service_credentials.contains_key("DATABASE_URL"));
        assert!(sv.service_credentials.contains_key("MYSQL_PASSWORD"));
        assert!(sv.service_credentials.contains_key("MYSQL_ROOT_PASSWORD"));
        assert!(sv.service_credentials.contains_key("MYSQL_URL"));
    }
}
