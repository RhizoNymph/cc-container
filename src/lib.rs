//! Public library API for generating containerized AI coding-agent project files.
//!
//! Prefer the `generate` facade for stable library integration. The lower-level
//! modules remain public for existing callers but should be treated as unstable
//! implementation details until they are explicitly documented as stable.

/// Authentication environment and volume helpers.
pub mod auth;
/// CLI command implementation. Unstable internal API.
pub mod cli;
/// Docker Compose generation internals. Unstable internal API.
pub mod compose;
/// Project and user configuration types/loaders.
pub mod config;
/// Crate error types.
pub mod error;
/// Firewall domain defaults and shell script generation internals.
pub mod firewall;
/// Stable generation facade for library callers.
pub mod generate;
/// Helm chart generation internals. Unstable internal API.
pub mod helm;
/// MCP config generation internals. Unstable internal API.
pub mod mcp;
/// Dockerfile module registry/rendering internals. Unstable internal API.
pub mod module;
/// Interactive init wizard internals. Unstable internal API.
pub mod wizard;

// Tests for the firewall feature.
// NOTE: Rust edition 2024 does not discover #[cfg(test)] blocks in sub-module
// files of library crates. All tests are collected here where the test harness
// can find them.
#[cfg(test)]
mod firewall_domains_tests {
    use crate::firewall::domains;

    // ── claude_defaults ──────────────────────────────────────────────

    #[test]
    fn claude_defaults_is_non_empty() {
        let defaults = domains::claude_defaults();
        assert!(!defaults.is_empty(), "claude defaults should not be empty");
    }

    #[test]
    fn claude_defaults_contains_anthropic_api() {
        let defaults = domains::claude_defaults();
        assert!(
            defaults.contains(&"api.anthropic.com"),
            "claude defaults must include api.anthropic.com"
        );
    }

    #[test]
    fn claude_defaults_contains_github() {
        let defaults = domains::claude_defaults();
        assert!(defaults.contains(&"github.com"));
        assert!(defaults.contains(&"api.github.com"));
    }

    #[test]
    fn claude_defaults_contains_npm_registry() {
        let defaults = domains::claude_defaults();
        assert!(defaults.contains(&"registry.npmjs.org"));
    }

    #[test]
    fn claude_defaults_contains_pypi() {
        let defaults = domains::claude_defaults();
        assert!(defaults.contains(&"pypi.org"));
        assert!(defaults.contains(&"files.pythonhosted.org"));
    }

    #[test]
    fn claude_defaults_contains_crates_io() {
        let defaults = domains::claude_defaults();
        assert!(defaults.contains(&"crates.io"));
        assert!(defaults.contains(&"static.crates.io"));
    }

    #[test]
    fn claude_defaults_has_no_duplicates() {
        let defaults = domains::claude_defaults();
        let mut seen = std::collections::HashSet::new();
        for d in &defaults {
            assert!(seen.insert(d), "duplicate domain in claude defaults: {d}");
        }
    }

    // ── codex_defaults ───────────────────────────────────────────────

    #[test]
    fn codex_defaults_is_non_empty() {
        let defaults = domains::codex_defaults();
        assert!(!defaults.is_empty(), "codex defaults should not be empty");
    }

    #[test]
    fn codex_defaults_contains_openai_api() {
        let defaults = domains::codex_defaults();
        assert!(
            defaults.contains(&"api.openai.com"),
            "codex defaults must include api.openai.com"
        );
    }

    #[test]
    fn codex_defaults_does_not_contain_anthropic() {
        let defaults = domains::codex_defaults();
        assert!(
            !defaults.contains(&"api.anthropic.com"),
            "codex defaults should not include Anthropic API"
        );
    }

    #[test]
    fn codex_defaults_contains_github() {
        let defaults = domains::codex_defaults();
        assert!(defaults.contains(&"github.com"));
        assert!(defaults.contains(&"api.github.com"));
    }

    #[test]
    fn codex_defaults_has_no_duplicates() {
        let defaults = domains::codex_defaults();
        let mut seen = std::collections::HashSet::new();
        for d in &defaults {
            assert!(seen.insert(d), "duplicate domain in codex defaults: {d}");
        }
    }

    // ── shared domains between claude and codex ──────────────────────

    #[test]
    fn claude_and_codex_share_common_infrastructure_domains() {
        let claude = domains::claude_defaults();
        let codex = domains::codex_defaults();

        for shared in &[
            "github.com",
            "api.github.com",
            "registry.npmjs.org",
            "pypi.org",
            "crates.io",
        ] {
            assert!(
                claude.contains(shared),
                "claude defaults missing shared domain: {shared}"
            );
            assert!(
                codex.contains(shared),
                "codex defaults missing shared domain: {shared}"
            );
        }
    }

    // ── all defaults are syntactically valid domains ─────────────────

    #[test]
    fn all_claude_defaults_look_like_valid_domains() {
        for d in domains::claude_defaults() {
            assert!(d.contains('.'), "domain should contain a dot: {d}");
            assert!(!d.starts_with('.'), "domain should not start with dot: {d}");
            assert!(!d.ends_with('.'), "domain should not end with dot: {d}");
            assert!(
                d.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'),
                "domain contains invalid chars: {d}"
            );
        }
    }

    #[test]
    fn all_codex_defaults_look_like_valid_domains() {
        for d in domains::codex_defaults() {
            assert!(d.contains('.'), "domain should contain a dot: {d}");
            assert!(!d.starts_with('.'), "domain should not start with dot: {d}");
            assert!(!d.ends_with('.'), "domain should not end with dot: {d}");
            assert!(
                d.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'),
                "domain contains invalid chars: {d}"
            );
        }
    }
}

#[cfg(test)]
mod firewall_generator_tests {
    use crate::config::project::*;
    use crate::firewall::generator;
    use indexmap::IndexMap;

    /// Helper: build a minimal ProjectConfig with the given firewall settings.
    fn make_config(agent_type: AgentType, firewall: FirewallConfig) -> ProjectConfig {
        ProjectConfig {
            project: ProjectMeta {
                name: "test-project".to_string(),
                description: None,
            },
            agent: AgentConfig {
                agent_type,
                claude_version: "latest".to_string(),
                codex_version: "latest".to_string(),
            },
            image: ImageConfig::default(),
            modules: IndexMap::new(),
            auth: AuthConfig::default(),
            firewall,
            workspace: WorkspaceConfig::default(),
            volumes: IndexMap::new(),
            environment: EnvironmentConfig::default(),
            services: IndexMap::new(),
            mcp: IndexMap::new(),
            runtime: RuntimeConfig::default(),
            helm: HelmConfig::default(),
        }
    }

    fn default_firewall() -> FirewallConfig {
        FirewallConfig {
            enabled: true,
            allowed_domains: Vec::new(),
            allowed_cidrs: Vec::new(),
            allow_ssh: true,
            allow_dns: true,
        }
    }

    // ── Script structure / boilerplate ────────────────────────────────

    #[test]
    fn script_starts_with_shebang() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.starts_with("#!/usr/bin/env bash\n"));
    }

    #[test]
    fn script_has_set_euo_pipefail() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("set -euo pipefail"));
    }

    #[test]
    fn script_checks_for_root() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("if [ \"$(id -u)\" -ne 0 ]"));
        assert!(script.contains("must be run as root"));
    }

    #[test]
    fn script_creates_ipset() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("ipset create allowed_ips hash:ip -exist"));
        assert!(script.contains("ipset flush allowed_ips"));
    }

    #[test]
    fn script_flushes_output_chain() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("iptables -F OUTPUT"));
    }

    #[test]
    fn script_allows_loopback() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("iptables -A OUTPUT -o lo -j ACCEPT"));
    }

    #[test]
    fn script_allows_established_connections() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(
            script.contains("iptables -A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT")
        );
    }

    #[test]
    fn script_allows_https_and_http_via_ipset() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("--dport 443 -m set --match-set allowed_ips dst -j ACCEPT"));
        assert!(script.contains("--dport 80 -m set --match-set allowed_ips dst -j ACCEPT"));
    }

    #[test]
    fn script_allows_docker_internal_networks() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT"));
        assert!(script.contains("iptables -A OUTPUT -d 172.16.0.0/12 -j ACCEPT"));
        assert!(script.contains("iptables -A OUTPUT -d 192.168.0.0/16 -j ACCEPT"));
    }

    #[test]
    fn script_ends_with_default_deny() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("iptables -A OUTPUT -j DROP"));
    }

    #[test]
    fn script_blocks_ipv6() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("ip6tables -F OUTPUT"));
        assert!(script.contains("ip6tables -A OUTPUT -j DROP"));
    }

    #[test]
    fn script_prints_summary_at_end() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("echo \"Firewall configured:"));
    }

    // ── DNS flag ─────────────────────────────────────────────────────

    #[test]
    fn dns_enabled_adds_udp_and_tcp_rules() {
        let fw = FirewallConfig {
            allow_dns: true,
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("iptables -A OUTPUT -p udp --dport 53 -j ACCEPT"));
        assert!(script.contains("iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT"));
    }

    #[test]
    fn dns_disabled_omits_port_53_rules() {
        let fw = FirewallConfig {
            allow_dns: false,
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("--dport 53"));
    }

    // ── SSH flag ─────────────────────────────────────────────────────

    #[test]
    fn ssh_enabled_adds_port_22_rule() {
        let fw = FirewallConfig {
            allow_ssh: true,
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("iptables -A OUTPUT -p tcp --dport 22 -j ACCEPT"));
    }

    #[test]
    fn ssh_disabled_omits_port_22_rule() {
        let fw = FirewallConfig {
            allow_ssh: false,
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("--dport 22"));
    }

    // ── Both SSH and DNS disabled ────────────────────────────────────

    #[test]
    fn all_flags_off_omits_ssh_and_dns() {
        let fw = FirewallConfig {
            allow_ssh: false,
            allow_dns: false,
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("--dport 22"));
        assert!(!script.contains("--dport 53"));
    }

    // ── Agent-specific default domains ───────────────────────────────

    #[test]
    fn claude_agent_includes_anthropic_domain() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("\"api.anthropic.com\""));
    }

    #[test]
    fn claude_agent_does_not_include_openai_domain() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(!script.contains("api.openai.com"));
    }

    #[test]
    fn codex_agent_includes_openai_domain() {
        let cfg = make_config(AgentType::Codex, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("\"api.openai.com\""));
    }

    #[test]
    fn codex_agent_does_not_include_anthropic_domain() {
        let cfg = make_config(AgentType::Codex, default_firewall());
        let script = generator::generate(&cfg);
        assert!(!script.contains("api.anthropic.com"));
    }

    #[test]
    fn both_agent_includes_claude_and_codex_domains() {
        let cfg = make_config(AgentType::Both, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("\"api.anthropic.com\""));
        assert!(script.contains("\"api.openai.com\""));
    }

    #[test]
    fn both_agent_deduplicates_shared_domains() {
        let cfg = make_config(AgentType::Both, default_firewall());
        let script = generator::generate(&cfg);
        let count = script.matches("\"github.com\"").count();
        assert_eq!(
            count, 1,
            "github.com should appear exactly once, found {count}"
        );
    }

    // ── User-configured domains ──────────────────────────────────────

    #[test]
    fn user_domains_appear_in_script() {
        let fw = FirewallConfig {
            allowed_domains: vec![
                "custom.example.com".to_string(),
                "another.example.org".to_string(),
            ],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("\"custom.example.com\""));
        assert!(script.contains("\"another.example.org\""));
    }

    #[test]
    fn duplicate_user_domain_not_repeated() {
        let fw = FirewallConfig {
            allowed_domains: vec!["api.anthropic.com".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        let count = script.matches("\"api.anthropic.com\"").count();
        assert_eq!(count, 1, "duplicate domain should only appear once");
    }

    #[test]
    fn invalid_domain_is_skipped() {
        let fw = FirewallConfig {
            allowed_domains: vec![
                "valid.example.com".to_string(),
                ".invalid-leading-dot.com".to_string(),
                "no-dot".to_string(),
                "valid2.example.com".to_string(),
            ],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("\"valid.example.com\""));
        assert!(script.contains("\"valid2.example.com\""));
        assert!(!script.contains("\".invalid-leading-dot.com\""));
        assert!(!script.contains("\"no-dot\""));
    }

    #[test]
    fn empty_user_domains_still_has_agent_defaults() {
        let fw = FirewallConfig {
            allowed_domains: Vec::new(),
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("\"api.anthropic.com\""));
    }

    // ── CIDRs ────────────────────────────────────────────────────────

    #[test]
    fn valid_cidrs_produce_iptables_rules() {
        let fw = FirewallConfig {
            allowed_cidrs: vec!["10.0.0.0/8".to_string(), "192.168.1.0/24".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT"));
        assert!(script.contains("iptables -A OUTPUT -d 192.168.1.0/24 -j ACCEPT"));
    }

    #[test]
    fn invalid_cidr_is_skipped() {
        let fw = FirewallConfig {
            allowed_cidrs: vec![
                "10.0.0.0/8".to_string(),
                "not-a-cidr".to_string(),
                "172.16.0.0/12".to_string(),
            ],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT"));
        assert!(script.contains("iptables -A OUTPUT -d 172.16.0.0/12 -j ACCEPT"));
        assert!(!script.contains("-d not-a-cidr"));
    }

    #[test]
    fn empty_cidrs_omits_cidr_section_comment() {
        let fw = FirewallConfig {
            allowed_cidrs: Vec::new(),
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("# Allow additional CIDR ranges"));
    }

    #[test]
    fn non_empty_cidrs_includes_section_comment() {
        let fw = FirewallConfig {
            allowed_cidrs: vec!["10.0.0.0/8".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("# Allow additional CIDR ranges"));
    }

    // ── Domain validation (via generate — invalid domains skipped) ───

    #[test]
    fn domain_with_underscore_is_skipped() {
        let fw = FirewallConfig {
            allowed_domains: vec!["has_underscore.com".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("has_underscore.com"));
    }

    #[test]
    fn domain_with_space_is_skipped() {
        let fw = FirewallConfig {
            allowed_domains: vec!["has space.com".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("has space.com"));
    }

    #[test]
    fn domain_with_trailing_dot_is_skipped() {
        let fw = FirewallConfig {
            allowed_domains: vec!["trailing.dot.com.".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("\"trailing.dot.com.\""));
    }

    #[test]
    fn domain_with_leading_hyphen_is_skipped() {
        let fw = FirewallConfig {
            allowed_domains: vec!["-leading-hyphen.com".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("-leading-hyphen.com"));
    }

    #[test]
    fn domain_without_dot_is_skipped() {
        let fw = FirewallConfig {
            allowed_domains: vec!["nodot".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("\"nodot\""));
    }

    // ── Domain validation (tested indirectly via generate) ─────────

    #[test]
    fn valid_domain_formats_accepted_by_generate() {
        // Domains that pass validation should appear in the DOMAINS array
        let valid_domains = vec![
            "example.com".to_string(),
            "sub.example.com".to_string(),
            "a.b.c.d.example.com".to_string(),
            "my-domain.co.uk".to_string(),
            "x.io".to_string(),
        ];
        let fw = FirewallConfig {
            allowed_domains: valid_domains.clone(),
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        for d in &valid_domains {
            assert!(
                script.contains(&format!("\"{d}\"")),
                "valid domain {d} should appear in script"
            );
        }
    }

    #[test]
    fn invalid_domain_formats_rejected_by_generate() {
        // Each of these invalid domains should be skipped by the generator
        let invalid_domains = vec![
            "".to_string(),
            "no-dot".to_string(),
            ".leading-dot.com".to_string(),
            "trailing-dot.com.".to_string(),
            "-leading-hyphen.com".to_string(),
            "trailing-hyphen.com-".to_string(),
            "has space.com".to_string(),
            "has_underscore.com".to_string(),
            "has@special.com".to_string(),
        ];
        let fw = FirewallConfig {
            allowed_domains: invalid_domains.clone(),
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        for d in &invalid_domains {
            if !d.is_empty() {
                assert!(
                    !script.contains(&format!("\"{d}\"")),
                    "invalid domain {d} should NOT appear in script"
                );
            }
        }
    }

    #[test]
    fn domain_over_253_chars_is_rejected() {
        let long_label = "a".repeat(250);
        let too_long = format!("{long_label}.com");
        assert!(too_long.len() > 253);
        let fw = FirewallConfig {
            allowed_domains: vec![too_long.clone()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(
            !script.contains(&format!("\"{too_long}\"")),
            "domain over 253 chars should be rejected"
        );
    }

    #[test]
    fn domain_exactly_253_chars_is_accepted() {
        let long_label = "a".repeat(250);
        let exactly_253 = format!("{long_label}.co");
        assert_eq!(exactly_253.len(), 253);
        let fw = FirewallConfig {
            allowed_domains: vec![exactly_253.clone()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(
            script.contains(&format!("\"{exactly_253}\"")),
            "domain exactly 253 chars should be accepted"
        );
    }

    // ── CIDR validation (tested indirectly via generate) ─────────────

    #[test]
    fn valid_cidr_formats_accepted_by_generate() {
        let valid_cidrs = vec![
            "10.0.0.0/8".to_string(),
            "192.168.1.0/24".to_string(),
            "172.16.0.0/12".to_string(),
            "0.0.0.0/0".to_string(),
        ];
        let fw = FirewallConfig {
            allowed_cidrs: valid_cidrs.clone(),
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        for c in &valid_cidrs {
            assert!(
                script.contains(&format!("iptables -A OUTPUT -d {c} -j ACCEPT")),
                "valid CIDR {c} should produce an iptables rule"
            );
        }
    }

    #[test]
    fn invalid_cidr_formats_rejected_by_generate() {
        let invalid_cidrs = vec![
            "10.0.0.0".to_string(),         // no slash
            "not-a-cidr".to_string(),       // letters + hyphens
            "abc/def".to_string(),          // letters
            "10.0.0.0/8 extra".to_string(), // space
        ];
        let fw = FirewallConfig {
            allowed_cidrs: invalid_cidrs.clone(),
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        for c in &invalid_cidrs {
            assert!(
                !script.contains(&format!("iptables -A OUTPUT -d {c} -j ACCEPT")),
                "invalid CIDR {c} should NOT produce an iptables rule"
            );
        }
    }

    #[test]
    fn ipv6_cidr_rejected_by_generate() {
        let fw = FirewallConfig {
            allowed_cidrs: vec!["fe80::/10".to_string(), "::1/128".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("-d fe80::/10"));
        assert!(!script.contains("-d ::1/128"));
    }

    // ── Full integration: domains + CIDRs + flags all together ───────

    #[test]
    fn full_config_with_everything_enabled() {
        let fw = FirewallConfig {
            enabled: true,
            allowed_domains: vec!["custom.dev".to_string()],
            allowed_cidrs: vec!["10.10.0.0/16".to_string()],
            allow_ssh: true,
            allow_dns: true,
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));

        assert!(script.starts_with("#!/usr/bin/env bash\n"));
        assert!(script.contains("--dport 53"));
        assert!(script.contains("--dport 22"));
        assert!(script.contains("\"custom.dev\""));
        assert!(script.contains("-d 10.10.0.0/16"));
        assert!(script.contains("iptables -A OUTPUT -j DROP"));
    }

    #[test]
    fn full_config_with_everything_disabled() {
        let fw = FirewallConfig {
            enabled: true,
            allowed_domains: Vec::new(),
            allowed_cidrs: Vec::new(),
            allow_ssh: false,
            allow_dns: false,
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));

        assert!(!script.contains("--dport 53"));
        assert!(!script.contains("--dport 22"));
        assert!(!script.contains("# Allow additional CIDR ranges"));
        assert!(script.contains("iptables -A OUTPUT -j DROP"));
        assert!(script.contains("\"api.anthropic.com\""));
    }

    // ── Domain resolution loop structure ─────────────────────────────

    #[test]
    fn domain_resolution_loop_uses_dig() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(script.contains("dig +short \"$domain\" A"));
    }

    #[test]
    fn domain_resolution_loop_validates_ip_format() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(
            script.contains("is_ipv4 \"$ip\""),
            "generated script should call is_ipv4 function to validate IPs"
        );
    }

    #[test]
    fn domains_array_is_quoted() {
        let fw = FirewallConfig {
            allowed_domains: vec!["test.example.com".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("  \"test.example.com\""));
    }

    // ── Ordering ─────────────────────────────────────────────────────

    #[test]
    fn dns_rules_appear_before_cidr_rules() {
        let fw = FirewallConfig {
            allowed_cidrs: vec!["10.0.0.0/8".to_string()],
            allow_dns: true,
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        let dns_pos = script.find("--dport 53").unwrap();
        let cidr_pos = script.find("-d 10.0.0.0/8").unwrap();
        assert!(
            dns_pos < cidr_pos,
            "DNS rules should appear before CIDR rules"
        );
    }

    #[test]
    fn ssh_rules_appear_before_cidr_rules() {
        let fw = FirewallConfig {
            allowed_cidrs: vec!["10.0.0.0/8".to_string()],
            allow_ssh: true,
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        let ssh_pos = script.find("--dport 22").unwrap();
        let cidr_pos = script.find("-d 10.0.0.0/8").unwrap();
        assert!(
            ssh_pos < cidr_pos,
            "SSH rules should appear before CIDR rules"
        );
    }

    #[test]
    fn default_deny_is_last_iptables_v4_rule() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        let deny_pos = script.find("iptables -A OUTPUT -j DROP").unwrap();
        let after_deny = &script[deny_pos + "iptables -A OUTPUT -j DROP".len()..];
        assert!(
            !after_deny.contains("iptables -A OUTPUT"),
            "no IPv4 OUTPUT rules should follow the default DROP"
        );
    }

    // ── Multiple CIDRs ──────────────────────────────────────────────

    #[test]
    fn multiple_cidrs_each_get_own_rule() {
        let fw = FirewallConfig {
            allowed_cidrs: vec![
                "10.0.0.0/8".to_string(),
                "172.20.0.0/16".to_string(),
                "192.168.100.0/24".to_string(),
            ],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(script.contains("iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT\n"));
        assert!(script.contains("iptables -A OUTPUT -d 172.20.0.0/16 -j ACCEPT\n"));
        assert!(script.contains("iptables -A OUTPUT -d 192.168.100.0/24 -j ACCEPT\n"));
    }

    // ── All-invalid CIDRs ────────────────────────────────────────────

    #[test]
    fn all_invalid_cidrs_skips_section_gracefully() {
        let fw = FirewallConfig {
            allowed_cidrs: vec!["not-valid".to_string(), "also-bad".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(!script.contains("iptables -A OUTPUT -d not-valid"));
        assert!(!script.contains("iptables -A OUTPUT -d also-bad"));
    }

    // ── Firewall disabled config still generates script ──────────────

    #[test]
    fn firewall_disabled_still_generates_valid_script() {
        let fw = FirewallConfig {
            enabled: false,
            ..default_firewall()
        };
        let cfg = make_config(AgentType::Claude, fw);
        let script = generator::generate(&cfg);
        // generate() produces a script regardless of enabled flag
        // (the caller checks enabled before calling generate)
        assert!(script.starts_with("#!/usr/bin/env bash\n"));
        assert!(script.contains("iptables -A OUTPUT -j DROP"));
    }

    // ── IPv4 validation function in generated script ────────────────

    #[test]
    fn generated_script_contains_is_ipv4_function() {
        let cfg = make_config(AgentType::Claude, default_firewall());
        let script = generator::generate(&cfg);
        assert!(
            script.contains("is_ipv4()"),
            "generated script should contain the is_ipv4 bash function"
        );
    }

    // ── CIDR validation rejects malformed inputs ────────────────────

    #[test]
    fn invalid_cidr_with_bad_octet_rejected() {
        let fw = FirewallConfig {
            allowed_cidrs: vec!["999.0.0.0/8".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(
            !script.contains("999.0.0.0/8"),
            "CIDR with invalid octet 999 should be rejected"
        );
    }

    #[test]
    fn invalid_cidr_with_bad_prefix_rejected() {
        let fw = FirewallConfig {
            allowed_cidrs: vec!["10.0.0.0/33".to_string()],
            ..default_firewall()
        };
        let script = generator::generate(&make_config(AgentType::Claude, fw));
        assert!(
            !script.contains("10.0.0.0/33"),
            "CIDR with prefix length 33 should be rejected"
        );
    }
}
