# Helm Chart Generation

## Scope

**In scope:**
- Generating a complete Helm chart directory from `ProjectConfig`
- Embedded Go-template YAML files for all K8s resource types
- Typed `HelmValues` struct that serializes to `values.yaml`
- Value builder modules for services, agent, network policy, and secrets
- CLI integration via `--only helm` on the `generate` command
- Support for infrastructure services (Deployments and StatefulSets)
- Agent container(s) with workspace PVC and secret-backed env vars
- MCP sidecar deployments
- NetworkPolicy generation from firewall config
- Ingress resource generation (optional)
- Secrets and ConfigMap generation

**Not in scope:**
- Helm chart installation or deployment (`helm install`)
- Chart packaging (`helm package`)
- Chart repository publishing
- Custom Resource Definitions (CRDs)
- Horizontal Pod Autoscaler (HPA) templates
- ServiceAccount / RBAC templates
- Helm hooks or post-install jobs
- Chart dependencies (subcharts)

## Data/Control Flow

```
ProjectConfig
    │
    ├──▶ helm::values::build()
    │       │
    │       ├── helm::service_values::build_service() — for each enabled service
    │       │     Returns (ServiceValues, agent_env: IndexMap<String, String>)
    │       │
    │       ├── DATABASE_URL aggregation logic (same as compose/generator.rs)
    │       │     - 0 DBs: no DATABASE_URL
    │       │     - 1 DB: DATABASE_URL set directly
    │       │     - N DBs: service-specific URLs + DATABASE_URL defaults to first
    │       │
    │       ├── helm::agent_values::build() — agent container values
    │       │     Merges infra_env + user env, sets image from helm config
    │       │
    │       ├── helm::network_policy::build() — maps firewall config
    │       │
    │       ├── helm::secrets::build() — maps auth config to secret keys
    │       │
    │       ├── MCP values — built inline from config.mcp
    │       │
    │       └── Ingress values — built inline from helm config
    │
    │       Result: HelmValues struct
    │
    ├──▶ helm::chart::generate()
    │       │
    │       ├── generate_chart_yaml() — Chart.yaml metadata
    │       │
    │       ├── serde_yaml::to_string(&values) — values.yaml
    │       │
    │       └── helm::templates::all_templates() — 11 embedded template files
    │             Each is (filename, content) via include_str!
    │
    │       Result: HelmChart { files: IndexMap<String, String> }
    │
    └──▶ cli::generate::generate_helm()
            │
            └── Writes files to output_dir/chart/<project_name>/
                or prints to stdout in dry-run mode
```

## Files

| File | Part of Feature | Key Exports/Interfaces |
|------|----------------|----------------------|
| `src/helm/mod.rs` | Module declarations | Re-exports all submodules |
| `src/helm/types.rs` | Type definitions | `HelmValues`, `ServiceValues`, `AgentValues`, `NetworkPolicyValues`, `SecretsValues`, `McpValues`, `IngressValues`, `ImageRef`, `PortSpec`, `VolumeMount`, `VolumeDefinition`, `HealthcheckSpec`, `SecurityContext`, `ResourceLimits`, `ResourceSpec`, `SecretKeyRef` |
| `src/helm/chart.rs` | Top-level generator | `HelmChart` struct, `generate(config) -> Result<HelmChart>` |
| `src/helm/templates.rs` | Template embedding | `all_templates() -> Vec<(&str, &str)>` |
| `src/helm/values.rs` | Value orchestrator | `build(config) -> Result<HelmValues>` |
| `src/helm/agent_values.rs` | Agent value builder | `build(config, agent_type, infra_env) -> AgentValues` |
| `src/helm/service_values.rs` | Service value builder | `build_service(name, config) -> Result<(ServiceValues, IndexMap<String, String>)>` |
| `src/helm/network_policy.rs` | Network policy builder | `build(config) -> NetworkPolicyValues` |
| `src/helm/secrets.rs` | Secrets builder | `build(config) -> SecretsValues` |
| `src/helm/builtin/_helpers.tpl` | Standard Helm helpers | Defines `chart.name`, `chart.fullname`, `chart.labels`, `chart.selectorLabels` |
| `src/helm/builtin/deployment.yaml` | Stateless service Deployment | Iterates `services` where `stateful: false` |
| `src/helm/builtin/statefulset.yaml` | Stateful service StatefulSet | Iterates `services` where `stateful: true`, includes `volumeClaimTemplates` |
| `src/helm/builtin/service.yaml` | ClusterIP Service | Uses bare `$name` (not release-prefixed) for service discovery |
| `src/helm/builtin/agent-deployment.yaml` | Agent Deployment | Handles both single and "both" agent types |
| `src/helm/builtin/agent-pvc.yaml` | Workspace PVC | ReadWriteOnce PVC with configurable size |
| `src/helm/builtin/secret.yaml` | K8s Secret | Auth keys with REPLACE_ME placeholders + service credentials |
| `src/helm/builtin/configmap.yaml` | ConfigMap | Non-secret agent env vars |
| `src/helm/builtin/networkpolicy.yaml` | NetworkPolicy | Conditional on `.Values.networkPolicy.enabled` |
| `src/helm/builtin/ingress.yaml` | Ingress | Conditional on `.Values.ingress` |
| `src/helm/builtin/mcp-deployment.yaml` | MCP sidecar Deployments | Iterates `.Values.mcp` |
| `src/config/project.rs` | Config types | `HelmConfig` struct |
| `src/error.rs` | Error variant | `Error::HelmGeneration(String)` |
| `src/cli/generate.rs` | CLI integration | `GenerateTarget::Helm`, `generate_helm()` |

## Invariants and Constraints

1. **Helm is opt-in**: The `Helm` target is NOT in the default target list. It must be explicitly requested via `--only helm`.

2. **Service names are unprefixed**: The Service template uses `{{ $name }}` (e.g., "postgres") as the metadata.name, NOT `{{ include "chart.fullname" $ }}-{{ $name }}`. This ensures agent connection URLs (e.g., `postgres://dev:pw@postgres:5432/devdb`) work without modification.

3. **Template syntax is Go-template**: The `.yaml` and `.tpl` files in `builtin/` use Helm's Go template syntax (`{{ .Values.foo }}`), NOT Jinja2. They are static files embedded via `include_str!` and copied verbatim to the chart's `templates/` directory.

4. **All types serialize with camelCase**: All Helm value types use `#[serde(rename_all = "camelCase")]` to match YAML/Helm conventions.

5. **DATABASE_URL aggregation**: The same logic from `compose/generator.rs` is replicated: single DB gets `DATABASE_URL`, multiple DBs get service-specific keys plus `DATABASE_URL` defaulting to the first.

6. **Template files are compiled in**: Changes to any `.yaml`, `.tpl`, or `.rs` file in `src/helm/` require recompilation.

7. **HelmValues must round-trip through serde_yaml**: The `HelmValues` struct must serialize cleanly through `serde_yaml::to_string()` to produce valid YAML.

8. **Stub modules will be replaced**: The `agent_values.rs`, `service_values.rs`, `network_policy.rs`, and `secrets.rs` files contain stub implementations that will be replaced by the `feat/helm-services` branch (WS B). The module interfaces (function signatures) must remain compatible.

9. **Chart output structure**: Generated chart files are written to `<output_dir>/chart/<project_name>/` with subdirectories for `templates/`.

10. **Optional fields use skip_serializing_if**: Fields like `ingress`, `command`, `registry` use `skip_serializing_if` to omit null/None values from the generated YAML.

11. **Volume definitions must match volume mounts**: Every entry in `AgentValues.volume_mounts` must have a corresponding entry in `AgentValues.volumes` with the same `name` field. The `agent_values::build()` function constructs both vectors in parallel to enforce this. Named config volumes map to PVC volume definitions, auth volumes map to Secret volume definitions, and additional mounts map to emptyDir volume definitions.

12. **Auth secret env var names must match actual auth code**: The env var names in `secrets.rs` must match what the auth modules (`auth::claude`, `auth::codex`) actually set. Specifically: Proxy auth uses `ANTHROPIC_AUTH_TOKEN` (not `ANTHROPIC_API_KEY`), BedrockApiKey uses `AWS_BEARER_TOKEN_BEDROCK` (not IAM keys), and Codex Custom defaults to `CUSTOM_API_KEY` (not `OPENAI_API_KEY`).
