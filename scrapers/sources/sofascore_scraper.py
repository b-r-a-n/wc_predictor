"""Sofascore form ratings scraper."""

import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .base import BaseScraper


class SofascoreScraper(BaseScraper):
    """Scraper for team form data from Sofascore.

    Fetches recent match results for each team and calculates a form score
    based on points per game (Win=3, Draw=1, Loss=0).
    """

    BASE_URL = "https://api.sofascore.com/api/v1"
    RATE_LIMIT_DELAY = 2.0  # seconds between requests

    def __init__(self, output_dir: Path, team_ids: dict[str, int] | None = None) -> None:
        """Initialize the Sofascore scraper.

        Args:
            output_dir: Directory where output files will be saved.
            team_ids: Optional dict mapping canonical team names to Sofascore IDs.
        """
        super().__init__(output_dir)
        self.team_ids = team_ids or {}
        self._last_request_time = 0.0

    def get_output_filename(self) -> str:
        """Get the output filename for this scraper.

        Returns:
            The filename for the Sofascore form output file.
        """
        return "sofascore_form.json"

    def _rate_limit(self) -> None:
        """Enforce rate limiting between requests."""
        elapsed = time.time() - self._last_request_time
        if elapsed < self.RATE_LIMIT_DELAY:
            time.sleep(self.RATE_LIMIT_DELAY - elapsed)
        self._last_request_time = time.time()

    def scrape(self, team_ids: dict[str, int] | None = None) -> dict[str, Any]:
        """Scrape form data for specified teams.

        Args:
            team_ids: Dict mapping canonical team names to Sofascore IDs.
                     Uses instance team_ids if not provided.

        Returns:
            Dictionary containing teams with their form scores,
            source information, and timestamp.

        Raises:
            ScraperError: If scraping fails.
        """
        teams_to_scrape = team_ids or self.team_ids
        if not teams_to_scrape:
            self.fail_fast("No team IDs provided to scrape")

        self.logger.info(f"Scraping Sofascore form for {len(teams_to_scrape)} teams")

        teams: dict[str, float] = {}
        matches_data: dict[str, dict] = {}

        for team_name, sofascore_id in teams_to_scrape.items():
            self._rate_limit()
            form_score, match_info = self._scrape_team_form(team_name, sofascore_id)
            if form_score is not None:
                teams[team_name] = round(form_score, 2)
                matches_data[team_name] = match_info
                self.logger.info(f"  {team_name}: {form_score:.2f} (from {match_info['matches_used']} matches)")
            else:
                self.logger.warning(f"  {team_name}: No form data found")

        if not teams:
            self.fail_fast("No form data found for any team")

        self.logger.info(f"Successfully scraped {len(teams)} team form scores")

        return {
            "teams": teams,
            "matches_info": matches_data,
            "source": "sofascore.com",
            "scraped_at": datetime.now(timezone.utc).isoformat(),
        }

    def _scrape_team_form(self, team_name: str, sofascore_id: int) -> tuple[float | None, dict]:
        """Scrape form score for a single team.

        Args:
            team_name: Canonical name of the team.
            sofascore_id: Sofascore's internal team ID.

        Returns:
            Tuple of (form_score, match_info) or (None, {}) if not found.
            Form score is average points per game (0.0 - 3.0).
        """
        url = f"{self.BASE_URL}/team/{sofascore_id}/events/last/0"

        headers = {
            "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            "Accept": "application/json",
            "Referer": "https://www.sofascore.com/",
        }

        try:
            response = self.session.get(url, headers=headers, timeout=30)
            response.raise_for_status()
            data = response.json()
        except Exception as e:
            self.logger.error(f"Failed to fetch {team_name} (ID: {sofascore_id}): {e}")
            return None, {}

        return self._calculate_form(data, sofascore_id, team_name)

    def _calculate_form(
        self, data: dict, team_id: int, team_name: str
    ) -> tuple[float | None, dict]:
        """Calculate form score from match data.

        Args:
            data: API response containing events.
            team_id: Sofascore team ID.
            team_name: Team name for logging.

        Returns:
            Tuple of (form_score, match_info).
            Form score is average points per game from last 10 matches.
        """
        events = data.get("events", [])
        if not events:
            self.logger.warning(f"No events found for {team_name}")
            return None, {}

        points = []
        results = []

        # Process last 10 completed matches
        for event in events[:10]:
            # Skip if match not finished
            status = event.get("status", {})
            if status.get("type") != "finished":
                continue

            home_team = event.get("homeTeam", {})
            away_team = event.get("awayTeam", {})
            home_score_data = event.get("homeScore", {})
            away_score_data = event.get("awayScore", {})

            # Get current/final score
            home_score = home_score_data.get("current")
            away_score = away_score_data.get("current")

            if home_score is None or away_score is None:
                continue

            is_home = home_team.get("id") == team_id

            if is_home:
                if home_score > away_score:
                    points.append(3)
                    results.append("W")
                elif home_score == away_score:
                    points.append(1)
                    results.append("D")
                else:
                    points.append(0)
                    results.append("L")
            else:
                if away_score > home_score:
                    points.append(3)
                    results.append("W")
                elif away_score == home_score:
                    points.append(1)
                    results.append("D")
                else:
                    points.append(0)
                    results.append("L")

        if not points:
            return None, {}

        form_score = sum(points) / len(points)
        match_info = {
            "matches_used": len(points),
            "results": "".join(results),
            "wins": results.count("W"),
            "draws": results.count("D"),
            "losses": results.count("L"),
            "total_points": sum(points),
        }

        return form_score, match_info
