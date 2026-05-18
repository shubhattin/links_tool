//! Shared stdin confirmation for [`db_export`] / [`db_import`] (`examples/*.rs`).
//!
//! Set `LINKS_TOOL_DB_SKIP_CONFIRM=1` to bypass (same idea as scripted runs).

use std::io::{BufRead, Write};

/// Same answers as legacy TS helpers: `y` / `yes` (trimmed; ASCII lowercased).
pub fn interactive_yes(
    prompt: &str,
    cancelled_message: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("LINKS_TOOL_DB_SKIP_CONFIRM").is_some_and(
        |v| matches!(v.to_str(), Some(s) if s == "1" || s.eq_ignore_ascii_case("true")),
    ) {
        return Ok(());
    }

    crate::cli_log::prompt_line(prompt);
    std::io::stdout().flush()?;

    let mut buf = String::new();
    std::io::stdin().lock().read_line(&mut buf)?;

    let ans = buf.trim().to_ascii_lowercase();
    if matches!(ans.as_str(), "y" | "yes") {
        Ok(())
    } else {
        crate::cli_log::err_line(cancelled_message);
        std::process::exit(1);
    }
}
