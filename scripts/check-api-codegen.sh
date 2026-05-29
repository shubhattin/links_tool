#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

"$ROOT/scripts/sync-api.sh" >/dev/null

if ! git -C "$ROOT" diff --quiet -- app/src/lib/api/generated; then
  echo "Generated API client is out of date. Run: bash scripts/sync-api.sh"
  git -C "$ROOT" diff --stat -- app/src/lib/api/generated
  exit 1
fi

echo "Generated API client is up to date."
