use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum ModuleCommand {
    /// List available modules
    List(ModuleListArgs),
    /// Show detailed module info
    Info(ModuleInfoArgs),
    /// Add module(s) to project config
    Add(ModuleAddArgs),
    /// Remove module(s) from project config
    Remove(ModuleRemoveArgs),
    /// Scaffold a custom module
    Create(ModuleCreateArgs),
}

#[derive(Parser)]
pub struct ModuleListArgs {
    /// Filter by category
    #[arg(long)]
    pub category: Option<String>,
}

#[derive(Parser)]
pub struct ModuleInfoArgs {
    /// Module name
    pub name: String,
}

#[derive(Parser)]
pub struct ModuleAddArgs {
    /// Module name(s) to add
    pub names: Vec<String>,

    /// Module parameters (key=value)
    #[arg(long = "with", value_parser = super::parse_key_val)]
    pub params: Vec<(String, String)>,
}

#[derive(Parser)]
pub struct ModuleRemoveArgs {
    /// Module name(s) to remove
    pub names: Vec<String>,
}

#[derive(Parser)]
pub struct ModuleCreateArgs {
    /// Module name
    #[arg(long)]
    pub name: String,

    /// Output directory
    #[arg(long)]
    pub dir: Option<std::path::PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_global() -> super::super::GlobalOpts {
        super::super::GlobalOpts {
            target_dir: None,
            config: None,
            verbose: 0,
            quiet: true,
            color: super::super::ColorMode::Never,
        }
    }

    #[test]
    fn module_list_runs_without_error() {
        let cmd = ModuleCommand::List(ModuleListArgs { category: None });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn module_list_with_category_runs_without_error() {
        let cmd = ModuleCommand::List(ModuleListArgs {
            category: Some("lang".to_string()),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn module_list_with_nonexistent_category_runs_without_error() {
        let cmd = ModuleCommand::List(ModuleListArgs {
            category: Some("nonexistent".to_string()),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn module_info_known_module() {
        let cmd = ModuleCommand::Info(ModuleInfoArgs {
            name: "node".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn module_info_unknown_module_errors() {
        let cmd = ModuleCommand::Info(ModuleInfoArgs {
            name: "nonexistent-module".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("module not found"));
    }

    #[test]
    fn module_add_returns_not_implemented() {
        let cmd = ModuleCommand::Add(ModuleAddArgs {
            names: vec!["node".to_string()],
            params: vec![],
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "Expected 'not yet implemented' error, got: {err_msg}"
        );
    }

    #[test]
    fn module_remove_returns_not_implemented() {
        let cmd = ModuleCommand::Remove(ModuleRemoveArgs {
            names: vec!["git".to_string()],
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "Expected 'not yet implemented' error, got: {err_msg}"
        );
    }

    #[test]
    fn module_create_returns_not_implemented() {
        let cmd = ModuleCommand::Create(ModuleCreateArgs {
            name: "mymod".to_string(),
            dir: None,
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "Expected 'not yet implemented' error, got: {err_msg}"
        );
    }

    #[test]
    fn module_info_git_module() {
        let cmd = ModuleCommand::Info(ModuleInfoArgs {
            name: "git".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }
}

pub fn run(cmd: &ModuleCommand, _global: &super::GlobalOpts) -> crate::error::Result<()> {
    match cmd {
        ModuleCommand::List(args) => {
            let registry = crate::module::ModuleRegistry::new();
            let filter_cat = args.category.as_deref();

            println!("{:<20} {:<10} DESCRIPTION", "NAME", "CATEGORY");
            println!("{}", "-".repeat(60));

            for (_name, entry) in registry.all() {
                let meta = &entry.definition.module;
                let cat = meta.category.to_string();
                if let Some(fc) = filter_cat
                    && cat != fc {
                        continue;
                    }
                println!("{:<20} {:<10} {}", meta.name, cat, meta.description);
            }
        }
        ModuleCommand::Info(args) => {
            let registry = crate::module::ModuleRegistry::new();
            let entry = registry
                .get(&args.name)
                .ok_or_else(|| crate::error::Error::ModuleNotFound(args.name.clone()))?;

            let meta = &entry.definition.module;
            println!("Name:        {}", meta.name);
            println!("Category:    {}", meta.category);
            println!("Description: {}", meta.description);

            if !meta.parameters.is_empty() {
                println!("\nParameters:");
                for (name, param) in &meta.parameters {
                    let default = param
                        .default
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "(none)".to_string());
                    println!("  {:<20} {:?}, default={}, {}", name, param.param_type, default, param.description);
                }
            }

            if !meta.dependencies.requires.is_empty() {
                println!("\nRequires: {}", meta.dependencies.requires.join(", "));
            }
            if !meta.dependencies.conflicts.is_empty() {
                println!("Conflicts: {}", meta.dependencies.conflicts.join(", "));
            }
        }
        ModuleCommand::Add(_args) => {
            return Err(crate::error::Error::Other(
                "module add is not yet implemented".to_string(),
            ));
        }
        ModuleCommand::Remove(_args) => {
            return Err(crate::error::Error::Other(
                "module remove is not yet implemented".to_string(),
            ));
        }
        ModuleCommand::Create(_args) => {
            return Err(crate::error::Error::Other(
                "module create is not yet implemented".to_string(),
            ));
        }
    }
    Ok(())
}
