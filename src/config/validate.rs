use super::project::{AgentType, ProjectConfig, ServiceConfig};

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

    // Check for port conflicts among enabled services (primary + secondary ports)
    let mut ports: Vec<(u16, String)> = Vec::new();
    for (name, svc) in &config.services {
        if !svc.enabled {
            continue;
        }
        for (port, label) in all_ports_for_service(name, svc) {
            for (existing_port, existing_label) in &ports {
                if *existing_port == port {
                    return Err(crate::error::Error::PortConflict {
                        port,
                        a: existing_label.clone(),
                        b: label.clone(),
                    });
                }
            }
            ports.push((port, label));
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

/// Returns all host-bound ports for a service: the primary configurable port
/// plus any hardcoded secondary ports emitted by the template.
fn all_ports_for_service(name: &str, config: &ServiceConfig) -> Vec<(u16, String)> {
    let mut ports = Vec::new();

    if let Some(port) = config.port {
        ports.push((port, name.to_string()));
    }

    // Secondary hardcoded ports from service templates
    match name {
        "cockroachdb" => ports.push((8080, format!("{name} admin UI"))),
        "rabbitmq" => ports.push((15672, format!("{name} management"))),
        "kafka" => ports.push((8081, format!("{name} schema registry"))),
        "nats" => ports.push((8222, format!("{name} monitoring"))),
        "traefik" => ports.push((8080, format!("{name} dashboard"))),
        "minio" => {
            let console_port = config.extra
                .get("console_port")
                .and_then(|v| v.as_integer())
                .unwrap_or(9001) as u16;
            ports.push((console_port, format!("{name} console")));
        }
        _ => {}
    }

    ports
}
