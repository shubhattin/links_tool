## Watch Rust while you edit (`cargo-watch`)

Install using `cargo install cargo-watch` or other binary distribution source (eg :- brew, pacman, dnf, etc)

```bash
cargo watch -w src -x 'run --bin main'
```

---

## Environment

| Variable | Used by |
|----------|---------|
| `PG_DATABASE_URL` | **`links_tool`** binaries (`main`, `api`), **`db_*` examples** (default **dev** mode), and **`migrate`** (or pass **`-u`**) |
| `PG_DATABASE_URL1` | **`db_*` examples** with **`--prod`** |
| `DATABASE_SCHEMA` | Optional Postgres search path for **`cargo run -p migration`** (default `public`) |
| `LISTEN_ADDR` / `PORT` | Local `main` server |

## Build & run (Rust)

```bash
cargo check --workspace --all-targets
cargo build -p links_tool --release    # prod-like only when needed
cargo run -p links_tool --bin main
cargo run -p links_tool --bin api   # Vercel-style entry
```

## SeaORM migrations

The **`migration`** crate provides the **`migrate`** binary. Set **`PG_DATABASE_URL`**, or pass **`-u`** / **`--database-url`**.

```bash
# Apply pending migrations
cargo run -p migration -- up

# Show status
cargo run -p migration -- status

# Roll back last migration
cargo run -p migration -- down -n 1

# Fresh database (destructive)
cargo run -p migration -- fresh
```

### `sea-orm-cli` scaffolding (optional)

Install once, then scaffold new migrations from the **`migration`** crate directory:

```bash
cargo install sea-orm-cli
cd migration
sea-orm-cli migrate generate descriptive_name_here
```

Edit the generated `m*_*.rs` and register it inside `migration/src/lib.rs`.

## JSON export / import (dev tooling)

Outputs land under **`./export_out/`** at the Rust project root (**gitignored**; **`db_import`** creates the directory if absent).

| Mode | Flag (default **dev**) | JSON file | DB URL var |
|------|------------------------|-----------|------------|
| **dev** | `--dev` or omit | **`db_data.json`** | **`PG_DATABASE_URL`** |
| **prod** | **`--prod`** | **`db_data_prod.json`** | **`PG_DATABASE_URL1`** |

Pass flags **after** `--` with `cargo run`:

```bash
cargo run -p links_tool --example db_export
cargo run -p links_tool --example db_export -- --prod

cargo run -p links_tool --example db_import
cargo run -p links_tool --example db_import -- --prod
```