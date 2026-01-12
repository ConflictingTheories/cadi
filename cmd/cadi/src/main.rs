use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod config;

/// CADI - Content-Addressed Development Interface
/// 
/// A universal build and distribution system for software artifacts,
/// treating all artifacts as content-addressed chunks with multiple
/// interchangeable representations.
#[derive(Parser)]
#[command(name = "cadi")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize CADI configuration
    Init(commands::init::InitArgs),

    /// Import a project and create Source CADI chunks
    Import(commands::import::ImportArgs),

    /// Build artifacts from a manifest
    Build(commands::build::BuildArgs),

    /// Publish chunks to registry
    Publish(commands::publish::PublishArgs),

    /// Fetch chunks from registry
    Fetch(commands::fetch::FetchArgs),

    /// Query registry for chunks
    Query(commands::query::QueryArgs),

    /// Run built artifacts
    Run(commands::run::RunArgs),

    /// Show build plan without executing
    Plan(commands::plan::PlanArgs),

    /// Verify signatures and provenance
    Verify(commands::verify::VerifyArgs),

    /// Manage trusted signers
    Trust(commands::trust::TrustArgs),

    /// Garbage collect local cache
    Gc(commands::gc::GcArgs),

    /// Show efficiency metrics and statistics
    Stats(commands::stats::StatsArgs),

    /// Run demo projects
    Demo(commands::demo::DemoArgs),

    /// Scrape and chunk repositories or files
    Scrape(commands::scrape::ScrapeArgs),

    /// Validate a CADL file against the specification
    Validate(commands::validate::ValidateArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("CADI_LOG").unwrap_or_else(|_| "cadi=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    // Load configuration
    let config = config::load_config(cli.config.as_deref())?;

    match cli.command {
        Commands::Init(args) => commands::init::execute(args, &config).await,
        Commands::Import(args) => commands::import::execute(args, &config).await,
        Commands::Build(args) => commands::build::execute(args, &config).await,
        Commands::Publish(args) => commands::publish::execute(args, &config).await,
        Commands::Fetch(args) => commands::fetch::execute(args, &config).await,
        Commands::Query(args) => commands::query::execute(args, &config).await,
        Commands::Run(args) => commands::run::execute(args, &config).await,
        Commands::Plan(args) => commands::plan::execute(args, &config).await,
        Commands::Verify(args) => commands::verify::execute(args, &config).await,
        Commands::Trust(args) => commands::trust::execute(args, &config).await,
        Commands::Gc(args) => commands::gc::execute(args, &config).await,
        Commands::Stats(args) => commands::stats::execute(args, &config).await,
        Commands::Demo(args) => commands::demo::execute(args, &config).await,
        Commands::Scrape(args) => commands::scrape::execute(args, &config).await,
        Commands::Validate(args) => commands::validate::execute(args, &config).await,
    }
}
