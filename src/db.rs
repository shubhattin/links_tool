use crate::redirect::{self, Link};
use diesel::ConnectionError;
use diesel_async::pooled_connection::deadpool::{BuildError, Pool, PoolError};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use diesel_async::AsyncPgConnection;
use futures_util::FutureExt;

/// Shared PostgreSQL pool (one per serverless instance / process).
pub type DbPool = Pool<AsyncPgConnection>;

#[derive(Debug)]
pub enum LookupError {
    Pool(PoolError),
    Query(diesel::result::Error),
}

impl std::fmt::Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LookupError::Pool(e) => write!(f, "connection pool: {e}"),
            LookupError::Query(e) => write!(f, "query: {e}"),
        }
    }
}

impl std::error::Error for LookupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LookupError::Pool(e) => Some(e),
            LookupError::Query(e) => Some(e),
        }
    }
}

/// Build an async pool with TLS support via `rustls`.
///
/// `AsyncPgConnection::establish` does **not** support TLS, so we provide a
/// custom setup callback that connects through `tokio-postgres` + `rustls`
/// and then wraps the resulting client into an `AsyncPgConnection`.
pub fn build_pool(database_url: &str) -> Result<DbPool, BuildError> {
    let mut manager_config = ManagerConfig::<AsyncPgConnection>::default();
    manager_config.custom_setup = Box::new(|url| establish_tls_connection(url).boxed());

    let config =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(database_url, manager_config);
    Pool::builder(config).build()
}

/// Establish a single TLS-enabled Postgres connection.
///
/// We parse the URL with the `url` crate so that each component (especially
/// the password) is decoded before being passed to `tokio_postgres::Config`.
/// This avoids authentication failures when the password contains special
/// characters that would need percent-encoding in a raw URL string.
async fn establish_tls_connection(database_url: &str) -> Result<AsyncPgConnection, ConnectionError> {
    let url = ::url::Url::parse(database_url)
        .map_err(|e| ConnectionError::BadConnection(format!("invalid DB URL: {e}")))?;

    let mut pg_config = tokio_postgres::Config::new();
    if let Some(host) = url.host_str() {
        pg_config.host(host);
    }
    if let Some(port) = url.port() {
        pg_config.port(port);
    }
    let user = url.username();
    if !user.is_empty() {
        pg_config.user(user);
    }
    if let Some(password) = url.password() {
        // `password()` returns the already-decoded string — no percent-encoding issues.
        pg_config.password(password);
    }
    let db = url.path().trim_start_matches('/');
    if !db.is_empty() {
        pg_config.dbname(db);
    }

    let rustls_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_certs())
        .with_no_client_auth();

    let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
    let (client, conn) = pg_config
        .connect(tls)
        .await
        .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;

    // The tokio-postgres `Connection` object must be polled in the background.
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("postgres connection error: {e}");
        }
    });

    AsyncPgConnection::try_from(client).await
}

/// Mozilla root certificates for TLS verification.
fn root_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    roots
}

/// Load a link by short id using an async pooled connection.
pub async fn lookup_link(pool: &DbPool, id: &str) -> Result<Option<Link>, LookupError> {
    let mut conn = pool.get().await.map_err(LookupError::Pool)?;
    redirect::find_link_by_id(&mut conn, id)
        .await
        .map_err(LookupError::Query)
}
