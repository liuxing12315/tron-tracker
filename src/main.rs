use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod core;
mod api;
mod web;

use crate::core::config::Config;
use crate::core::database::Database;
use crate::core::scanner::Scanner;
use crate::api::server::ApiServer;
use crate::web::admin::AdminServer;

#[derive(Parser)]
#[command(name = "tron-tracker")]
#[command(about = "Unified TRX/USDT transaction tracking system")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the unified server (default)
    Serve {
        /// API server port
        #[arg(long, default_value = "8080")]
        api_port: u16,
        
        /// Admin interface port
        #[arg(long, default_value = "3000")]
        admin_port: u16,
    },
    
    /// Run database migrations
    Migrate,
    
    /// Start only the blockchain scanner
    Scan,
    
    /// Show system status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("tron_tracker={}", cli.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting TRX/USDT Tracker v3.0.0");

    // Load configuration
    let config = Config::load(&cli.config).await?;
    info!("Configuration loaded from: {}", cli.config);

    // Initialize database
    let database = Database::new(&config.database).await?;
    info!("Database connection established");

    match cli.command.unwrap_or(Commands::Serve { api_port: 8080, admin_port: 3000 }) {
        Commands::Serve { api_port, admin_port } => {
            serve(config, database, api_port, admin_port).await
        }
        Commands::Migrate => {
            migrate(database).await
        }
        Commands::Scan => {
            scan(config, database).await
        }
        Commands::Status => {
            status(config, database).await
        }
    }
}

async fn serve(config: Config, database: Database, api_port: u16, admin_port: u16) -> Result<()> {
    info!("Starting unified server mode");
    
    // Start scanner in background
    let scanner = Scanner::new(config.clone(), database.clone());
    let scanner_handle = tokio::spawn(async move {
        if let Err(e) = scanner.run().await {
            warn!("Scanner error: {}", e);
        }
    });

    // Start API server
    let api_server = ApiServer::new(config.clone(), database.clone());
    let api_handle = tokio::spawn(async move {
        if let Err(e) = api_server.serve(api_port).await {
            warn!("API server error: {}", e);
        }
    });

    // Start admin server
    let admin_server = AdminServer::new(config.clone(), database.clone());
    let admin_handle = tokio::spawn(async move {
        if let Err(e) = admin_server.serve(admin_port).await {
            warn!("Admin server error: {}", e);
        }
    });

    info!("All services started successfully");
    info!("API server: http://0.0.0.0:{}", api_port);
    info!("Admin interface: http://0.0.0.0:{}", admin_port);

    // Wait for all services
    tokio::select! {
        _ = scanner_handle => warn!("Scanner stopped"),
        _ = api_handle => warn!("API server stopped"),
        _ = admin_handle => warn!("Admin server stopped"),
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }

    info!("Shutting down gracefully");
    Ok(())
}

async fn migrate(database: Database) -> Result<()> {
    info!("Running database migrations");
    database.migrate().await?;
    info!("Migrations completed successfully");
    Ok(())
}

async fn scan(config: Config, database: Database) -> Result<()> {
    info!("Starting scanner-only mode");
    let scanner = Scanner::new(config, database);
    scanner.run().await?;
    Ok(())
}

async fn status(config: Config, database: Database) -> Result<()> {
    info!("Checking system status");
    
    // Check database connection
    match database.health_check().await {
        Ok(_) => info!("✓ Database: Connected"),
        Err(e) => warn!("✗ Database: {}", e),
    }
    
    // Check configuration
    info!("✓ Configuration: Loaded");
    info!("  - Database URL: {}", config.database.url);
    info!("  - Redis URL: {}", config.redis.url);
    info!("  - Blockchain nodes: {}", config.blockchain.nodes.len());
    
    Ok(())
}

