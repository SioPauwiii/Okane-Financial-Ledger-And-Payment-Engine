mod app;
mod errors;
mod handlers;
mod models;
mod routes;
mod services;
mod requests;
mod responses;
mod state;

use axum::serve;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
    .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(30)
        .connect(&database_url)
        .await;
    let state = state::AppState { db: db_pool.expect("Failed to connect to the database") };
    let app = app::build_server(state);
    let addr: SocketAddr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into()).parse()?;
    let listener = TcpListener::bind(&addr).await?;

    println!("Server is now listening on http://{addr}");

    serve(listener, app).await?;

    Ok(())
}