//! Dump database to JSON for offline backup / restores.
//!
//! ```bash
//! cargo run -p links_tool --example db_export
//! cargo run -p links_tool --example db_export -- --prod
//! ```
//!

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
use sea_orm::EntityTrait;
use serde::Serialize;

#[derive(Serialize)]
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
            "Are you sure SELECT? {} ",
            cli_log::mode_badge(mode.label())
        ),
        "Aborted export (answered other than yes/y)",
    )?;

    cli_log::step(&format!(
        "Fetching data from database … {} file={}",
        cli_log::mode_badge(mode.label()),
        mode.json_filename()
    ));
    let db = connect(&mode.database_url()?).await?;

    let links_all = links::Entity::find().all(&db).await?;
    let others_all = others::Entity::find().all(&db).await?;

    let out_dir = manifest_dir().join("export_out");
    std::fs::create_dir_all(&out_dir)?;

    let path = out_dir.join(mode.json_filename());
    std::fs::File::create(&path).and_then(|f| {
        serde_json::to_writer_pretty(
            f,
            &DbDump {
                links: links_all,
                others: others_all,
            },
        )
        .map_err(|e| std::io::Error::other(e.to_string()))
    })?;

    cli_log::ok(&format!(
        "Wrote {:?} {}",
        path,
        cli_log::mode_badge(mode.label())
    ));
    Ok(())
}
