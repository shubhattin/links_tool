use std::env;
use std::io;
use tokio::net::TcpListener;
use vercel_runtime::Error;

const PORT: u16 = 5778;

fn io_error(msg: impl Into<String>) -> Error {
    Box::new(io::Error::other(msg.into()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename(".env.local");

    let database_url =
        env::var("PG_DATABASE_URL").map_err(|_| io_error("PG_DATABASE_URL must be set"))?;
    let db = links_tool::db::connect(&database_url)
        .await
        .map_err(|e| io_error(format!("failed to connect to database: {e}")))?;

    let state = links_tool::app::build_state(db).map_err(io_error)?;
    let app = links_tool::app::router(state);

    let host = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("PORT").unwrap_or_else(|_| PORT.to_string());
    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| io_error(format!("failed to bind {addr}: {e}")))?;
    eprintln!(
        "listening on http://{}",
        listener
            .local_addr()
            .map_err(|e| io_error(format!("failed to read local addr: {e}")))?
    );

    axum::serve(listener, app)
        .await
        .map_err(|e| io_error(format!("server error: {e}")))
}
