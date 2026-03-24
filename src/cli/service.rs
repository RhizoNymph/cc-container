use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum ServiceCommand {
    /// List available service templates
    List(ServiceListArgs),
    /// Show service template details
    Info(ServiceInfoArgs),
    /// Add service(s) to project config
    Add(ServiceAddArgs),
    /// Remove service(s) from project config
    Remove(ServiceRemoveArgs),
}

#[derive(Parser)]
pub struct ServiceListArgs {
    /// Filter by category
    #[arg(long)]
    pub category: Option<String>,
}

#[derive(Parser)]
pub struct ServiceInfoArgs {
    /// Service name
    pub name: String,
}

#[derive(Parser)]
pub struct ServiceAddArgs {
    /// Service name(s) to add
    pub names: Vec<String>,

    /// Service parameters (key=value)
    #[arg(long = "with", value_parser = super::parse_key_val)]
    pub params: Vec<(String, String)>,
}

#[derive(Parser)]
pub struct ServiceRemoveArgs {
    /// Service name(s) to remove
    pub names: Vec<String>,
}

pub fn run(cmd: &ServiceCommand, _global: &super::GlobalOpts) -> crate::error::Result<()> {
    match cmd {
        ServiceCommand::List(args) => {
            let templates = crate::compose::service_templates::list_all();
            let filter_cat = args.category.as_deref();

            println!("{:<20} {:<12} {:<8} DESCRIPTION", "NAME", "CATEGORY", "PORT");
            println!("{}", "-".repeat(65));

            for t in &templates {
                let cat = t.category.to_string();
                if let Some(fc) = filter_cat
                    && cat != fc {
                        continue;
                    }
                println!("{:<20} {:<12} {:<8} {}", t.name, cat, t.default_port, t.description);
            }
        }
        ServiceCommand::Info(args) => {
            let templates = crate::compose::service_templates::list_all();
            match templates.iter().find(|t| t.name == args.name) {
                Some(t) => {
                    println!("Name:        {}", t.name);
                    println!("Category:    {}", t.category);
                    println!("Default port: {}", t.default_port);
                    println!("Description: {}", t.description);
                }
                None => {
                    return Err(crate::error::Error::ServiceNotFound(args.name.clone()));
                }
            }
        }
        ServiceCommand::Add(args) => {
            eprintln!("Service add {:?} not yet implemented", args.names);
        }
        ServiceCommand::Remove(args) => {
            eprintln!("Service remove {:?} not yet implemented", args.names);
        }
    }
    Ok(())
}
