// Stub module: will be replaced by WS B (feat/helm-services branch).
// Provides network policy value builder for Helm chart generation.

use crate::config::project::ProjectConfig;
use crate::helm::types::NetworkPolicyValues;

/// Build network policy values from config.
///
/// Maps firewall config (domains, CIDRs, SSH/DNS flags) to Helm
/// `NetworkPolicyValues` for Kubernetes NetworkPolicy generation.
pub fn build(config: &ProjectConfig) -> NetworkPolicyValues {
    NetworkPolicyValues {
        enabled: config.firewall.enabled,
        allowed_cidrs: config.firewall.allowed_cidrs.clone(),
        allowed_domains: config.firewall.allowed_domains.clone(),
        allow_dns: config.firewall.allow_dns,
        allow_ssh: config.firewall.allow_ssh,
    }
}
