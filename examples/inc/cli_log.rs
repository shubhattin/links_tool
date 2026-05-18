#![allow(dead_code)]
//! Chalk-style terminal output for `db_export` / `db_import` (examples only).

use colored::Colorize;

/// Mode badge like `[dev]` / `[prod]`.
pub fn mode_badge(label: &str) -> String {
    let inner = match label {
        "prod" => label.bright_magenta().bold().to_string(),
        _ => label.bright_yellow().bold().to_string(),
    };
    format!("{}{}{}", "[".dimmed(), inner, "]".dimmed())
}

pub fn prompt_line(text: &str) {
    print!("{}", text.bright_cyan().bold());
}

pub fn step(msg: &str) {
    eprintln!("{}", msg.bright_blue());
}

pub fn ok(msg: &str) {
    eprintln!("{}", msg.green());
}

pub fn warn(msg: &str) {
    eprintln!("{}", msg.bright_yellow());
}

pub fn err_line(msg: &str) {
    eprintln!("{}", msg.red().bold());
}
