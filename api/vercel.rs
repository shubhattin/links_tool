use std::env;
use std::io;
use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

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
    let router = links_tool::app::router(state);

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);
    vercel_runtime::run(app).await
}
