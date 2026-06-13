"""Completed match results scraper.

Fetches finished World Cup 2026 group-stage scores from ESPN's public
soccer scoreboard API and maps each result to a schedule ``matchNumber``
so the web app can pin already-played matches as fixed results.

Only group-stage matches are emitted. Knockout fixtures use positional
placeholders ("2A", "W74", ...) that the web app's fixed-result path does
not yet support, so they are intentionally skipped.
"""

from datetime import date, datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from .base import BaseScraper

# ESPN exposes the men's World Cup under the "fifa.world" soccer league slug.
# The same slug serves past tournaments, which is how the response shape was
# validated against the 2022 edition.
ESPN_SCOREBOARD_URL = (
    "https://site.api.espn.com/apis/site/v2/sports/soccer/{league}/scoreboard"
)

# ESPN display names that differ from our canonical team names. Anything not
# listed here is matched directly (or via the team_mapping aliases / FIFA code).
ESPN_NAME_OVERRIDES = {
    "usa": "United States",
    "united states of america": "United States",
    "korea republic": "South Korea",
    "south korea": "South Korea",
    "ir iran": "Iran",
    "iran": "Iran",
    "türkiye": "Turkey",
    "turkiye": "Turkey",
    "côte d'ivoire": "Ivory Coast",
    "cote d'ivoire": "Ivory Coast",
    "ivory coast": "Ivory Coast",
    "cabo verde": "Cape Verde",
    "czech republic": "Czechia",
    "dr congo": "DR Congo",
    "congo dr": "DR Congo",
    "curacao": "Curacao",
    "curaçao": "Curacao",
    "bosnia & herzegovina": "Bosnia and Herzegovina",
}


class ResultsScraper(BaseScraper):
    """Scraper for completed group-stage match results from ESPN."""

    DEFAULT_LEAGUE = "fifa.world"
    # Group stage window for WC2026 (June 11-27). We pad by a day on each side
    # because ESPN buckets late kickoffs into the following UTC date.
    DEFAULT_START = date(2026, 6, 10)
    DEFAULT_END = date(2026, 6, 28)

    def __init__(
        self,
        output_dir: Path,
        schedule_path: Path,
        groups_path: Path,
        team_mapping_path: Path,
        league: str | None = None,
    ) -> None:
        super().__init__(output_dir)
        self.league = league or self.DEFAULT_LEAGUE
        self.schedule = self._load_json(schedule_path)
        self.groups = self._load_json(groups_path).get("groups", {})
        self.team_mapping = self._load_json(team_mapping_path)
        self._name_to_id = self._build_name_index()
        self._pair_to_match = self._build_pair_index()

    def get_output_filename(self) -> str:
        return "results.json"

    @staticmethod
    def _load_json(path: Path) -> dict:
        import json

        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)

    # ------------------------------------------------------------------
    # Index construction
    # ------------------------------------------------------------------
    def _build_name_index(self) -> dict[str, int]:
        """Map lowercase team identifiers (names, aliases, FIFA codes) to IDs."""
        index: dict[str, int] = {}
        for team in self.team_mapping.get("teams", []):
            team_id = team.get("id")
            if team_id is None:
                continue
            canonical = team.get("canonical_name", "")
            if canonical and "TBD" not in canonical:
                index[canonical.lower()] = team_id
            fifa_code = team.get("fifa_code")
            if fifa_code:
                index[fifa_code.lower()] = team_id
            aliases = team.get("aliases", {})
            for key in ("fifa", "elo", "transfermarkt"):
                alias = aliases.get(key)
                if isinstance(alias, str) and alias and alias != "TBD":
                    index[alias.lower()] = team_id
        return index

    def _resolve_team(self, display_name: str, abbreviation: str | None) -> int | None:
        """Resolve an ESPN team to a canonical team ID."""
        name = (display_name or "").strip().lower()
        override = ESPN_NAME_OVERRIDES.get(name)
        if override:
            return self._name_to_id.get(override.lower())
        if name in self._name_to_id:
            return self._name_to_id[name]
        if abbreviation and abbreviation.lower() in self._name_to_id:
            return self._name_to_id[abbreviation.lower()]
        return None

    @staticmethod
    def _placeholder_position(placeholder: str) -> int | None:
        """Convert a group placeholder ("A1") to a zero-based group index."""
        if not placeholder or len(placeholder) < 2:
            return None
        try:
            n = int(placeholder[1:])
        except ValueError:
            return None
        if 1 <= n <= 4:
            return n - 1
        return None

    def _build_pair_index(self) -> dict[frozenset[int], dict[str, Any]]:
        """Map an unordered pair of team IDs to its scheduled group match.

        Resolves each group-stage fixture's placeholders ("A1", "A2") against
        the group composition to recover the two team IDs, then records the
        canonical home/away orientation so scores can be oriented correctly.
        """
        name_to_id = {
            team.get("canonical_name", "").lower(): team.get("id")
            for team in self.team_mapping.get("teams", [])
            if team.get("id") is not None
        }
        pairs: dict[frozenset[int], dict[str, Any]] = {}
        for match in self.schedule.get("matches", []):
            if match.get("round") != "group_stage":
                continue
            group_id = match.get("groupId")
            roster = self.groups.get(group_id)
            if not roster:
                continue
            home_pos = self._placeholder_position(match.get("homePlaceholder", ""))
            away_pos = self._placeholder_position(match.get("awayPlaceholder", ""))
            if home_pos is None or away_pos is None:
                continue
            home_id = name_to_id.get(roster[home_pos].lower())
            away_id = name_to_id.get(roster[away_pos].lower())
            if home_id is None or away_id is None:
                continue
            pairs[frozenset({home_id, away_id})] = {
                "matchNumber": match.get("matchNumber"),
                "groupId": group_id,
                "homeTeamId": home_id,
                "awayTeamId": away_id,
            }
        return pairs

    # ------------------------------------------------------------------
    # Scraping
    # ------------------------------------------------------------------
    def _date_range(self, start: date, end: date) -> list[date]:
        days = []
        current = start
        while current <= end:
            days.append(current)
            current += timedelta(days=1)
        return days

    def _fetch_day(self, day: date) -> list[dict]:
        url = ESPN_SCOREBOARD_URL.format(league=self.league)
        params = {"dates": day.strftime("%Y%m%d")}
        try:
            response = self.session.get(url, params=params, timeout=30)
            response.raise_for_status()
            return response.json().get("events", [])
        except Exception as e:  # noqa: BLE001 - network/JSON errors are non-fatal per day
            self.logger.warning(f"Failed to fetch {day.isoformat()}: {e}")
            return []

    @staticmethod
    def _event_completed(event: dict) -> bool:
        competitions = event.get("competitions", [])
        if not competitions:
            return False
        status = competitions[0].get("status", {}).get("type", {})
        return bool(status.get("completed"))

    def _parse_event(self, event: dict) -> dict | None:
        """Turn a completed ESPN event into a results entry, or None to skip."""
        competition = event.get("competitions", [{}])[0]
        competitors = competition.get("competitors", [])
        if len(competitors) != 2:
            return None

        sides: dict[str, dict] = {}
        for c in competitors:
            home_away = c.get("homeAway")
            team = c.get("team", {})
            score = c.get("score")
            if home_away not in ("home", "away") or score is None:
                return None
            try:
                goals = int(score)
            except (TypeError, ValueError):
                return None
            team_id = self._resolve_team(
                team.get("displayName", ""), team.get("abbreviation")
            )
            sides[home_away] = {
                "team_id": team_id,
                "name": team.get("displayName", ""),
                "goals": goals,
            }

        home, away = sides.get("home"), sides.get("away")
        if not home or not away:
            return None
        if home["team_id"] is None or away["team_id"] is None:
            unresolved = [s["name"] for s in (home, away) if s["team_id"] is None]
            self.logger.warning(f"Could not resolve team(s): {', '.join(unresolved)}")
            return None

        scheduled = self._pair_to_match.get(
            frozenset({home["team_id"], away["team_id"]})
        )
        if not scheduled:
            # Not a group-stage fixture we track (e.g. knockout, or pairing
            # that does not exist in the group draw).
            return None

        # Orient the scraped score to the schedule's home/away ordering.
        if home["team_id"] == scheduled["homeTeamId"]:
            home_goals, away_goals = home["goals"], away["goals"]
        else:
            home_goals, away_goals = away["goals"], home["goals"]

        match_date = (event.get("date", "") or "")[:10]
        return {
            "matchNumber": scheduled["matchNumber"],
            "groupId": scheduled["groupId"],
            "homeTeamId": scheduled["homeTeamId"],
            "awayTeamId": scheduled["awayTeamId"],
            "homeScore": home_goals,
            "awayScore": away_goals,
            "status": "completed",
            "date": match_date,
        }

    def scrape(
        self, start: date | None = None, end: date | None = None
    ) -> dict[str, Any]:
        start = start or self.DEFAULT_START
        end = end or self.DEFAULT_END
        self.logger.info(
            f"Scraping {self.league} results from {start} to {end}"
        )

        results_by_match: dict[int, dict] = {}
        for day in self._date_range(start, end):
            for event in self._fetch_day(day):
                if not self._event_completed(event):
                    continue
                entry = self._parse_event(event)
                if entry is None:
                    continue
                # Last write wins; identical matches across padded dates collapse.
                results_by_match[entry["matchNumber"]] = entry

        matches = sorted(results_by_match.values(), key=lambda m: m["matchNumber"])
        self.logger.info(f"Resolved {len(matches)} completed group-stage matches")

        return {
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "source": f"espn:{self.league}",
            "matches": matches,
        }
