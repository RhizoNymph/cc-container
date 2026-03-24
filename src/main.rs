use cc_container::cli::{Cli, Commands};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    // Set up tracing
    let filter = match cli.global.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let filter = if cli.global.quiet { "error" } else { filter };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .without_time()
        .with_target(false)
        .init();

    let result = match &cli.command {
        Commands::Init(args) => cc_container::cli::init::run(args, &cli.global),
        Commands::Generate(args) => cc_container::cli::generate::run(args, &cli.global),
        Commands::Module(cmd) => cc_container::cli::module::run(cmd, &cli.global),
        Commands::Service(cmd) => cc_container::cli::service::run(cmd, &cli.global),
        Commands::Mcp(cmd) => cc_container::cli::mcp::run(cmd, &cli.global),
        Commands::Config(cmd) => cc_container::cli::config_cmd::run(cmd, &cli.global),
        Commands::Doctor(args) => cc_container::cli::doctor::run(args, &cli.global),
        Commands::Completions(args) => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            clap_complete::generate(args.shell, &mut cmd, "cc-container", &mut std::io::stdout());
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
