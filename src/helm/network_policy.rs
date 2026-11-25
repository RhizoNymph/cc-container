use crate::config::project::{AgentType, ProjectConfig};
use crate::firewall::domains;
use crate::helm::types::NetworkPolicyValues;

/// Build the NetworkPolicyValues for the Helm chart from project config.
///
/// Maps firewall configuration to Kubernetes NetworkPolicy annotations and
/// allowed CIDR blocks. Domain-based filtering is stored as annotations since
/// standard K8s NetworkPolicy does not support FQDN-based egress rules.
pub fn build(config: &ProjectConfig) -> NetworkPolicyValues {
    let enabled = config.firewall.enabled;

    // Collect allowed domains (same logic as firewall/generator.rs)
    let mut allowed_domains: Vec<String> = Vec::new();

    match config.agent.agent_type {
        AgentType::Claude => {
            allowed_domains.extend(domains::claude_defaults().into_iter().map(String::from));
        }
        AgentType::Codex => {
            allowed_domains.extend(domains::codex_defaults().into_iter().map(String::from));
        }
        AgentType::Both => {
            allowed_domains.extend(domains::claude_defaults().into_iter().map(String::from));
            for d in domains::codex_defaults() {
                let s = d.to_string();
                if !allowed_domains.contains(&s) {
                    allowed_domains.push(s);
                }
            }
        }
    }

    // Add user-configured domains (dedup)
    for d in &config.firewall.allowed_domains {
        if !allowed_domains.contains(d) {
            allowed_domains.push(d.clone());
        }
    }

    let allowed_cidrs = config.firewall.allowed_cidrs.clone();
    let allow_dns = config.firewall.allow_dns;
    let allow_ssh = config.firewall.allow_ssh;

    NetworkPolicyValues {
        enabled,
        allowed_cidrs,
        allowed_domains,
        allow_dns,
        allow_ssh,
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

    // -- Enabled/disabled --

    #[test]
    fn build_disabled_by_default() {
        let config = minimal_config();
        let np = build(&config);
        assert!(!np.enabled);
    }

    #[test]
    fn build_enabled_when_firewall_enabled() {
        let mut config = minimal_config();
        config.firewall.enabled = true;
        let np = build(&config);
        assert!(np.enabled);
    }

    // -- Claude defaults --

    #[test]
    fn build_claude_includes_anthropic_domain() {
        let config = minimal_config();
        let np = build(&config);
        assert!(
            np.allowed_domains
                .contains(&"api.anthropic.com".to_string())
        );
    }

    #[test]
    fn build_claude_includes_github_domain() {
        let config = minimal_config();
        let np = build(&config);
        assert!(np.allowed_domains.contains(&"github.com".to_string()));
    }

    #[test]
    fn build_claude_does_not_include_openai_domain() {
        let config = minimal_config();
        let np = build(&config);
        assert!(!np.allowed_domains.contains(&"api.openai.com".to_string()));
    }

    // -- Codex defaults --

    #[test]
    fn build_codex_includes_openai_domain() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;
        let np = build(&config);
        assert!(np.allowed_domains.contains(&"api.openai.com".to_string()));
    }

    #[test]
    fn build_codex_does_not_include_anthropic_domain() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;
        let np = build(&config);
        assert!(
            !np.allowed_domains
                .contains(&"api.anthropic.com".to_string())
        );
    }

    // -- Both defaults --

    #[test]
    fn build_both_includes_anthropic_and_openai() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;
        let np = build(&config);
        assert!(
            np.allowed_domains
                .contains(&"api.anthropic.com".to_string())
        );
        assert!(np.allowed_domains.contains(&"api.openai.com".to_string()));
    }

    #[test]
    fn build_both_deduplicates_shared_domains() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;
        let np = build(&config);

        // github.com is in both claude and codex defaults
        let github_count = np
            .allowed_domains
            .iter()
            .filter(|d| d.as_str() == "github.com")
            .count();
        assert_eq!(github_count, 1, "github.com should appear only once");
    }

    // -- User-configured domains --

    #[test]
    fn build_includes_user_domains() {
        let mut config = minimal_config();
        config.firewall.allowed_domains = vec!["custom.example.com".to_string()];
        let np = build(&config);
        assert!(
            np.allowed_domains
                .contains(&"custom.example.com".to_string())
        );
    }

    #[test]
    fn build_deduplicates_user_domains() {
        let mut config = minimal_config();
        config.firewall.allowed_domains = vec!["github.com".to_string()];
        let np = build(&config);

        let github_count = np
            .allowed_domains
            .iter()
            .filter(|d| d.as_str() == "github.com")
            .count();
        assert_eq!(github_count, 1);
    }

    // -- CIDRs --

    #[test]
    fn build_maps_cidrs() {
        let mut config = minimal_config();
        config.firewall.allowed_cidrs = vec!["10.0.0.0/8".to_string(), "172.16.0.0/12".to_string()];
        let np = build(&config);
        assert_eq!(np.allowed_cidrs.len(), 2);
        assert!(np.allowed_cidrs.contains(&"10.0.0.0/8".to_string()));
        assert!(np.allowed_cidrs.contains(&"172.16.0.0/12".to_string()));
    }

    #[test]
    fn build_empty_cidrs_by_default() {
        let config = minimal_config();
        let np = build(&config);
        assert!(np.allowed_cidrs.is_empty());
    }

    // -- DNS and SSH flags --

    #[test]
    fn build_dns_enabled_by_default() {
        let config = minimal_config();
        let np = build(&config);
        assert!(np.allow_dns);
    }

    #[test]
    fn build_ssh_enabled_by_default() {
        let config = minimal_config();
        let np = build(&config);
        assert!(np.allow_ssh);
    }

    #[test]
    fn build_dns_disabled() {
        let mut config = minimal_config();
        config.firewall.allow_dns = false;
        let np = build(&config);
        assert!(!np.allow_dns);
    }

    #[test]
    fn build_ssh_disabled() {
        let mut config = minimal_config();
        config.firewall.allow_ssh = false;
        let np = build(&config);
        assert!(!np.allow_ssh);
    }
}
