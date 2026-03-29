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
    fn service_list_runs_without_error() {
        let cmd = ServiceCommand::List(ServiceListArgs { category: None });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn service_list_with_database_category() {
        let cmd = ServiceCommand::List(ServiceListArgs {
            category: Some("database".to_string()),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn service_list_with_cache_category() {
        let cmd = ServiceCommand::List(ServiceListArgs {
            category: Some("cache".to_string()),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn service_list_with_nonexistent_category() {
        let cmd = ServiceCommand::List(ServiceListArgs {
            category: Some("nonexistent".to_string()),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn service_info_postgres() {
        let cmd = ServiceCommand::Info(ServiceInfoArgs {
            name: "postgres".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn service_info_redis() {
        let cmd = ServiceCommand::Info(ServiceInfoArgs {
            name: "redis".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn service_info_kafka() {
        let cmd = ServiceCommand::Info(ServiceInfoArgs {
            name: "kafka".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_ok());
    }

    #[test]
    fn service_info_unknown_errors() {
        let cmd = ServiceCommand::Info(ServiceInfoArgs {
            name: "nonexistent-service".to_string(),
        });
        let result = run(&cmd, &default_global());
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("service template not found"));
    }

    #[test]
    fn service_add_returns_not_implemented() {
        let cmd = ServiceCommand::Add(ServiceAddArgs {
            names: vec!["postgres".to_string()],
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
    fn service_remove_returns_not_implemented() {
        let cmd = ServiceCommand::Remove(ServiceRemoveArgs {
            names: vec!["redis".to_string()],
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
    fn service_info_all_known_services() {
        let known = [
            "postgres", "mysql", "mariadb", "mongodb", "cockroachdb",
            "redis", "memcached", "rabbitmq", "kafka", "nats",
            "elasticsearch", "meilisearch", "typesense", "minio",
            "prometheus", "grafana", "traefik", "nginx",
        ];
        for name in &known {
            let cmd = ServiceCommand::Info(ServiceInfoArgs {
                name: name.to_string(),
            });
            let result = run(&cmd, &default_global());
            assert!(result.is_ok(), "service info should succeed for '{}'", name);
        }
    }
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
        ServiceCommand::Add(_args) => {
            return Err(crate::error::Error::Other(
                "service add is not yet implemented".to_string(),
            ));
        }
        ServiceCommand::Remove(_args) => {
            return Err(crate::error::Error::Other(
                "service remove is not yet implemented".to_string(),
            ));
        }
    }
    Ok(())
}
