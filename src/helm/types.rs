use indexmap::IndexMap;
use serde::Serialize;

/// Root values.yaml representation for the generated Helm chart.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmValues {
    pub project_name: String,
    pub namespace: String,
    pub agent: AgentValues,
    pub services: IndexMap<String, ServiceValues>,
    pub mcp: IndexMap<String, McpValues>,
    pub network_policy: NetworkPolicyValues,
    pub secrets: SecretsValues,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress: Option<IngressValues>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,
}

/// Agent container configuration in values.yaml.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentValues {
    pub agent_type: String,
    pub image: ImageRef,
    pub replicas: u32,
    pub resources: ResourceLimits,
    pub env: IndexMap<String, String>,
    pub env_from_secret: Vec<String>,
    pub volume_mounts: Vec<VolumeMount>,
    pub workspace_pvc_size: String,
    pub workspace_mount_path: String,
    pub security_context: SecurityContext,
}

/// Container image reference for K8s resources.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    pub repository: String,
    pub tag: String,
    pub pull_policy: String,
}

/// K8s resource limits and requests.
#[derive(Debug, Clone, Serialize)]
pub struct ResourceLimits {
    pub requests: ResourceSpec,
    pub limits: ResourceSpec,
}

/// CPU and memory resource specification.
#[derive(Debug, Clone, Serialize)]
pub struct ResourceSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

/// Per-infrastructure-service values for the Helm chart.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceValues {
    pub enabled: bool,
    pub image: ImageRef,
    pub category: String,
    pub stateful: bool,
    pub ports: Vec<PortSpec>,
    pub env: IndexMap<String, String>,
    pub env_from_secret: IndexMap<String, String>,
    pub agent_env: IndexMap<String, String>,
    pub volume_mounts: Vec<VolumeMount>,
    pub pvc_size: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    pub healthcheck: HealthcheckSpec,
    pub resources: ResourceLimits,
}

/// K8s container port specification.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortSpec {
    pub name: String,
    pub container_port: u16,
    pub protocol: String,
}

/// Volume mount for a K8s container.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub read_only: bool,
}

/// K8s liveness/readiness probe specification.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthcheckSpec {
    pub command: Vec<String>,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub failure_threshold: u32,
}

/// NetworkPolicy values for the Helm chart.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPolicyValues {
    pub enabled: bool,
    pub allowed_cidrs: Vec<String>,
    /// Domains are stored as annotations since standard K8s NetworkPolicy
    /// does not support FQDN-based egress rules. Users with Cilium/Calico
    /// can reference these for their CNI-specific policies.
    pub allowed_domains: Vec<String>,
    pub allow_dns: bool,
    pub allow_ssh: bool,
}

/// Secrets template values for auth and service credentials.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretsValues {
    pub auth_keys: Vec<SecretKeyRef>,
    pub service_credentials: IndexMap<String, String>,
}

/// Reference to a key within a K8s Secret.
#[derive(Debug, Clone, Serialize)]
pub struct SecretKeyRef {
    pub key: String,
    pub description: String,
}

/// MCP sidecar container values.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpValues {
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    pub env_from_secret: Vec<String>,
    pub ports: Vec<PortSpec>,
}

/// K8s security context for a pod/container.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityContext {
    pub run_as_non_root: bool,
    pub capabilities_add: Vec<String>,
    pub capabilities_drop: Vec<String>,
}

/// Ingress resource values.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngressValues {
    pub class_name: String,
    pub host: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_ref_serializes_with_camel_case() {
        let img = ImageRef {
            registry: Some("ghcr.io".to_string()),
            repository: "myorg/myapp".to_string(),
            tag: "latest".to_string(),
            pull_policy: "IfNotPresent".to_string(),
        };
        let yaml = serde_yaml::to_string(&img).unwrap();
        assert!(yaml.contains("pullPolicy"));
        assert!(yaml.contains("ghcr.io"));
    }

    #[test]
    fn image_ref_omits_null_registry() {
        let img = ImageRef {
            registry: None,
            repository: "myapp".to_string(),
            tag: "v1".to_string(),
            pull_policy: "Always".to_string(),
        };
        let yaml = serde_yaml::to_string(&img).unwrap();
        assert!(!yaml.contains("registry"));
    }

    #[test]
    fn resource_spec_omits_null_fields() {
        let spec = ResourceSpec {
            cpu: Some("1".to_string()),
            memory: None,
        };
        let yaml = serde_yaml::to_string(&spec).unwrap();
        assert!(yaml.contains("cpu"));
        assert!(!yaml.contains("memory"));
    }

    #[test]
    fn port_spec_serializes_container_port_as_camel_case() {
        let port = PortSpec {
            name: "http".to_string(),
            container_port: 8080,
            protocol: "TCP".to_string(),
        };
        let yaml = serde_yaml::to_string(&port).unwrap();
        assert!(yaml.contains("containerPort: 8080"));
    }

    #[test]
    fn volume_mount_omits_read_only_when_false() {
        let vm = VolumeMount {
            name: "data".to_string(),
            mount_path: "/data".to_string(),
            read_only: false,
        };
        let yaml = serde_yaml::to_string(&vm).unwrap();
        assert!(!yaml.contains("readOnly"));
    }

    #[test]
    fn volume_mount_includes_read_only_when_true() {
        let vm = VolumeMount {
            name: "config".to_string(),
            mount_path: "/etc/config".to_string(),
            read_only: true,
        };
        let yaml = serde_yaml::to_string(&vm).unwrap();
        assert!(yaml.contains("readOnly: true"));
    }

    #[test]
    fn healthcheck_spec_serializes_correctly() {
        let hc = HealthcheckSpec {
            command: vec![
                "pg_isready".to_string(),
                "-U".to_string(),
                "dev".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        };
        let yaml = serde_yaml::to_string(&hc).unwrap();
        assert!(yaml.contains("initialDelaySeconds: 30"));
        assert!(yaml.contains("periodSeconds: 10"));
        assert!(yaml.contains("failureThreshold: 5"));
    }

    #[test]
    fn network_policy_values_serializes_domains() {
        let np = NetworkPolicyValues {
            enabled: true,
            allowed_cidrs: vec!["10.0.0.0/8".to_string()],
            allowed_domains: vec!["api.anthropic.com".to_string()],
            allow_dns: true,
            allow_ssh: false,
        };
        let yaml = serde_yaml::to_string(&np).unwrap();
        assert!(yaml.contains("allowedDomains"));
        assert!(yaml.contains("api.anthropic.com"));
        assert!(yaml.contains("allowDns: true"));
        assert!(yaml.contains("allowSsh: false"));
    }

    #[test]
    fn secrets_values_serializes() {
        let sv = SecretsValues {
            auth_keys: vec![SecretKeyRef {
                key: "ANTHROPIC_API_KEY".to_string(),
                description: "Claude API key".to_string(),
            }],
            service_credentials: IndexMap::from([(
                "POSTGRES_PASSWORD".to_string(),
                "changeme".to_string(),
            )]),
        };
        let yaml = serde_yaml::to_string(&sv).unwrap();
        assert!(yaml.contains("ANTHROPIC_API_KEY"));
        assert!(yaml.contains("POSTGRES_PASSWORD"));
    }

    #[test]
    fn mcp_values_omits_null_command() {
        let mcp = McpValues {
            image: "test:latest".to_string(),
            command: None,
            env_from_secret: vec!["API_KEY".to_string()],
            ports: vec![],
        };
        let yaml = serde_yaml::to_string(&mcp).unwrap();
        assert!(!yaml.contains("command"));
        assert!(yaml.contains("envFromSecret"));
    }

    #[test]
    fn service_values_marks_stateful() {
        let sv = ServiceValues {
            enabled: true,
            image: ImageRef {
                registry: None,
                repository: "postgres".to_string(),
                tag: "16".to_string(),
                pull_policy: "IfNotPresent".to_string(),
            },
            category: "database".to_string(),
            stateful: true,
            ports: vec![PortSpec {
                name: "postgres".to_string(),
                container_port: 5432,
                protocol: "TCP".to_string(),
            }],
            env: IndexMap::new(),
            env_from_secret: IndexMap::new(),
            agent_env: IndexMap::new(),
            volume_mounts: vec![],
            pvc_size: "10Gi".to_string(),
            command: None,
            healthcheck: HealthcheckSpec {
                command: vec!["pg_isready".to_string()],
                initial_delay_seconds: 30,
                period_seconds: 10,
                timeout_seconds: 5,
                failure_threshold: 5,
            },
            resources: ResourceLimits {
                requests: ResourceSpec {
                    cpu: None,
                    memory: None,
                },
                limits: ResourceSpec {
                    cpu: None,
                    memory: None,
                },
            },
        };
        let yaml = serde_yaml::to_string(&sv).unwrap();
        assert!(yaml.contains("stateful: true"));
        assert!(yaml.contains("pvcSize: 10Gi"));
    }

    #[test]
    fn ingress_values_serializes() {
        let ingress = IngressValues {
            class_name: "nginx".to_string(),
            host: "agent.example.com".to_string(),
        };
        let yaml = serde_yaml::to_string(&ingress).unwrap();
        assert!(yaml.contains("className: nginx"));
        assert!(yaml.contains("host: agent.example.com"));
    }

    #[test]
    fn helm_values_omits_ingress_when_none() {
        let hv = HelmValues {
            project_name: "test".to_string(),
            namespace: "default".to_string(),
            agent: AgentValues {
                agent_type: "claude".to_string(),
                image: ImageRef {
                    registry: None,
                    repository: "test".to_string(),
                    tag: "latest".to_string(),
                    pull_policy: "IfNotPresent".to_string(),
                },
                replicas: 1,
                resources: ResourceLimits {
                    requests: ResourceSpec {
                        cpu: None,
                        memory: None,
                    },
                    limits: ResourceSpec {
                        cpu: None,
                        memory: None,
                    },
                },
                env: IndexMap::new(),
                env_from_secret: vec![],
                volume_mounts: vec![],
                workspace_pvc_size: "10Gi".to_string(),
                workspace_mount_path: "/workspace".to_string(),
                security_context: SecurityContext {
                    run_as_non_root: true,
                    capabilities_add: vec![],
                    capabilities_drop: vec![],
                },
            },
            services: IndexMap::new(),
            mcp: IndexMap::new(),
            network_policy: NetworkPolicyValues {
                enabled: false,
                allowed_cidrs: vec![],
                allowed_domains: vec![],
                allow_dns: true,
                allow_ssh: true,
            },
            secrets: SecretsValues {
                auth_keys: vec![],
                service_credentials: IndexMap::new(),
            },
            ingress: None,
            storage_class: None,
        };
        let yaml = serde_yaml::to_string(&hv).unwrap();
        assert!(!yaml.contains("ingress"));
        assert!(yaml.contains("projectName: test"));
    }
}
