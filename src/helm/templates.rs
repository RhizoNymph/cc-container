/// Returns all embedded Helm template files as (filename, content) pairs.
///
/// These are static Go-template YAML files that get copied verbatim into
/// the generated chart's `templates/` directory. Helm's template engine
/// processes them at install time.
pub fn all_templates() -> Vec<(&'static str, &'static str)> {
    vec![
        ("_helpers.tpl", include_str!("builtin/_helpers.tpl")),
        ("deployment.yaml", include_str!("builtin/deployment.yaml")),
        ("statefulset.yaml", include_str!("builtin/statefulset.yaml")),
        ("service.yaml", include_str!("builtin/service.yaml")),
        (
            "agent-deployment.yaml",
            include_str!("builtin/agent-deployment.yaml"),
        ),
        ("agent-pvc.yaml", include_str!("builtin/agent-pvc.yaml")),
        ("secret.yaml", include_str!("builtin/secret.yaml")),
        ("configmap.yaml", include_str!("builtin/configmap.yaml")),
        (
            "networkpolicy.yaml",
            include_str!("builtin/networkpolicy.yaml"),
        ),
        ("ingress.yaml", include_str!("builtin/ingress.yaml")),
        (
            "agent-service.yaml",
            include_str!("builtin/agent-service.yaml"),
        ),
        (
            "deployment-pvc.yaml",
            include_str!("builtin/deployment-pvc.yaml"),
        ),
        (
            "mcp-deployment.yaml",
            include_str!("builtin/mcp-deployment.yaml"),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_templates_returns_expected_count() {
        let templates = all_templates();
        assert_eq!(templates.len(), 13);
    }

    #[test]
    fn all_template_filenames_are_unique() {
        let templates = all_templates();
        let mut names: Vec<&str> = templates.iter().map(|(name, _)| *name).collect();
        let original_len = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), original_len, "duplicate template filenames");
    }

    #[test]
    fn all_templates_have_non_empty_content() {
        for (name, content) in all_templates() {
            assert!(
                !content.is_empty(),
                "template {name} should have non-empty content"
            );
        }
    }

    #[test]
    fn helpers_template_defines_chart_name() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "_helpers.tpl")
            .expect("_helpers.tpl should exist");
        assert!(content.contains("define \"chart.name\""));
        assert!(content.contains("define \"chart.fullname\""));
        assert!(content.contains("define \"chart.labels\""));
        assert!(content.contains("define \"chart.selectorLabels\""));
    }

    #[test]
    fn deployment_template_uses_go_template_syntax() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "deployment.yaml")
            .expect("deployment.yaml should exist");
        assert!(content.contains("{{- range $name, $svc := .Values.services }}"));
        assert!(content.contains("kind: Deployment"));
    }

    #[test]
    fn statefulset_template_has_volume_claim_templates() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "statefulset.yaml")
            .expect("statefulset.yaml should exist");
        assert!(content.contains("volumeClaimTemplates"));
        assert!(content.contains("kind: StatefulSet"));
    }

    #[test]
    fn service_template_uses_bare_name() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "service.yaml")
            .expect("service.yaml should exist");
        // Service name should be just $name, not prefixed with release name
        assert!(content.contains("name: {{ $name }}"));
    }

    #[test]
    fn agent_deployment_handles_both_type() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "agent-deployment.yaml")
            .expect("agent-deployment.yaml should exist");
        assert!(content.contains("eq .Values.agent.agentType \"both\""));
    }

    #[test]
    fn networkpolicy_is_conditional() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "networkpolicy.yaml")
            .expect("networkpolicy.yaml should exist");
        assert!(content.contains(".Values.networkPolicy.enabled"));
        assert!(content.contains("kind: NetworkPolicy"));
    }

    #[test]
    fn ingress_is_conditional() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "ingress.yaml")
            .expect("ingress.yaml should exist");
        assert!(content.contains("{{- if .Values.ingress }}"));
        assert!(content.contains("kind: Ingress"));
    }

    #[test]
    fn secret_template_uses_replace_me_placeholder() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "secret.yaml")
            .expect("secret.yaml should exist");
        assert!(content.contains("REPLACE_ME"));
        assert!(content.contains("kind: Secret"));
    }

    #[test]
    fn mcp_deployment_iterates_mcp_values() {
        let templates = all_templates();
        let (_, content) = templates
            .iter()
            .find(|(name, _)| *name == "mcp-deployment.yaml")
            .expect("mcp-deployment.yaml should exist");
        assert!(content.contains("range $name, $mcp := .Values.mcp"));
    }
}
