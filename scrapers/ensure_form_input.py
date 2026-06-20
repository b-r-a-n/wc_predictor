#!/usr/bin/env python3
"""Guarantee a usable Sofascore form input for the merge step.

Sofascore's API 403s requests from datacenter IPs (e.g. GitHub Actions
runners), so the daily refresh can't reliably re-scrape recent form. Rather
than block the whole refresh — or let merge fall back to a flat 1.5 form for
every team, which silently neutralizes the Form strategy — we preserve the
last-known-good form by reconstructing the merge's expected form file from the
currently committed teams.json.

If the freshly scraped form file exists and covers enough teams, it's left
untouched. Otherwise it's rebuilt from teams.json as
`{"teams": {<canonical_name>: <form>}}`, which is exactly the shape
merge_data.get_sofascore_form reads.

Usage:
    python -m scrapers.ensure_form_input \
        --form scrapers/output/sofascore_form.json \
        --fallback web/public/data/teams.json \
        --min-teams 45
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--form", required=True, type=Path, help="Scraped sofascore_form.json")
    parser.add_argument(
        "--fallback",
        required=True,
        type=Path,
        help="Committed teams.json to rebuild form from when the scrape is unusable",
    )
    parser.add_argument("--min-teams", type=int, default=45)
    args = parser.parse_args()

    if args.form.exists():
        try:
            count = len(json.loads(args.form.read_text()).get("teams", {}))
        except (json.JSONDecodeError, OSError):
            count = 0
        if count >= args.min_teams:
            print(f"Using freshly scraped form ({count} teams).")
            return 0
        print(f"Scraped form covers only {count} teams (< {args.min_teams}); falling back.")
    else:
        print("No scraped form file found; falling back.")

    teams = json.loads(args.fallback.read_text()).get("teams", [])
    form = {t["name"]: t["sofascore_form"] for t in teams if t.get("sofascore_form") is not None}
    if len(form) < args.min_teams:
        print(
            f"ERROR: committed teams.json only yields {len(form)} form entries; cannot fall back.",
            file=sys.stderr,
        )
        return 1

    args.form.parent.mkdir(parents=True, exist_ok=True)
    args.form.write_text(
        json.dumps(
            {"teams": form, "source": "preserved-from-committed-teams.json"},
            indent=2,
        )
    )
    print(f"Preserved last-known-good form for {len(form)} teams from {args.fallback}.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
