"""Completed match results scraper.

Fetches finished World Cup 2026 scores from ESPN's public soccer scoreboard
API and maps each result to a schedule ``matchNumber`` so the web app can pin
already-played matches as fixed results.

Group-stage matches are matched by the unordered pair of teams (the draw fixes
who plays whom). Knockout matches can't be matched that way — the teams depend
on earlier results — so each completed knockout event is matched to its
scheduled ``matchNumber`` by (date, venue), which is unique across the bracket.
Knockout entries record the winning team id; the web app pins them as
winner-only fixed results (the exact score isn't needed downstream).
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
    "bosnia-herzegovina": "Bosnia and Herzegovina",
    "bosnia herzegovina": "Bosnia and Herzegovina",
}

# ESPN venue names that differ from our venues.json names. Keyed by the
# normalized (lowercase, alphanumeric-only) ESPN venue name. City-based
# matching handles most cases; this covers renamed/sponsor-named stadiums.
VENUE_NAME_OVERRIDES = {
    "estadiobanorte": "azteca",  # Estadio Azteca was renamed for 2026
}


class ResultsScraper(BaseScraper):
    """Scraper for completed group-stage match results from ESPN."""

    DEFAULT_LEAGUE = "fifa.world"
    # Whole-tournament window for WC2026 (group stage June 11-27, final July 19).
    # We pad by a day on each side because ESPN buckets late kickoffs into the
    # following UTC date.
    DEFAULT_START = date(2026, 6, 10)
    DEFAULT_END = date(2026, 7, 20)

    def __init__(
        self,
        output_dir: Path,
        schedule_path: Path,
        team_mapping_path: Path,
        groups_path: Path | None = None,
        venues_path: Path | None = None,
        league: str | None = None,
    ) -> None:
        super().__init__(output_dir)
        self.league = league or self.DEFAULT_LEAGUE
        self.schedule = self._load_json(schedule_path)
        self.team_mapping = self._load_json(team_mapping_path)
        # Group composition is needed to resolve schedule placeholders ("A1")
        # to team IDs. Fall back to the committed team_mapping.json, which
        # carries the same draw, so a separate groups.json is optional (the
        # generated one is gitignored and absent in CI).
        if groups_path is not None:
            self.groups = self._load_json(groups_path).get("groups", {})
        else:
            self.groups = self.team_mapping.get("groups", {})
        # Venues let us map completed knockout events to their scheduled
        # matchNumber by (date, venue). Optional: without it, knockout matches
        # on days with a single fixture still map unambiguously.
        if venues_path is not None:
            venues = self._load_json(venues_path)
            self.venues = venues.get("venues", venues) if isinstance(venues, dict) else venues
        else:
            self.venues = []
        self._name_to_id = self._build_name_index()
        self._pair_to_match = self._build_pair_index()
        self._venue_index = self._build_venue_index()
        self._ko_by_venue, self._ko_fixtures = self._build_knockout_index()

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

    @staticmethod
    def _normalize(value: str) -> str:
        """Lowercase and strip to alphanumerics for fuzzy name matching."""
        return "".join(ch for ch in value.lower() if ch.isalnum())

    def _build_venue_index(self) -> dict[str, str]:
        """Map normalized venue names and cities to our venue ids.

        ESPN venue naming drifts (sponsor renames, abbreviations), but each of
        our venues is in a distinct city, so city is the most reliable key. We
        index by stadium name and city so either can resolve a venue id.
        """
        index: dict[str, str] = {}
        for venue in self.venues:
            venue_id = venue.get("id")
            if not venue_id:
                continue
            name = venue.get("name", "")
            if name:
                index[self._normalize(name)] = venue_id
            city = venue.get("city", "")
            if city:
                index[self._normalize(city)] = venue_id
                # Also index the city name without its state/province suffix
                # ("East Rutherford, NJ" -> "East Rutherford").
                index[self._normalize(city.split(",")[0])] = venue_id
        # Explicit overrides win over derived keys.
        index.update(VENUE_NAME_OVERRIDES)
        return index

    def _build_knockout_index(
        self,
    ) -> tuple[dict[str, list[tuple[date, dict[str, Any]]]], list[tuple[date, dict[str, Any]]]]:
        """Index knockout fixtures by venue (and overall) with their dates.

        ``by_venue`` maps each venueId to its knockout fixtures paired with the
        scheduled date; ``all_fixtures`` is every knockout fixture. Matching
        uses a date tolerance rather than an exact key because ESPN buckets late
        kickoffs into the following UTC day, so a match's scoreboard date can be
        one day after its scheduled date.
        """
        by_venue: dict[str, list[tuple[date, dict[str, Any]]]] = {}
        all_fixtures: list[tuple[date, dict[str, Any]]] = []
        for match in self.schedule.get("matches", []):
            round_id = match.get("round")
            if not round_id or round_id == "group_stage":
                continue
            match_date = match.get("date")
            venue_id = match.get("venueId")
            if not match_date:
                continue
            try:
                date_obj = datetime.strptime(match_date, "%Y-%m-%d").date()
            except ValueError:
                continue
            entry = {
                "matchNumber": match.get("matchNumber"),
                "round": round_id,
                "venueId": venue_id,
                "date": match_date,
            }
            all_fixtures.append((date_obj, entry))
            if venue_id:
                by_venue.setdefault(venue_id, []).append((date_obj, entry))
        return by_venue, all_fixtures

    def _resolve_venue_id(self, event: dict) -> str | None:
        """Resolve an ESPN event's venue to one of our venue ids."""
        competition = event.get("competitions", [{}])[0]
        venue = competition.get("venue", {}) or {}
        candidates = [venue.get("fullName", ""), venue.get("shortName", "")]
        address = venue.get("address", {}) or {}
        candidates.append(address.get("city", ""))
        for candidate in candidates:
            if not candidate:
                continue
            venue_id = self._venue_index.get(self._normalize(candidate))
            if venue_id:
                return venue_id
        return None

    # Max gap between an event's scoreboard date and its scheduled date. ESPN
    # buckets late kickoffs into the following UTC day, so allow one day.
    _DATE_TOLERANCE = timedelta(days=1)

    def _scheduled_knockout(self, event_date: date, event: dict) -> dict[str, Any] | None:
        """Find the schedule entry for a completed knockout event, if any.

        Matches on venue (unique per bracket), picking the fixture at that venue
        whose date is nearest the event's — within one day, to absorb ESPN's
        next-day bucketing of late kickoffs.
        """
        venue_id = self._resolve_venue_id(event)
        if venue_id is None:
            return None
        near = [
            (abs(fixture_date - event_date), entry)
            for fixture_date, entry in self._ko_by_venue.get(venue_id, [])
            if abs(fixture_date - event_date) <= self._DATE_TOLERANCE
        ]
        near.sort(key=lambda item: item[0])
        # Accept the nearest fixture as long as it's unambiguous.
        if len(near) == 1 or (len(near) > 1 and near[0][0] != near[1][0]):
            return near[0][1]
        return None

    def _in_knockout_window(self, event_date: date) -> bool:
        """Whether a date falls within the knockout stage (± the tolerance)."""
        if not self._ko_fixtures:
            return False
        dates = [fixture_date for fixture_date, _ in self._ko_fixtures]
        return (
            min(dates) - self._DATE_TOLERANCE
            <= event_date
            <= max(dates) + self._DATE_TOLERANCE
        )

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
                "winner": bool(c.get("winner")),
            }

        home, away = sides.get("home"), sides.get("away")
        if not home or not away:
            return None
        if home["team_id"] is None or away["team_id"] is None:
            unresolved = [s["name"] for s in (home, away) if s["team_id"] is None]
            self.logger.warning(f"Could not resolve team(s): {', '.join(unresolved)}")
            return None

        match_date = (event.get("date", "") or "")[:10]
        try:
            event_date = datetime.strptime(match_date, "%Y-%m-%d").date()
        except ValueError:
            return None

        # Knockout matches can't be matched by team pair (teams depend on
        # earlier results), so try mapping by venue first. Checking knockout
        # first also correctly handles a knockout rematch of a group pairing,
        # which would otherwise collide with the group pair index.
        knockout = self._scheduled_knockout(event_date, event)
        if knockout is not None:
            return self._knockout_entry(knockout, home, away)

        scheduled = self._pair_to_match.get(
            frozenset({home["team_id"], away["team_id"]})
        )
        if not scheduled:
            # Neither a knockout fixture nor a group pairing we track. A
            # completed, fully-resolved event inside the knockout window that
            # lands here usually means an unrecognized venue name — surface it
            # rather than dropping the result silently.
            if self._in_knockout_window(event_date):
                venue = (
                    event.get("competitions", [{}])[0].get("venue", {}) or {}
                ).get("fullName")
                self.logger.warning(
                    f"Completed knockout-window event not mapped to a fixture: "
                    f"{home['name']} vs {away['name']} on {match_date} "
                    f"(venue={venue!r})"
                )
            return None

        # Orient the scraped score to the schedule's home/away ordering.
        if home["team_id"] == scheduled["homeTeamId"]:
            home_goals, away_goals = home["goals"], away["goals"]
        else:
            home_goals, away_goals = away["goals"], home["goals"]

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

    def _knockout_entry(self, scheduled: dict, home: dict, away: dict) -> dict | None:
        """Build a knockout results entry, recording the winning team.

        The web app pins knockout matches as winner-only fixed results, so the
        winner is what matters. Prefer ESPN's ``winner`` flag (which accounts
        for extra time and penalties); fall back to the higher score.
        """
        if home["winner"] and not away["winner"]:
            winner_id = home["team_id"]
        elif away["winner"] and not home["winner"]:
            winner_id = away["team_id"]
        elif home["goals"] != away["goals"]:
            winner_id = home["team_id"] if home["goals"] > away["goals"] else away["team_id"]
        else:
            # Level score with no winner flag — likely undecided penalty data.
            self.logger.warning(
                f"Could not determine knockout winner for match "
                f"{scheduled['matchNumber']} ({home['name']} vs {away['name']})"
            )
            return None

        return {
            "matchNumber": scheduled["matchNumber"],
            "round": scheduled["round"],
            "homeTeamId": home["team_id"],
            "awayTeamId": away["team_id"],
            "homeScore": home["goals"],
            "awayScore": away["goals"],
            "winnerTeamId": winner_id,
            "status": "completed",
            # Use the scheduled date, not ESPN's (which buckets late kickoffs
            # into the next UTC day).
            "date": scheduled["date"],
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
        group_count = sum(1 for m in matches if "groupId" in m)
        knockout_count = len(matches) - group_count
        self.logger.info(
            f"Resolved {len(matches)} completed matches "
            f"({group_count} group, {knockout_count} knockout)"
        )

        return {
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "source": f"espn:{self.league}",
            "matches": matches,
        }
