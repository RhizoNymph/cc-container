# Firewall

Generates iptables rules to restrict container network access.

## How It Works

When `firewall.enabled = true` in config, generates a shell script with iptables rules that allow only specified domains (resolved to IPs) and CIDR ranges. Supports flags for allowing SSH and DNS traffic.

## Config

```toml
[firewall]
enabled = true
allowed_domains = ["api.anthropic.com", "registry.npmjs.org"]
allowed_cidrs = ["10.0.0.0/8"]
allow_ssh = true
allow_dns = true
```

## Implementation Files

- `src/firewall/generator.rs` — Generates iptables rule scripts
- `src/firewall/domains.rs` — Domain and CIDR validation/management
