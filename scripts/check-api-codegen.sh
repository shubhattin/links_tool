#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

./scripts/sync-api.sh >/dev/null

if ! git diff --quiet -- app/src/lib/api/generated; then
  echo "Generated API client is out of date. Run: bash scripts/sync-api.sh"
  git diff --stat -- app/src/lib/api/generated
  exit 1
fi

echo "Generated API client is up to date."
