use super::definition::ModuleDefinition;

/// A built-in module: its TOML definition + Dockerfile template.
pub struct BuiltinModule {
    pub definition: ModuleDefinition,
    pub template: &'static str,
}

macro_rules! builtin {
    ($toml:expr, $template:expr) => {{
        let def_str = include_str!($toml);
        let definition: ModuleDefinition =
            toml::from_str(def_str).expect(concat!("invalid built-in module TOML: ", $toml));
        BuiltinModule {
            definition,
            template: include_str!($template),
        }
    }};
}

/// Load all built-in modules.
pub fn load_all() -> Vec<BuiltinModule> {
    vec![
        // Base
        builtin!("base/ubuntu.toml", "base/ubuntu.dockerfile.j2"),
        builtin!("base/debian.toml", "base/debian.dockerfile.j2"),
        builtin!("base/alpine.toml", "base/alpine.dockerfile.j2"),
        // Lang
        builtin!("lang/node.toml", "lang/node.dockerfile.j2"),
        builtin!("lang/python.toml", "lang/python.dockerfile.j2"),
        builtin!("lang/rust.toml", "lang/rust.dockerfile.j2"),
        builtin!("lang/go.toml", "lang/go.dockerfile.j2"),
        builtin!("lang/java.toml", "lang/java.dockerfile.j2"),
        builtin!("lang/ruby.toml", "lang/ruby.dockerfile.j2"),
        builtin!("lang/dotnet.toml", "lang/dotnet.dockerfile.j2"),
        builtin!("lang/zig.toml", "lang/zig.dockerfile.j2"),
        builtin!("lang/cpp.toml", "lang/cpp.dockerfile.j2"),
        // Tool
        builtin!("tool/git.toml", "tool/git.dockerfile.j2"),
        builtin!(
            "tool/build_essential.toml",
            "tool/build_essential.dockerfile.j2"
        ),
        builtin!("tool/docker_cli.toml", "tool/docker_cli.dockerfile.j2"),
        // Agent
        builtin!("agent/claude_code.toml", "agent/claude_code.dockerfile.j2"),
        builtin!("agent/codex_cli.toml", "agent/codex_cli.dockerfile.j2"),
        // Security
        builtin!(
            "security/user_setup.toml",
            "security/user_setup.dockerfile.j2"
        ),
        builtin!(
            "security/firewall.toml",
            "security/firewall.dockerfile.j2"
        ),
    ]
}
