#!/usr/bin/env bash
# Regenerate web/public/data/baseline_results.json from a full composite
# simulation. Intended to be run locally or in CI whenever teams.json or
# the simulator changes.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ITERATIONS="${ITERATIONS:-500000}"
SEED="${SEED:-42}"
ELO_W="${ELO_W:-0.4}"
MARKET_W="${MARKET_W:-0.4}"
FIFA_W="${FIFA_W:-0.1}"
FORM_W="${FORM_W:-0.1}"
STRATEGY="${STRATEGY:-composite}"
OUTPUT="${OUTPUT:-$REPO_ROOT/web/public/data/baseline_results.json}"

echo "Building wc-cli (release)..."
cargo build --release -p wc-cli --manifest-path "$REPO_ROOT/Cargo.toml"

TMP_RESULTS="$(mktemp)"
trap 'rm -f "$TMP_RESULTS"' EXIT

echo "Running $ITERATIONS-iteration $STRATEGY simulation (seed=$SEED)..."
"$REPO_ROOT/target/release/wc" \
  --data "$REPO_ROOT/data/teams.json" \
  --format json \
  simulate \
  -n "$ITERATIONS" \
  -s "$STRATEGY" \
  --elo-weight "$ELO_W" \
  --market-weight "$MARKET_W" \
  --fifa-weight "$FIFA_W" \
  --form-weight "$FORM_W" \
  --seed "$SEED" \
  --raw > "$TMP_RESULTS"

GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
GIT_SHA="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")"

mkdir -p "$(dirname "$OUTPUT")"

# Wrap raw AggregatedResults with metadata the UI can display.
python3 - "$TMP_RESULTS" "$OUTPUT" "$GENERATED_AT" "$GIT_SHA" \
  "$STRATEGY" "$ITERATIONS" "$SEED" \
  "$ELO_W" "$MARKET_W" "$FIFA_W" "$FORM_W" <<'PY'
import json, sys
results_path, out_path, generated_at, git_sha, strategy, iters, seed, elo, mkt, fifa, form = sys.argv[1:]
with open(results_path) as f:
    results = json.load(f)
wrapped = {
    "generated_at": generated_at,
    "git_sha": git_sha,
    "strategy": strategy,
    "iterations": int(iters),
    "seed": int(seed),
    "composite_weights": {
        "elo": float(elo),
        "market": float(mkt),
        "fifa": float(fifa),
        "form": float(form),
    },
    "results": results,
}
with open(out_path, "w") as f:
    json.dump(wrapped, f)
PY

echo "Baseline written to $OUTPUT"
