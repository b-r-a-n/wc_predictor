#!/usr/bin/env python3
"""Sanity guards for the automated team-data refresh.

`validate` already enforces the hard schema invariants (48 teams, ELO range,
etc.), but several scrapers can *silently degrade* — returning fewer teams or
default values without erroring (see the data-refresh notes: Sofascore 403s,
ELO name drift, ...). This script catches that class of failure so a degraded
scrape never gets auto-committed to the live `teams.json`.

It checks:
  * each raw scraper output still covers enough teams, and
  * the freshly-merged teams.json hasn't regressed versus the committed one
    (no zeroed strategy inputs, no implausible aggregate swings).

Exits 0 if all guards pass, 1 (with an explanation) otherwise.

Usage:
    python -m scrapers.check_data_guards \
        --new data/teams.json \
        --old web/public/data/teams.json \
        --output-dir scrapers/output
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Minimum team counts per raw scraper output. ELO / Transfermarkt / Sofascore
# all cover the full 48-team field; we allow a small slack for occasional
# name-drift misses but fail on real degradation. FIFA is Wikipedia's top-20
# only by design (merge runs with --allow-missing-fifa), so its floor is lower.
RAW_MIN_TEAMS = {
    "elo_ratings.json": 45,
    "transfermarkt_values.json": 45,
    "sofascore_form.json": 45,
    "fifa_rankings.json": 18,
}

# Aggregate-regression bounds versus the previously committed teams.json.
MARKET_VALUE_MIN_RATIO = 0.5
MARKET_VALUE_MAX_RATIO = 2.0
ELO_AVG_MAX_DELTA = 200.0


def _load(path: Path) -> dict:
    with path.open() as fh:
        return json.load(fh)


def check_raw_outputs(output_dir: Path, failures: list[str]) -> None:
    for filename, minimum in RAW_MIN_TEAMS.items():
        path = output_dir / filename
        if not path.exists():
            failures.append(f"{filename}: missing (scraper did not produce output)")
            continue
        count = len(_load(path).get("teams", []))
        if count < minimum:
            failures.append(
                f"{filename}: only {count} teams (expected >= {minimum}) — likely a degraded scrape"
            )


def check_merged(new: dict, old: dict | None, failures: list[str]) -> None:
    teams = new.get("teams", [])
    if len(teams) != 48:
        failures.append(f"teams.json: {len(teams)} teams (expected 48)")
        return

    zero_value = [t["name"] for t in teams if t.get("market_value_millions", 0) <= 0]
    if zero_value:
        failures.append(f"teams with non-positive market value: {zero_value[:5]}")

    missing_form = [t["name"] for t in teams if t.get("sofascore_form") in (None, "")]
    if missing_form:
        failures.append(f"teams missing Sofascore form: {missing_form[:5]}")

    bad_elo = [t["name"] for t in teams if not (1000 <= t.get("elo_rating", 0) <= 2500)]
    if bad_elo:
        failures.append(f"teams with out-of-range ELO: {bad_elo[:5]}")

    if old is None:
        return

    old_teams = old.get("teams", [])
    new_total = sum(t.get("market_value_millions", 0) for t in teams)
    old_total = sum(t.get("market_value_millions", 0) for t in old_teams)
    if old_total > 0:
        ratio = new_total / old_total
        if not (MARKET_VALUE_MIN_RATIO <= ratio <= MARKET_VALUE_MAX_RATIO):
            failures.append(
                f"total market value swung implausibly: {old_total:.0f}M -> {new_total:.0f}M "
                f"({ratio:.2f}x, allowed {MARKET_VALUE_MIN_RATIO}-{MARKET_VALUE_MAX_RATIO}x)"
            )

    if old_teams:
        new_avg = sum(t.get("elo_rating", 0) for t in teams) / len(teams)
        old_avg = sum(t.get("elo_rating", 0) for t in old_teams) / len(old_teams)
        if abs(new_avg - old_avg) > ELO_AVG_MAX_DELTA:
            failures.append(
                f"average ELO swung implausibly: {old_avg:.0f} -> {new_avg:.0f} "
                f"(delta {new_avg - old_avg:+.0f}, allowed +/-{ELO_AVG_MAX_DELTA:.0f})"
            )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--new", required=True, type=Path, help="Freshly merged teams.json")
    parser.add_argument(
        "--old",
        type=Path,
        help="Previously committed teams.json to compare against (optional)",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        type=Path,
        help="Directory holding the raw scraper outputs",
    )
    args = parser.parse_args()

    failures: list[str] = []
    check_raw_outputs(args.output_dir, failures)

    new = _load(args.new)
    old = _load(args.old) if args.old and args.old.exists() else None
    check_merged(new, old, failures)

    if failures:
        print("Data refresh guards FAILED:", file=sys.stderr)
        for f in failures:
            print(f"  - {f}", file=sys.stderr)
        return 1

    print("Data refresh guards passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
