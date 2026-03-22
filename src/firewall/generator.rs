use crate::config::project::{AgentType, ProjectConfig};

use super::domains;

/// Generate the contents of init-firewall.sh.
pub fn generate(config: &ProjectConfig) -> String {
    let mut domains: Vec<String> = Vec::new();

    // Add agent-specific default domains
    match config.agent.agent_type {
        AgentType::Claude => {
            domains.extend(domains::claude_defaults().into_iter().map(String::from));
        }
        AgentType::Codex => {
            domains.extend(domains::codex_defaults().into_iter().map(String::from));
        }
        AgentType::Both => {
            domains.extend(domains::claude_defaults().into_iter().map(String::from));
            for d in domains::codex_defaults() {
                let s = d.to_string();
                if !domains.contains(&s) {
                    domains.push(s);
                }
            }
        }
    }

    // Add user-configured domains
    for d in &config.firewall.allowed_domains {
        if !domains.contains(d) {
            domains.push(d.clone());
        }
    }

    let cidrs = &config.firewall.allowed_cidrs;
    let allow_dns = config.firewall.allow_dns;
    let allow_ssh = config.firewall.allow_ssh;

    let mut script = String::new();
    script.push_str("#!/usr/bin/env bash\n");
    script.push_str("set -euo pipefail\n\n");
    script.push_str("# Auto-generated firewall script by cc-container\n");
    script.push_str("# Implements a default-deny policy with domain whitelisting.\n\n");

    // Check for root
    script.push_str("if [ \"$(id -u)\" -ne 0 ]; then\n");
    script.push_str("  echo \"Error: init-firewall.sh must be run as root\" >&2\n");
    script.push_str("  exit 1\n");
    script.push_str("fi\n\n");

    // Create ipset for allowed IPs
    script.push_str("# Create ipset for allowed destinations\n");
    script.push_str("ipset create allowed_ips hash:ip -exist\n");
    script.push_str("ipset flush allowed_ips\n\n");

    // Resolve domains and add to ipset
    script.push_str("# Resolve allowed domains and add to ipset\n");
    script.push_str("DOMAINS=(\n");
    for d in &domains {
        script.push_str(&format!("  \"{d}\"\n"));
    }
    script.push_str(")\n\n");

    script.push_str("for domain in \"${DOMAINS[@]}\"; do\n");
    script.push_str("  ips=$(dig +short \"$domain\" A 2>/dev/null || true)\n");
    script.push_str("  for ip in $ips; do\n");
    script.push_str("    if [[ \"$ip\" =~ ^[0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+$ ]]; then\n");
    script.push_str("      ipset add allowed_ips \"$ip\" -exist\n");
    script.push_str("    fi\n");
    script.push_str("  done\n");
    script.push_str("done\n\n");

    // Flush existing rules
    script.push_str("# Flush existing rules\n");
    script.push_str("iptables -F OUTPUT\n\n");

    // Allow loopback
    script.push_str("# Allow loopback\n");
    script.push_str("iptables -A OUTPUT -o lo -j ACCEPT\n\n");

    // Allow established connections
    script.push_str("# Allow established/related connections\n");
    script.push_str("iptables -A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT\n\n");

    // Allow DNS
    if allow_dns {
        script.push_str("# Allow DNS\n");
        script.push_str("iptables -A OUTPUT -p udp --dport 53 -j ACCEPT\n");
        script.push_str("iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT\n\n");
    }

    // Allow SSH
    if allow_ssh {
        script.push_str("# Allow SSH\n");
        script.push_str("iptables -A OUTPUT -p tcp --dport 22 -j ACCEPT\n\n");
    }

    // Allow CIDR ranges
    if !cidrs.is_empty() {
        script.push_str("# Allow additional CIDR ranges\n");
        for cidr in cidrs {
            script.push_str(&format!(
                "iptables -A OUTPUT -d {cidr} -j ACCEPT\n"
            ));
        }
        script.push_str("\n");
    }

    // Allow ipset destinations (HTTPS)
    script.push_str("# Allow HTTPS to resolved domains\n");
    script.push_str("iptables -A OUTPUT -p tcp --dport 443 -m set --match-set allowed_ips dst -j ACCEPT\n");
    script.push_str("iptables -A OUTPUT -p tcp --dport 80 -m set --match-set allowed_ips dst -j ACCEPT\n\n");

    // Allow Docker network (compose services communicate via internal network)
    script.push_str("# Allow Docker internal network (compose services)\n");
    script.push_str("iptables -A OUTPUT -d 172.16.0.0/12 -j ACCEPT\n");
    script.push_str("iptables -A OUTPUT -d 192.168.0.0/16 -j ACCEPT\n\n");

    // Default deny
    script.push_str("# Default deny all other outbound traffic\n");
    script.push_str("iptables -A OUTPUT -j DROP\n\n");

    script.push_str("echo \"Firewall configured: $(ipset list allowed_ips | grep -c 'Members') IPs allowed\"\n");

    script
}
