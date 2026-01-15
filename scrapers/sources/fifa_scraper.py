"""FIFA world rankings scraper."""

import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import requests
from bs4 import BeautifulSoup

from scrapers.sources.base import BaseScraper, ScraperError
from scrapers.config.settings import TIMEOUT


# FIFA API endpoints to try (in order of preference)
FIFA_API_ENDPOINTS = [
    "https://cxm-api.fifa.com/fifaplusweb/api/ranking/overview",
    "https://www.fifa.com/api/ranking-overview?locale=en",
    "https://api.fifa.com/api/v3/rankings/men",
]

# Fallback HTML page URL
FIFA_RANKINGS_PAGE = "https://www.fifa.com/fifa-world-ranking/men"

# Wikipedia fallback
WIKIPEDIA_RANKINGS_PAGE = "https://en.wikipedia.org/wiki/FIFA_Men%27s_World_Ranking"

# Headers required by FIFA API
FIFA_API_HEADERS = {
    "Accept": "application/json",
    "Origin": "https://www.fifa.com",
    "Referer": "https://www.fifa.com/fifa-world-ranking/men",
}


class FifaScraper(BaseScraper):
    """Scraper for FIFA world rankings."""

    def __init__(self, output_dir: Path) -> None:
        """Initialize the FIFA scraper.

        Args:
            output_dir: Directory where output files will be saved.
        """
        super().__init__(output_dir)

    def get_output_filename(self) -> str:
        """Get the output filename for FIFA rankings.

        Returns:
            The filename for the FIFA rankings JSON file.
        """
        return "fifa_rankings.json"

    def scrape(self) -> dict[str, Any]:
        """Scrape FIFA world rankings.

        Tries multiple API endpoints, falling back to HTML scraping if needed.

        Returns:
            Dict with team rankings, source, and timestamp.

        Raises:
            ScraperError: If all scraping methods fail.
        """
        self.logger.info("Starting FIFA rankings scrape")

        # Try API endpoints first
        for endpoint in FIFA_API_ENDPOINTS:
            try:
                self.logger.info(f"Trying API endpoint: {endpoint}")
                rankings = self._fetch_from_api(endpoint)
                if rankings:
                    return self._format_output(rankings, "fifa.com/api")
            except Exception as e:
                self.logger.warning(f"API endpoint {endpoint} failed: {e}")
                continue

        # Fall back to HTML scraping
        self.logger.info("API endpoints failed, trying HTML scrape")
        try:
            rankings = self._scrape_from_html()
            if rankings:
                return self._format_output(rankings, "fifa.com/html")
        except Exception as e:
            self.logger.error(f"HTML scrape failed: {e}")

        # Final fallback: Wikipedia
        self.logger.info("FIFA sources failed, trying Wikipedia")
        try:
            rankings = self._scrape_from_wikipedia()
            if rankings:
                return self._format_output(rankings, "wikipedia.org")
        except Exception as e:
            self.logger.error(f"Wikipedia scrape failed: {e}")

        self.fail_fast(
            "Failed to fetch FIFA rankings from all sources. "
            "The FIFA API may have changed or be temporarily unavailable. "
            "Please check https://www.fifa.com/fifa-world-ranking/men manually."
        )

    def _fetch_from_api(self, endpoint: str) -> dict[str, int] | None:
        """Fetch rankings from a FIFA API endpoint.

        Args:
            endpoint: The API URL to fetch from.

        Returns:
            Dict mapping team name to ranking position, or None if failed.
        """
        headers = {**self.session.headers, **FIFA_API_HEADERS}

        response = self.session.get(endpoint, headers=headers, timeout=TIMEOUT)
        response.raise_for_status()

        data = response.json()
        return self._parse_api_response(data)

    def _parse_api_response(self, data: Any) -> dict[str, int] | None:
        """Parse FIFA API JSON response to extract rankings.

        Handles various API response formats.

        Args:
            data: The JSON response data.

        Returns:
            Dict mapping team name to ranking position, or None if parsing fails.
        """
        rankings: dict[str, int] = {}

        # Try common response structures
        # Format 1: {"rankings": [{"rank": 1, "team": {"name": "Argentina"}}]}
        if isinstance(data, dict) and "rankings" in data:
            for entry in data.get("rankings", []):
                rank = entry.get("rank") or entry.get("position")
                team_data = entry.get("team") or entry.get("country")
                if isinstance(team_data, dict):
                    name = team_data.get("name") or team_data.get("teamName")
                elif isinstance(team_data, str):
                    name = team_data
                else:
                    continue
                if rank and name:
                    rankings[name] = int(rank)

        # Format 2: {"data": [{"rank": 1, "teamName": "Argentina"}]}
        elif isinstance(data, dict) and "data" in data:
            for entry in data.get("data", []):
                rank = entry.get("rank") or entry.get("position")
                name = entry.get("teamName") or entry.get("name") or entry.get("team")
                if rank and name:
                    rankings[name] = int(rank)

        # Format 3: Direct array [{"rank": 1, "team": "Argentina"}]
        elif isinstance(data, list):
            for entry in data:
                if isinstance(entry, dict):
                    rank = entry.get("rank") or entry.get("position")
                    name = (
                        entry.get("team")
                        or entry.get("teamName")
                        or entry.get("name")
                        or entry.get("country")
                    )
                    if isinstance(name, dict):
                        name = name.get("name") or name.get("teamName")
                    if rank and name:
                        rankings[name] = int(rank)

        if rankings:
            self.logger.info(f"Parsed {len(rankings)} teams from API response")
            return rankings

        self.logger.warning("Could not parse rankings from API response structure")
        return None

    def _scrape_from_html(self) -> dict[str, int] | None:
        """Scrape rankings from FIFA HTML page as fallback.

        Returns:
            Dict mapping team name to ranking position, or None if failed.
        """
        headers = {**self.session.headers}
        headers["Accept"] = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"

        response = self.session.get(FIFA_RANKINGS_PAGE, headers=headers, timeout=TIMEOUT)
        response.raise_for_status()

        soup = BeautifulSoup(response.text, "html.parser")
        rankings: dict[str, int] = {}

        # Look for ranking table rows
        # FIFA uses various class patterns, try multiple selectors
        selectors = [
            "tr[data-team-name]",
            ".ranking-item",
            ".rank-table-row",
            "[class*='RankingItem']",
            "[class*='ranking-table'] tr",
        ]

        for selector in selectors:
            elements = soup.select(selector)
            if elements:
                self.logger.debug(f"Found {len(elements)} elements with selector: {selector}")
                for elem in elements:
                    rank, name = self._extract_rank_from_element(elem)
                    if rank and name:
                        rankings[name] = rank

                if rankings:
                    break

        # Try extracting from JSON-LD or embedded JSON
        if not rankings:
            rankings = self._extract_from_embedded_json(soup)

        if rankings:
            self.logger.info(f"Scraped {len(rankings)} teams from HTML")
            return rankings

        return None

    def _scrape_from_wikipedia(self) -> dict[str, int] | None:
        """Scrape rankings from Wikipedia as final fallback.

        Returns:
            Dict mapping team name to ranking position, or None if failed.
        """
        response = self.session.get(WIKIPEDIA_RANKINGS_PAGE, timeout=TIMEOUT)
        response.raise_for_status()

        soup = BeautifulSoup(response.text, "lxml")
        rankings: dict[str, int] = {}

        # Find wikitables - the first one has current rankings
        tables = soup.find_all("table", class_="wikitable")
        if not tables:
            return None

        # Parse the ranking table (skip header rows)
        for row in tables[0].find_all("tr")[2:]:
            cells = row.find_all("td")
            if len(cells) >= 3:
                rank_text = cells[0].get_text(strip=True)
                team_cell = cells[2]  # Team is in 3rd column

                # Get team name from link if available
                link = team_cell.find("a")
                if link:
                    team_name = link.get_text(strip=True)
                else:
                    team_name = team_cell.get_text(strip=True)

                if rank_text.isdigit() and team_name:
                    rankings[team_name] = int(rank_text)

        if rankings:
            self.logger.info(f"Scraped {len(rankings)} teams from Wikipedia")
            return rankings

        return None

    def _extract_rank_from_element(self, elem: Any) -> tuple[int | None, str | None]:
        """Extract rank and team name from an HTML element.

        Args:
            elem: BeautifulSoup element to parse.

        Returns:
            Tuple of (rank, team_name) or (None, None) if not found.
        """
        rank = None
        name = None

        # Try data attributes
        if elem.get("data-team-name"):
            name = elem.get("data-team-name")
        if elem.get("data-rank"):
            try:
                rank = int(elem.get("data-rank"))
            except (ValueError, TypeError):
                pass

        # Try common class patterns for rank
        rank_elem = elem.select_one(".rank, .position, [class*='Rank'], [class*='position']")
        if rank_elem:
            try:
                rank_text = rank_elem.get_text(strip=True)
                rank = int(re.sub(r"\D", "", rank_text))
            except (ValueError, TypeError):
                pass

        # Try common class patterns for team name
        name_elem = elem.select_one(
            ".team-name, .country-name, [class*='TeamName'], [class*='countryName']"
        )
        if name_elem:
            name = name_elem.get_text(strip=True)

        # Fallback: look for text content patterns
        if not name or not rank:
            text = elem.get_text(" ", strip=True)
            # Pattern like "1 Argentina" or "Argentina 1"
            match = re.match(r"^(\d+)\s+([A-Za-z\s]+)", text)
            if match:
                rank = int(match.group(1))
                name = match.group(2).strip()

        return rank, name

    def _extract_from_embedded_json(self, soup: BeautifulSoup) -> dict[str, int]:
        """Extract rankings from embedded JSON/JavaScript in the page.

        Args:
            soup: Parsed HTML page.

        Returns:
            Dict mapping team name to ranking, or empty dict if not found.
        """
        rankings: dict[str, int] = {}

        # Look for script tags with ranking data
        scripts = soup.find_all("script")
        for script in scripts:
            if not script.string:
                continue

            # Look for JSON-like structures with ranking data
            # Pattern: {"rank":1,"team":"Argentina"} or similar
            matches = re.findall(
                r'"(?:rank|position)":\s*(\d+).*?"(?:team|teamName|name|country)":\s*"([^"]+)"',
                script.string,
            )
            for rank_str, name in matches:
                try:
                    rankings[name] = int(rank_str)
                except ValueError:
                    continue

            # Also try reverse pattern
            matches = re.findall(
                r'"(?:team|teamName|name|country)":\s*"([^"]+)".*?"(?:rank|position)":\s*(\d+)',
                script.string,
            )
            for name, rank_str in matches:
                try:
                    if name not in rankings:
                        rankings[name] = int(rank_str)
                except ValueError:
                    continue

        return rankings

    def _format_output(self, rankings: dict[str, int], source: str) -> dict[str, Any]:
        """Format the rankings data for output.

        Args:
            rankings: Dict mapping team name to ranking position.
            source: Description of the data source.

        Returns:
            Formatted output dict with teams, source, and timestamp.
        """
        return {
            "teams": rankings,
            "source": source,
            "scraped_at": datetime.now(timezone.utc).isoformat(),
        }
