mod config;
mod database;
mod methods;
mod middleware;

use crate::config::Config;
use crate::database::Database;
use axum::{Extension, Router};
use clap::{ArgAction, Parser};
use tower::ServiceBuilder;
use tracing::Level;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_max_level(args.logging_level())
        .init();

    let database = Database::new(args.config.database).unwrap();

    let middleware_stack = ServiceBuilder::new()
        .layer_fn(middleware::logging::LoggingMiddleware)
        .into_inner();

    let app = Router::new()
        .nest("/", methods::router())
        .layer(Extension(database))
        .layer(middleware_stack);

    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await
        .unwrap();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Logging verbosity
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,
    #[arg(short, long, value_parser = Config::from_toml_path)]
    pub config: Config,
}

impl Args {
    #[must_use]
    pub fn logging_level(&self) -> Level {
        match self.verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        }
    }
}
