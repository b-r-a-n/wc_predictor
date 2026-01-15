"""ELO ratings scraper from international-football.net."""

import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bs4 import BeautifulSoup

from .base import BaseScraper


class EloScraper(BaseScraper):
    """Scraper for ELO ratings from international-football.net.

    Note: eloratings.net uses JavaScript rendering which requires a browser.
    international-football.net provides the same ELO ratings with static HTML.
    """

    BASE_URL = "https://www.international-football.net/country"
    RATE_LIMIT_DELAY = 1.0  # seconds between requests

    def __init__(self, output_dir: Path, team_names: list[str] | None = None) -> None:
        """Initialize the ELO scraper.

        Args:
            output_dir: Directory where output files will be saved.
            team_names: Optional list of team names to scrape. If None, must be provided to scrape().
        """
        super().__init__(output_dir)
        self.team_names = team_names or []
        self._last_request_time = 0.0

    def get_output_filename(self) -> str:
        """Get the output filename for this scraper.

        Returns:
            The filename for the ELO ratings output file.
        """
        return "elo_ratings.json"

    def _rate_limit(self) -> None:
        """Enforce rate limiting between requests."""
        elapsed = time.time() - self._last_request_time
        if elapsed < self.RATE_LIMIT_DELAY:
            time.sleep(self.RATE_LIMIT_DELAY - elapsed)
        self._last_request_time = time.time()

    def scrape(self, team_names: list[str] | None = None) -> dict[str, Any]:
        """Scrape ELO ratings for specified teams.

        Args:
            team_names: List of team names to scrape. Uses instance team_names if not provided.

        Returns:
            Dictionary containing teams with their ELO ratings,
            source information, and timestamp.

        Raises:
            ScraperError: If scraping fails.
        """
        teams_to_scrape = team_names or self.team_names
        if not teams_to_scrape:
            self.fail_fast("No team names provided to scrape")

        self.logger.info(f"Scraping ELO ratings for {len(teams_to_scrape)} teams from international-football.net")

        teams: dict[str, int] = {}

        for team_name in teams_to_scrape:
            self._rate_limit()
            rating = self._scrape_team(team_name)
            if rating is not None:
                teams[team_name] = rating
                self.logger.info(f"  {team_name}: {rating}")
            else:
                self.logger.warning(f"  {team_name}: No rating found")

        if not teams:
            self.fail_fast("No ELO ratings found for any team")

        self.logger.info(f"Successfully scraped {len(teams)} team ratings")

        return {
            "teams": teams,
            "source": "international-football.net",
            "scraped_at": datetime.now(timezone.utc).isoformat(),
        }

    def _scrape_team(self, team_name: str) -> int | None:
        """Scrape ELO rating for a single team.

        Args:
            team_name: Name of the team (e.g., "Argentina", "France").

        Returns:
            ELO rating as integer, or None if not found.
        """
        url = f"{self.BASE_URL}?team={team_name}"

        try:
            response = self.session.get(url, timeout=30)
            response.raise_for_status()
        except Exception as e:
            self.logger.error(f"Failed to fetch {team_name}: {e}")
            return None

        return self._parse_elo_from_page(response.text, team_name)

    def _parse_elo_from_page(self, html_content: str, team_name: str) -> int | None:
        """Parse ELO rating from team page HTML.

        The page contains a table with rows like:
        ['Elo Score', '2113']
        ['Elo Ranking', '2nd']

        Args:
            html_content: HTML content of the team page.
            team_name: Name of the team (for logging).

        Returns:
            ELO rating as integer, or None if not found.
        """
        soup = BeautifulSoup(html_content, "lxml")

        # Find tables and look for 'Elo Score' row
        for table in soup.find_all("table"):
            for row in table.find_all("tr"):
                cells = row.find_all(["td", "th"])
                if len(cells) >= 2:
                    label = cells[0].get_text(strip=True)
                    value = cells[1].get_text(strip=True)

                    if label == "Elo Score" and value.isdigit():
                        rating = int(value)
                        if 1000 <= rating <= 2500:
                            return rating

        # Fallback: search for pattern in text
        import re
        match = re.search(r"Elo\s*Score[^\d]*(\d{4})", html_content)
        if match:
            rating = int(match.group(1))
            if 1000 <= rating <= 2500:
                return rating

        self.logger.warning(f"Could not parse ELO rating for {team_name}")
        return None
