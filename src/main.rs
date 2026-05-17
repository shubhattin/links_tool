use std::env;
use tokio::net::TcpListener;

type DynError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename(".env.local");

    let database_url = env::var("PG_DATABASE_URL")
        .map_err(|_| "PG_DATABASE_URL must be set: missing environment variable")?;
    let pool = links_tool::db::build_pool(&database_url)
        .map_err(|e| format!("failed to create database pool: {e}"))?;

    let app = links_tool::app::router(pool);

    let host = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr).await?;
    eprintln!("listening on http://{}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}
