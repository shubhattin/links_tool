//! CLI `--dev` / `--prod` mode for [`db_export`] / [`db_import`].

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpMode {
    /// `PG_DATABASE_URL`, file `db_data.json`.
    Dev,
    /// `PG_DATABASE_URL1`, file `db_data_prod.json`.
    Prod,
}

impl DumpMode {
    /// Parses `std::env::args` after the binary (`cargo run … -- `--prod`).
    pub fn parse_from_args() -> Result<Self, String> {
        let mut mode = Self::Dev;
        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--dev" => mode = Self::Dev,
                "--prod" => mode = Self::Prod,
                other => {
                    return Err(format!(
                        "unknown argument `{other}` (expected `--dev` or `--prod`)"
                    ));
                }
            }
        }
        Ok(mode)
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Prod => "prod",
        }
    }

    pub const fn json_filename(self) -> &'static str {
        match self {
            Self::Dev => "db_data.json",
            Self::Prod => "db_data_prod.json",
        }
    }

    pub fn database_url(self) -> Result<String, String> {
        match self {
            Self::Dev => std::env::var("PG_DATABASE_URL").map_err(|_| {
                "`PG_DATABASE_URL` required for dev mode (`--dev` default)".to_string()
            }),
            Self::Prod => std::env::var("PG_DATABASE_URL1")
                .map_err(|_| "`PG_DATABASE_URL1` required for prod mode (`--prod`)".to_string()),
        }
    }
}
