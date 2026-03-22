/// Default domain allowlist for Claude Code.
pub fn claude_defaults() -> Vec<&'static str> {
    vec![
        // Anthropic API
        "api.anthropic.com",
        "statsig.anthropic.com",
        "sentry.io",
        // npm registry (for package installs)
        "registry.npmjs.org",
        // GitHub
        "github.com",
        "api.github.com",
        "raw.githubusercontent.com",
        "objects.githubusercontent.com",
        // pip / PyPI
        "pypi.org",
        "files.pythonhosted.org",
        // crates.io
        "crates.io",
        "static.crates.io",
    ]
}

/// Default domain allowlist for OpenAI Codex CLI.
pub fn codex_defaults() -> Vec<&'static str> {
    vec![
        // OpenAI API
        "api.openai.com",
        // npm registry
        "registry.npmjs.org",
        // GitHub
        "github.com",
        "api.github.com",
        "raw.githubusercontent.com",
        "objects.githubusercontent.com",
        // pip / PyPI
        "pypi.org",
        "files.pythonhosted.org",
        // crates.io
        "crates.io",
        "static.crates.io",
    ]
}
