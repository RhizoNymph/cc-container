use super::project::{AgentType, ProjectConfig};

/// A validation warning (non-fatal).
#[derive(Debug)]
pub struct ValidationWarning {
    pub message: String,
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Validate a project config and return any warnings or errors.
pub fn validate_config(config: &ProjectConfig) -> crate::error::Result<Vec<ValidationWarning>> {
    let mut warnings = Vec::new();

    // Check that auth is configured for the selected agent type
    match config.agent.agent_type {
        AgentType::Claude | AgentType::Both => {
            if config.auth.claude.is_none() {
                warnings.push(ValidationWarning {
                    message: "No [auth.claude] configured. You will need to set up authentication."
                        .to_string(),
                });
            }
        }
        AgentType::Codex => {}
    }

    match config.agent.agent_type {
        AgentType::Codex | AgentType::Both => {
            if config.auth.codex.is_none() {
                warnings.push(ValidationWarning {
                    message: "No [auth.codex] configured. You will need to set up authentication."
                        .to_string(),
                });
            }
        }
        AgentType::Claude => {}
    }

    // Check for port conflicts among enabled services
    let mut ports: Vec<(u16, String)> = Vec::new();
    for (name, svc) in &config.services {
        if !svc.enabled {
            continue;
        }
        if let Some(port) = svc.port {
            for (existing_port, existing_name) in &ports {
                if *existing_port == port {
                    return Err(crate::error::Error::PortConflict {
                        port,
                        a: existing_name.clone(),
                        b: name.clone(),
                    });
                }
            }
            ports.push((port, name.clone()));
        }
    }

    // Warn if firewall is enabled but NET_ADMIN not in cap_add
    if config.firewall.enabled && !config.runtime.cap_add.iter().any(|c| c == "NET_ADMIN") {
        warnings.push(ValidationWarning {
            message:
                "Firewall is enabled but NET_ADMIN capability not in [runtime].cap_add. Firewall rules require NET_ADMIN."
                    .to_string(),
        });
    }

    Ok(warnings)
}
