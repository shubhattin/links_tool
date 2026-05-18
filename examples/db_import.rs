//! Restore database from JSON produced by [`db_export`].
//!
//! ```bash
//! cargo run -p links_tool --example db_import
//! cargo run -p links_tool --example db_import -- --prod
//! ```

#[path = "inc/cli_log.rs"]
mod cli_log;
#[path = "inc/db_dump_mode.rs"]
mod db_dump_mode;
#[path = "inc/prompt.rs"]
mod prompt;

use db_dump_mode::DumpMode;
use links_tool::{
    db::connect,
    entities::{links, others},
};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, TransactionTrait};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DbDump {
    links: Vec<links::Model>,
    others: Vec<others::Model>,
}

fn manifest_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn dotenv_hints() {
    let _ = dotenvy::dotenv();
    let root = manifest_dir();
    let _ = dotenvy::from_filename(root.join(".env.local"));
    let _ = dotenvy::from_filename(root.join("app/.env.local"));
    if let Some(parent) = root.parent() {
        let _ = dotenvy::from_filename(parent.join(".env.local"));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv_hints();

    let mode = DumpMode::parse_from_args()?;

    prompt::interactive_yes(
        &format!(
            "Are you sure INSERT? {} ",
            cli_log::mode_badge(mode.label())
        ),
        "Aborted import (answered other than yes/y)",
    )?;

    cli_log::step(&format!(
        "Restoring database … {} file={}",
        cli_log::mode_badge(mode.label()),
        mode.json_filename()
    ));

    let export_dir = manifest_dir().join("export_out");
    std::fs::create_dir_all(&export_dir)?;

    let path = export_dir.join(mode.json_filename());
    if !path.is_file() {
        cli_log::err_line(&format!("Missing backup JSON ({})", path.display()));
        let hint = match mode {
            DumpMode::Dev => "Run `cargo run -p links_tool --example db_export` first.",
            DumpMode::Prod => "Run `cargo run -p links_tool --example db_export -- --prod` first.",
        };
        let msg = format!(
            "missing backup JSON at {}\n{hint}\n\
             (dev needs `PG_DATABASE_URL`; prod needs `PG_DATABASE_URL1`)",
            path.display(),
        );
        return Err(msg.into());
    }

    let bytes = std::fs::read(&path)?;
    let dump: DbDump = serde_json::from_slice(&bytes)?;
    let n_links = dump.links.len();
    let n_others = dump.others.len();

    let db = connect(&mode.database_url()?).await?;

    let txn = db.begin().await?;

    links::Entity::delete_many().exec(&txn).await?;
    others::Entity::delete_many().exec(&txn).await?;
    cli_log::warn(&format!(
        "Cleared tables `links`, `others` {}",
        cli_log::mode_badge(mode.label())
    ));

    for row in dump.links {
        row.into_active_model().insert(&txn).await?;
    }
    cli_log::step(&format!("Inserted {n_links} link row(s)"));

    for row in dump.others {
        row.into_active_model().insert(&txn).await?;
    }
    cli_log::step(&format!("Inserted {n_others} other row(s)"));

    txn.commit().await?;

    cli_log::ok(&format!(
        "Import committed successfully {}",
        cli_log::mode_badge(mode.label())
    ));
    Ok(())
}
