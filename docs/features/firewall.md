# Firewall

Generates iptables bash scripts to restrict container network access using a default-deny policy.

## Scope

**In scope:**
- iptables rule script generation from config
- Agent-specific default domain allowlists (Claude vs Codex)
- Domain and CIDR validation
- DNS resolution of allowed domains to IPs
- IPv6 blocking
- SSH and DNS traffic toggles
- Docker internal network allowlisting

**Not in scope:**
- Firewall execution (the script is generated, not run by cc-container)
- nftables or non-iptables firewalls
- Per-service network policies
- Inbound (INPUT chain) filtering

## Data/Control Flow

```
firewall::generator::generate(config: &ProjectConfig)
    │
    ├── 1. Script header
    │     └── Shebang, set -euo pipefail, root check
    │
    ├── 2. IPSet creation
    │     └── ipset create allowed_ips hash:ip -exist
    │         ipset flush allowed_ips
    │
    ├── 3. Domain collection & deduplication
    │     ├── Agent-specific defaults:
    │     │     ├── Claude: claude_defaults() (12+ domains)
    │     │     ├── Codex: codex_defaults() (10+ domains)
    │     │     └── Both: union of claude + codex defaults
    │     ├── User-configured allowed_domains (validated)
    │     └── Deduplicate all domains
    │
    ├── 4. Domain resolution loop (in generated script)
    │     ├── is_ipv4() helper function
    │     ├── For each domain: dig +short "$domain" A
    │     └── Add resolved IPs to ipset
    │
    ├── 5. iptables rules (in order):
    │     ├── Flush OUTPUT chain
    │     ├── Allow loopback (-o lo -j ACCEPT)
    │     ├── Allow established/related (-m state -j ACCEPT)
    │     ├── [if allow_dns] Allow UDP+TCP port 53
    │     ├── [if allow_ssh] Allow TCP port 22
    │     ├── [for each CIDR] Allow destination CIDR
    │     ├── Allow HTTP/HTTPS to ipset (--match-set allowed_ips dst)
    │     ├── Allow Docker internals: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
    │     └── Default deny: -j DROP
    │
    ├── 6. IPv6 blocking (if ip6tables available)
    │     ├── Flush OUTPUT chain
    │     ├── Allow loopback
    │     ├── Allow established/related
    │     └── Default deny: -j DROP
    │
    └── 7. Summary echo (allowed IP count)
```

## Default Domain Allowlists

### Claude Defaults

| Domain | Purpose |
|--------|---------|
| `api.anthropic.com` | Claude API |
| `statsig.anthropic.com` | Anthropic analytics |
| `sentry.io` | Error reporting |
| `registry.npmjs.org` | npm packages |
| `pypi.org` | Python packages |
| `files.pythonhosted.org` | Python package files |
| `crates.io` | Rust packages |
| `static.crates.io` | Rust package downloads |
| `github.com` | Git operations |
| `api.github.com` | GitHub API |
| `raw.githubusercontent.com` | GitHub raw content |
| `objects.githubusercontent.com` | GitHub objects |

### Codex Defaults

Same as Claude except:
- Uses `api.openai.com` instead of `api.anthropic.com`
- No `statsig.anthropic.com` or `sentry.io`

### Both Mode

Union of Claude and Codex defaults (deduplicated).

## Validation

### Domain Validation (`is_valid_domain`)

- Length <= 253 characters
- Contains at least one dot
- No leading or trailing dots or hyphens
- Only alphanumeric characters, dots, and hyphens

### CIDR Validation (`is_valid_cidr`)

- Format: `a.b.c.d/prefix`
- Each octet: 0-255
- Prefix length: 0-32

Invalid domains and CIDRs are silently skipped during generation (validated at config level).

## Config

```toml
[firewall]
enabled = true
allowed_domains = ["custom.api.com", "registry.npmjs.org"]
allowed_cidrs = ["10.0.0.0/8", "192.168.1.0/24"]
allow_ssh = true    # default: true
allow_dns = true    # default: true
```

## Public API

| Function | Signature | Description |
|----------|-----------|-------------|
| `generator::generate()` | `(config: &ProjectConfig) -> String` | Generate complete iptables bash script |
| `domains::claude_defaults()` | `-> Vec<&'static str>` | Claude agent default domains |
| `domains::codex_defaults()` | `-> Vec<&'static str>` | Codex agent default domains |
| `domains::is_valid_domain()` | `(s: &str) -> bool` | Validate domain format |
| `domains::is_valid_cidr()` | `(s: &str) -> bool` | Validate IPv4 CIDR format |

## Implementation Files

| File | Role | Key Exports |
|------|------|-------------|
| `src/firewall/mod.rs` | Module exports | — |
| `src/firewall/generator.rs` | iptables script generation | `generate()` |
| `src/firewall/domains.rs` | Default domains, validation | `claude_defaults()`, `codex_defaults()`, `is_valid_domain()`, `is_valid_cidr()` |

## Invariants and Constraints

1. **Default-deny policy**: The generated script always ends with `iptables -A OUTPUT -j DROP`.
2. **Loopback always allowed**: `-o lo -j ACCEPT` is unconditional.
3. **Established connections always allowed**: `-m state --state ESTABLISHED,RELATED -j ACCEPT` is unconditional.
4. **Docker internals always allowed**: Private subnets (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) are always permitted for inter-container communication.
5. **IPv6 blocked entirely**: If `ip6tables` is available, all IPv6 egress is blocked except loopback and established.
6. **Script must run as root**: The generated script includes a root check that exits with error if not root.
7. **Requires NET_ADMIN capability**: The container must have `NET_ADMIN` in `[runtime].cap_add` for iptables to work. Config validation warns if this is missing.
8. **Domain resolution happens at runtime**: `dig` resolves domains when the script runs, not during generation. IPs may change between script generation and execution.
9. **Deduplication applied**: Both default and user-configured domains are deduplicated before inclusion in the script.
10. **Rule ordering matters**: iptables rules are applied in order — specific allows before the final DROP.
