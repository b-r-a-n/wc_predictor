"""Transfermarkt market value scraper for national teams."""

import re
import time
from pathlib import Path
from typing import Any

import cloudscraper
from bs4 import BeautifulSoup

from scrapers.sources.base import BaseScraper, ScraperError


class TransfermarktScraper(BaseScraper):
    """Scraper for national team market values from Transfermarkt."""

    BASE_URL = "https://www.transfermarkt.us"
    RATE_LIMIT_DELAY = 2.0  # seconds between requests

    def __init__(self, output_dir: Path) -> None:
        """Initialize the Transfermarkt scraper.

        Args:
            output_dir: Directory where output files will be saved.
        """
        super().__init__(output_dir)
        # Use cloudscraper to bypass Cloudflare protection
        self.session = cloudscraper.create_scraper(
            browser={
                "browser": "chrome",
                "platform": "darwin",
                "desktop": True,
            }
        )
        self.session.headers.update(
            {
                "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
                "Accept-Language": "en-US,en;q=0.9",
                "Accept-Encoding": "gzip, deflate, br",
                "Connection": "keep-alive",
                "Upgrade-Insecure-Requests": "1",
                "Sec-Fetch-Dest": "document",
                "Sec-Fetch-Mode": "navigate",
                "Sec-Fetch-Site": "none",
                "Sec-Fetch-User": "?1",
                "Cache-Control": "max-age=0",
            }
        )
        self._last_request_time: float = 0.0

    def _rate_limit(self) -> None:
        """Enforce rate limiting between requests."""
        now = time.time()
        elapsed = now - self._last_request_time
        if elapsed < self.RATE_LIMIT_DELAY:
            time.sleep(self.RATE_LIMIT_DELAY - elapsed)
        self._last_request_time = time.time()

    def _build_url(self, team_slug: str, team_id: int) -> str:
        """Build the URL for a team's page.

        Args:
            team_slug: The Transfermarkt slug for the team (e.g., "argentinien").
            team_id: The Transfermarkt ID for the team.

        Returns:
            The full URL for the team's page.
        """
        return f"{self.BASE_URL}/{team_slug}/startseite/verein/{team_id}"

    def _parse_market_value(self, value_str: str) -> float:
        """Parse a market value string into millions.

        Args:
            value_str: Value string like "EUR1.54bn" or "EUR795.00m" or "EUR30.50k".

        Returns:
            The value in millions (e.g., 1540.0 for "EUR1.54bn", 795.0 for "EUR795.00m").

        Raises:
            ScraperError: If the value format is unrecognized.
        """
        # Clean up the string - remove currency symbols and whitespace
        cleaned = value_str.strip().replace(",", ".")

        # Match patterns like "EUR1.54bn", "EUR795.00m", "EUR30.50k", "$1.54bn"
        pattern = r"[€$£]?\s*(\d+(?:\.\d+)?)\s*(bn|m|k|million|billion|thousand)?"
        match = re.search(pattern, cleaned, re.IGNORECASE)

        if not match:
            self.fail_fast(f"Could not parse market value: {value_str}")

        value = float(match.group(1))
        suffix = (match.group(2) or "").lower()

        if suffix in ("bn", "billion"):
            return value * 1000.0  # Convert billions to millions
        elif suffix in ("m", "million", ""):
            return value  # Already in millions
        elif suffix in ("k", "thousand"):
            return value / 1000.0  # Convert thousands to millions
        else:
            self.fail_fast(f"Unrecognized value suffix in: {value_str}")

    def _fetch_team_value(self, team_slug: str, team_id: int) -> float:
        """Fetch the market value for a single team.

        Args:
            team_slug: The Transfermarkt slug for the team.
            team_id: The Transfermarkt ID for the team.

        Returns:
            The team's total squad market value in millions.

        Raises:
            ScraperError: If the request fails or parsing fails.
        """
        self._rate_limit()

        url = self._build_url(team_slug, team_id)
        self.logger.info(f"Fetching {url}")

        try:
            response = self.session.get(url, timeout=30)
        except Exception as e:
            self.fail_fast(f"Request failed for {team_slug}: {e}")

        if response.status_code != 200:
            self.fail_fast(
                f"HTTP {response.status_code} for {team_slug} ({url})"
            )

        soup = BeautifulSoup(response.text, "lxml")

        # The total market value is in the header section
        # Look for the data-market-value attribute or the market value display
        value_element = soup.select_one(
            "a.data-header__market-value-wrapper"
        )

        if value_element:
            # Extract text and parse - format is usually "€795.00m"
            value_text = value_element.get_text(strip=True)
            self.logger.debug(f"Found market value text: {value_text}")
            return self._parse_market_value(value_text)

        # Alternative selector - look in the box with "Total market value"
        value_box = soup.select_one(".data-header__box--small .data-header__content")
        if value_box:
            value_text = value_box.get_text(strip=True)
            self.logger.debug(f"Found market value in box: {value_text}")
            return self._parse_market_value(value_text)

        # Try another common pattern
        market_value_span = soup.select_one("span.data-header__market-value")
        if market_value_span:
            value_text = market_value_span.get_text(strip=True)
            self.logger.debug(f"Found market value span: {value_text}")
            return self._parse_market_value(value_text)

        self.fail_fast(
            f"Could not find market value element for {team_slug}. "
            f"Page may have changed structure."
        )

    def scrape_team(self, canonical_name: str, team_slug: str, team_id: int) -> tuple[str, float]:
        """Scrape market value for a single team.

        Args:
            canonical_name: The canonical name of the team.
            team_slug: The Transfermarkt slug for the team.
            team_id: The Transfermarkt ID for the team.

        Returns:
            Tuple of (canonical_name, market_value_in_millions).

        Raises:
            ScraperError: If scraping fails.
        """
        value = self._fetch_team_value(team_slug, team_id)
        self.logger.info(f"{canonical_name}: {value:.2f}m")
        return canonical_name, value

    def scrape(self) -> Any:
        """Execute the scraping operation.

        This method is not used directly - use scrape_team() instead
        for per-team scraping controlled by the CLI.

        Returns:
            Empty dict - scraping is done per-team via scrape_team().
        """
        return {}

    def get_output_filename(self) -> str:
        """Get the output filename for this scraper.

        Returns:
            The filename for the output file.
        """
        return "transfermarkt_values.json"
