//! SeaORM `migrate` binary. Configure the connection with **`PG_DATABASE_URL`**
//! or **`--database-url` / `-u`**.

use clap::Parser;
use sea_orm_cli::MigrateSubcommands;
use sea_orm_migration::cli::run_migrate;
use sea_orm_migration::sea_orm::{ConnectOptions, Database};

fn load_env() {
    let _ = dotenvy::dotenv();
    for path in [".env", ".env.local", "app/.env.local"] {
        let _ = dotenvy::from_filename(path);
    }
}

#[derive(Parser)]
#[command(name = "migrate", version)]
struct MigrateCli {
    #[arg(short = 'v', long, global = true, help = "Show debug messages")]
    verbose: bool,

    #[arg(
        global = true,
        short = 's',
        long,
        env = "DATABASE_SCHEMA",
        long_help = "Database schema\n \
                    - For MySQL and SQLite, this argument is ignored.\n \
                    - For PostgreSQL, this argument is optional with default value 'public'.\n"
    )]
    database_schema: Option<String>,

    #[arg(
        global = true,
        short = 'u',
        long,
        env = "PG_DATABASE_URL",
        help = "PostgreSQL connection URL",
        hide_env_values = true
    )]
    database_url: Option<String>,

    #[command(subcommand)]
    command: Option<MigrateSubcommands>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    load_env();

    let cli = MigrateCli::parse();

    let Some(url) = cli.database_url.clone() else {
        eprintln!("Set PG_DATABASE_URL or pass -u/--database-url");
        std::process::exit(1);
    };

    let schema = cli.database_schema.unwrap_or_else(|| "public".to_owned());

    let connect_options = ConnectOptions::new(url)
        .set_schema_search_path(schema)
        .to_owned();

    let db = match Database::connect(connect_options).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Fail to acquire database connection: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = run_migrate(migration::Migrator, &db, cli.command, cli.verbose).await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
