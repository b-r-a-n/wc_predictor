#!/usr/bin/env python3
"""Patch fresh ELO ratings into an existing teams.json.

The daily CI refresh only re-scrapes ELO (Transfermarkt and Sofascore block
datacenter IPs, and FIFA's source is a small static top-20). Rather than re-run
the full merge — which would need those blocked sources — we update just the
`elo_rating` field of each team in place, leaving market values, FIFA rankings
and form untouched. Refresh those by running the full scrape + merge locally.

Self-guards against a degraded ELO scrape: requires a minimum number of matched
teams, every new rating in the valid range, and no implausible swing in the
average rating. Exits non-zero (changing nothing) if any guard fails, so a bad
scrape never gets committed.

Usage:
    python -m scrapers.apply_elo \
        --teams data/teams.json \
        --elo scrapers/output/elo_ratings.json \
        --min-teams 45
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ELO_MIN = 1000
ELO_MAX = 2500
ELO_AVG_MAX_DELTA = 200.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--teams", required=True, type=Path, help="teams.json to patch in place")
    parser.add_argument("--elo", required=True, type=Path, help="Scraped elo_ratings.json")
    parser.add_argument("--min-teams", type=int, default=45)
    args = parser.parse_args()

    data = json.loads(args.teams.read_text())
    teams = data.get("teams", [])
    elo = json.loads(args.elo.read_text()).get("teams", {})

    # Validate the incoming ratings before touching anything. Cast to float so
    # the value matches teams.json's existing float formatting (e.g. 2113.0) and
    # unchanged ratings produce no diff.
    matched = {t["name"]: float(elo[t["name"]]) for t in teams if t["name"] in elo}
    if len(matched) < args.min_teams:
        print(
            f"ERROR: ELO scrape matched only {len(matched)}/{len(teams)} teams "
            f"(expected >= {args.min_teams}) — likely degraded. Names unmatched: "
            f"{[t['name'] for t in teams if t['name'] not in elo][:10]}",
            file=sys.stderr,
        )
        return 1

    out_of_range = {n: v for n, v in matched.items() if not (ELO_MIN <= v <= ELO_MAX)}
    if out_of_range:
        print(f"ERROR: ELO ratings out of range ({ELO_MIN}-{ELO_MAX}): {out_of_range}", file=sys.stderr)
        return 1

    old_avg = sum(t["elo_rating"] for t in teams) / len(teams)

    updated = 0
    for t in teams:
        new = matched.get(t["name"])
        if new is not None and new != t["elo_rating"]:
            t["elo_rating"] = new
            updated += 1

    new_avg = sum(t["elo_rating"] for t in teams) / len(teams)
    if abs(new_avg - old_avg) > ELO_AVG_MAX_DELTA:
        print(
            f"ERROR: average ELO swung implausibly: {old_avg:.0f} -> {new_avg:.0f} "
            f"(delta {new_avg - old_avg:+.0f}, allowed +/-{ELO_AVG_MAX_DELTA:.0f}). Not writing.",
            file=sys.stderr,
        )
        return 1

    # Match merge_data's exact serialization (indent=2, no trailing newline) so
    # an unchanged team keeps byte-identical formatting and only real ELO changes
    # show up in the diff.
    args.teams.write_text(json.dumps(data, indent=2))
    unmatched = [t["name"] for t in teams if t["name"] not in elo]
    print(
        f"Applied ELO to {args.teams}: {updated} ratings changed, {len(matched)} matched, "
        f"avg {old_avg:.0f} -> {new_avg:.0f}."
        + (f" Kept existing ELO for unmatched: {unmatched}." if unmatched else "")
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
