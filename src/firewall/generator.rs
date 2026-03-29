use crate::config::project::{AgentType, ProjectConfig};

use super::domains;

/// Returns true if `s` looks like a valid domain name (only [a-zA-Z0-9.-], has a dot, doesn't start/end with dot/hyphen).
fn is_valid_domain(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 253
        && s.contains('.')
        && !s.starts_with('.')
        && !s.starts_with('-')
        && !s.ends_with('.')
        && !s.ends_with('-')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// Returns true if `s` is a valid IPv4 CIDR (e.g. `10.0.0.0/8`).
/// Validates each octet is 0-255 and prefix length is 0-32.
fn is_valid_cidr(s: &str) -> bool {
    let Some((ip_part, prefix_part)) = s.split_once('/') else {
        return false;
    };
    let Ok(prefix) = prefix_part.parse::<u8>() else {
        return false;
    };
    if prefix > 32 {
        return false;
    }
    let octets: Vec<&str> = ip_part.split('.').collect();
    if octets.len() != 4 {
        return false;
    }
    octets.iter().all(|o| o.parse::<u8>().is_ok())
}

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
        if !is_valid_domain(d) {
            eprintln!("warning: skipping invalid domain in firewall config: {d}");
            continue;
        }
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

    // Bash function to validate IPv4 addresses (each octet 0-255)
    script.push_str("# Validate that a string is a well-formed IPv4 address\n");
    script.push_str("is_ipv4() {\n");
    script.push_str("  local IFS='.'\n");
    script.push_str("  read -ra octets <<< \"$1\"\n");
    script.push_str("  [[ ${#octets[@]} -eq 4 ]] || return 1\n");
    script.push_str("  for o in \"${octets[@]}\"; do\n");
    script.push_str(
        "    [[ \"$o\" =~ ^[0-9]+$ ]] && [ \"$o\" -ge 0 ] && [ \"$o\" -le 255 ] || return 1\n",
    );
    script.push_str("  done\n");
    script.push_str("}\n\n");

    script.push_str("for domain in \"${DOMAINS[@]}\"; do\n");
    script.push_str("  ips=$(dig +short \"$domain\" A 2>/dev/null || true)\n");
    script.push_str("  for ip in $ips; do\n");
    script.push_str("    if is_ipv4 \"$ip\"; then\n");
    script.push_str("      ipset add allowed_ips \"$ip\" -exist\n");
    script.push_str("    fi\n");
    script.push_str("  done\n");
    script.push_str("done\n\n");

    // Set default policy to DROP before flushing to prevent open window
    script.push_str("# Set default policy to DROP before flushing to prevent open window\n");
    script.push_str("iptables -P OUTPUT DROP\n\n");

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
            if !is_valid_cidr(cidr) {
                eprintln!("warning: skipping invalid CIDR in firewall config: {cidr}");
                continue;
            }
            script.push_str(&format!("iptables -A OUTPUT -d {cidr} -j ACCEPT\n"));
        }
        script.push('\n');
    }

    // Allow ipset destinations (HTTPS)
    script.push_str("# Allow HTTPS to resolved domains\n");
    script.push_str(
        "iptables -A OUTPUT -p tcp --dport 443 -m set --match-set allowed_ips dst -j ACCEPT\n",
    );
    script.push_str(
        "iptables -A OUTPUT -p tcp --dport 80 -m set --match-set allowed_ips dst -j ACCEPT\n\n",
    );

    // Allow Docker network (compose services communicate via internal network)
    script.push_str("# Allow Docker internal network (compose services)\n");
    script.push_str("iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT\n");
    script.push_str("iptables -A OUTPUT -d 172.16.0.0/12 -j ACCEPT\n");
    script.push_str("iptables -A OUTPUT -d 192.168.0.0/16 -j ACCEPT\n\n");

    // Default deny
    script.push_str("# Default deny all other outbound traffic\n");
    script.push_str("iptables -A OUTPUT -j DROP\n\n");

    // Restore ACCEPT policy (explicit DROP rule handles blocking)
    script.push_str("# Restore ACCEPT policy (explicit DROP rule handles blocking)\n");
    script.push_str("iptables -P OUTPUT ACCEPT\n\n");

    // Block IPv6 egress
    script.push_str("# Block IPv6 egress (prevent bypass of IPv4 firewall)\n");
    script.push_str("if command -v ip6tables &>/dev/null; then\n");
    script.push_str("  ip6tables -P OUTPUT DROP\n");
    script.push_str("  ip6tables -F OUTPUT\n");
    script.push_str("  ip6tables -A OUTPUT -o lo -j ACCEPT\n");
    script.push_str("  ip6tables -A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT\n");
    script.push_str("  ip6tables -A OUTPUT -j DROP\n");
    script.push_str("  ip6tables -P OUTPUT ACCEPT\n");
    script.push_str("fi\n\n");

    script.push_str(
        "echo \"Firewall configured: $(ipset list allowed_ips | grep -c '^[0-9]') IPs allowed\"\n",
    );

    script
}
