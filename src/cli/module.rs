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
    #[arg(long = "with", value_parser = parse_key_val)]
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

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let (key, val) = s
        .split_once('=')
        .ok_or_else(|| format!("invalid key=value pair: {s}"))?;
    Ok((key.to_string(), val.to_string()))
}

pub fn run(cmd: &ModuleCommand, _global: &super::GlobalOpts) -> crate::error::Result<()> {
    match cmd {
        ModuleCommand::List(args) => {
            let registry = crate::module::ModuleRegistry::new();
            let filter_cat = args.category.as_deref();

            println!("{:<20} {:<10} {}", "NAME", "CATEGORY", "DESCRIPTION");
            println!("{}", "-".repeat(60));

            for (_name, entry) in registry.all() {
                let meta = &entry.definition.module;
                let cat = meta.category.to_string();
                if let Some(fc) = filter_cat {
                    if cat != fc {
                        continue;
                    }
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
        ModuleCommand::Add(args) => {
            eprintln!("Module add {:?} not yet implemented", args.names);
        }
        ModuleCommand::Remove(args) => {
            eprintln!("Module remove {:?} not yet implemented", args.names);
        }
        ModuleCommand::Create(args) => {
            eprintln!("Module create '{}' not yet implemented", args.name);
        }
    }
    Ok(())
}
