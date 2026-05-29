#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Generating OpenAPI schema from Rust..."
(cd "$ROOT" && cargo run --bin gen-openapi --quiet)

echo "Generating TypeScript client..."
(cd "$ROOT/app" && bun run openapi-ts)

echo "Done. Generated client: app/src/lib/api/generated/"
