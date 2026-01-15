"""World Cup 2026 group assignments scraper."""

import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bs4 import BeautifulSoup

from scrapers.config.settings import TIMEOUT
from scrapers.sources.base import BaseScraper, ScraperError


class GroupsScraper(BaseScraper):
    """Scraper for FIFA World Cup 2026 group assignments.

    Fetches official group draw data from FIFA website and validates
    against existing team_mapping.json data.
    """

    # FIFA URLs for group draw information
    FIFA_TEAMS_URL = (
        "https://www.fifa.com/fifaplus/en/tournaments/mens/worldcup/"
        "canadamexicousa2026/teams"
    )
    FIFA_DRAW_URL = (
        "https://www.fifa.com/en/tournaments/mens/worldcup/"
        "canadamexicousa2026/articles/final-draw-results"
    )

    # Expected groups A-L
    GROUP_LETTERS = [chr(ord("A") + i) for i in range(12)]
    TEAMS_PER_GROUP = 4

    def __init__(self, output_dir: Path) -> None:
        """Initialize the groups scraper.

        Args:
            output_dir: Directory where output files will be saved.
        """
        super().__init__(output_dir)
        self._team_mapping_path = (
            Path(__file__).parent.parent / "config" / "team_mapping.json"
        )

    def get_output_filename(self) -> str:
        """Get the output filename for groups data.

        Returns:
            The filename for the groups output file.
        """
        return "groups.json"

    def scrape(self) -> dict[str, Any]:
        """Execute the scraping operation for group assignments.

        Attempts to fetch data from FIFA website. If that fails or returns
        incomplete data, falls back to validating existing team_mapping.json.

        Returns:
            Dictionary containing groups data with metadata.

        Raises:
            ScraperError: If scraping fails and no valid fallback data exists.
        """
        self.logger.info("Starting World Cup 2026 groups scraper")

        # Try FIFA website first
        groups_data = self._try_scrape_fifa()

        if groups_data and self._validate_groups(groups_data):
            self.logger.info("Successfully scraped groups from FIFA website")
            return self._format_output(groups_data, source="fifa.com")

        # Fall back to team_mapping.json
        self.logger.warning(
            "Could not scrape FIFA website, falling back to team_mapping.json"
        )
        groups_data = self._load_from_team_mapping()

        if not groups_data:
            self.fail_fast("No valid groups data found from any source")

        if not self._validate_groups(groups_data):
            self.fail_fast("Groups data from team_mapping.json is invalid")

        self.logger.info("Using groups from team_mapping.json")
        return self._format_output(groups_data, source="team_mapping.json")

    def _try_scrape_fifa(self) -> dict[str, list[str]] | None:
        """Attempt to scrape groups from FIFA website.

        Returns:
            Dictionary mapping group letters to team lists, or None if failed.
        """
        try:
            self.logger.info(f"Fetching {self.FIFA_TEAMS_URL}")
            response = self.session.get(self.FIFA_TEAMS_URL, timeout=TIMEOUT)
            response.raise_for_status()

            groups = self._parse_fifa_teams_page(response.text)
            if groups:
                return groups

            # Try the draw results page as fallback
            self.logger.info(f"Trying draw results page: {self.FIFA_DRAW_URL}")
            response = self.session.get(self.FIFA_DRAW_URL, timeout=TIMEOUT)
            response.raise_for_status()

            return self._parse_fifa_draw_page(response.text)

        except Exception as e:
            self.logger.warning(f"Failed to scrape FIFA website: {e}")
            return None

    def _parse_fifa_teams_page(self, html: str) -> dict[str, list[str]] | None:
        """Parse FIFA teams page for group assignments.

        Args:
            html: Raw HTML content from FIFA teams page.

        Returns:
            Dictionary of groups or None if parsing fails.
        """
        soup = BeautifulSoup(html, "lxml")
        groups: dict[str, list[str]] = {}

        # Look for group sections - FIFA typically uses data attributes or specific classes
        # Try various selectors that FIFA has used historically

        # Pattern 1: Look for elements with group data
        group_elements = soup.find_all(
            attrs={"data-group": re.compile(r"^[A-L]$")}
        )
        if group_elements:
            for elem in group_elements:
                group_letter = elem.get("data-group")
                teams = self._extract_team_names(elem)
                if teams and len(teams) == self.TEAMS_PER_GROUP:
                    groups[group_letter] = teams

        if len(groups) == len(self.GROUP_LETTERS):
            return groups

        # Pattern 2: Look for section headers with "Group X"
        group_headers = soup.find_all(
            string=re.compile(r"Group\s+[A-L]", re.IGNORECASE)
        )
        for header in group_headers:
            match = re.search(r"Group\s+([A-L])", header, re.IGNORECASE)
            if match:
                group_letter = match.group(1).upper()
                # Find the parent container and extract team names
                parent = header.find_parent(["div", "section", "article"])
                if parent:
                    teams = self._extract_team_names(parent)
                    if teams and len(teams) >= self.TEAMS_PER_GROUP:
                        groups[group_letter] = teams[: self.TEAMS_PER_GROUP]

        if len(groups) == len(self.GROUP_LETTERS):
            return groups

        # Pattern 3: Look for JSON-LD structured data
        scripts = soup.find_all("script", type="application/ld+json")
        for script in scripts:
            try:
                data = json.loads(script.string)
                parsed = self._extract_groups_from_json_ld(data)
                if parsed and len(parsed) == len(self.GROUP_LETTERS):
                    return parsed
            except (json.JSONDecodeError, TypeError):
                continue

        self.logger.debug(f"Could only parse {len(groups)} groups from teams page")
        return None if len(groups) < len(self.GROUP_LETTERS) else groups

    def _parse_fifa_draw_page(self, html: str) -> dict[str, list[str]] | None:
        """Parse FIFA draw results page for group assignments.

        Args:
            html: Raw HTML content from FIFA draw results page.

        Returns:
            Dictionary of groups or None if parsing fails.
        """
        soup = BeautifulSoup(html, "lxml")
        groups: dict[str, list[str]] = {}

        # Draw results pages often have tables or structured lists
        # Look for table-based layouts
        tables = soup.find_all("table")
        for table in tables:
            rows = table.find_all("tr")
            for row in rows:
                cells = row.find_all(["td", "th"])
                if cells:
                    first_cell = cells[0].get_text(strip=True)
                    match = re.match(r"^Group\s+([A-L])$", first_cell, re.IGNORECASE)
                    if match:
                        group_letter = match.group(1).upper()
                        teams = [
                            cell.get_text(strip=True)
                            for cell in cells[1:]
                            if cell.get_text(strip=True)
                        ]
                        if len(teams) >= self.TEAMS_PER_GROUP:
                            groups[group_letter] = teams[: self.TEAMS_PER_GROUP]

        if len(groups) == len(self.GROUP_LETTERS):
            return groups

        self.logger.debug(f"Could only parse {len(groups)} groups from draw page")
        return None if len(groups) < len(self.GROUP_LETTERS) else groups

    def _extract_team_names(self, element: Any) -> list[str]:
        """Extract team names from an HTML element.

        Args:
            element: BeautifulSoup element containing team information.

        Returns:
            List of team names found within the element.
        """
        teams = []

        # Look for common patterns FIFA uses for team names
        # Pattern 1: Links with team names
        team_links = element.find_all("a", href=re.compile(r"/team/"))
        for link in team_links:
            name = link.get_text(strip=True)
            if name and len(name) > 1:
                teams.append(name)

        if teams:
            return teams

        # Pattern 2: Elements with specific class patterns
        team_elements = element.find_all(
            class_=re.compile(r"team|country|nation", re.IGNORECASE)
        )
        for elem in team_elements:
            name = elem.get_text(strip=True)
            if name and len(name) > 1 and not re.match(r"^Group\s", name):
                teams.append(name)

        return teams

    def _extract_groups_from_json_ld(
        self, data: Any
    ) -> dict[str, list[str]] | None:
        """Extract group data from JSON-LD structured data.

        Args:
            data: Parsed JSON-LD data.

        Returns:
            Dictionary of groups or None if not found.
        """
        # Handle various JSON-LD structures FIFA might use
        if isinstance(data, list):
            for item in data:
                result = self._extract_groups_from_json_ld(item)
                if result:
                    return result

        if isinstance(data, dict):
            # Look for groups in common keys
            for key in ["groups", "groupStage", "draw", "competition"]:
                if key in data:
                    nested = data[key]
                    if isinstance(nested, dict) and all(
                        k in self.GROUP_LETTERS for k in nested.keys()
                    ):
                        return nested
                    result = self._extract_groups_from_json_ld(nested)
                    if result:
                        return result

        return None

    def _load_from_team_mapping(self) -> dict[str, list[str]] | None:
        """Load groups data from team_mapping.json.

        Returns:
            Dictionary of groups or None if file doesn't exist or has no groups.
        """
        if not self._team_mapping_path.exists():
            self.logger.warning(
                f"team_mapping.json not found at {self._team_mapping_path}"
            )
            return None

        try:
            with open(self._team_mapping_path, "r", encoding="utf-8") as f:
                data = json.load(f)

            groups = data.get("groups")
            if not groups:
                self.logger.warning("No 'groups' field in team_mapping.json")
                return None

            return groups

        except (json.JSONDecodeError, IOError) as e:
            self.logger.error(f"Error reading team_mapping.json: {e}")
            return None

    def _validate_groups(self, groups: dict[str, list[str]]) -> bool:
        """Validate that groups data is complete and correctly structured.

        Args:
            groups: Dictionary mapping group letters to team lists.

        Returns:
            True if valid, False otherwise.
        """
        # Check we have all 12 groups
        if set(groups.keys()) != set(self.GROUP_LETTERS):
            missing = set(self.GROUP_LETTERS) - set(groups.keys())
            extra = set(groups.keys()) - set(self.GROUP_LETTERS)
            self.logger.error(f"Invalid groups: missing={missing}, extra={extra}")
            return False

        # Check each group has 4 teams
        for letter, teams in groups.items():
            if len(teams) != self.TEAMS_PER_GROUP:
                self.logger.error(
                    f"Group {letter} has {len(teams)} teams, expected {self.TEAMS_PER_GROUP}"
                )
                return False

        # Check for duplicate teams across groups
        all_teams = []
        for teams in groups.values():
            all_teams.extend(teams)

        if len(all_teams) != len(set(all_teams)):
            duplicates = [t for t in all_teams if all_teams.count(t) > 1]
            self.logger.error(f"Duplicate teams found: {set(duplicates)}")
            return False

        # Should have 48 unique teams
        if len(set(all_teams)) != 48:
            self.logger.error(
                f"Expected 48 teams, found {len(set(all_teams))}"
            )
            return False

        return True

    def _format_output(
        self,
        groups: dict[str, list[str]],
        source: str,
    ) -> dict[str, Any]:
        """Format groups data with metadata for output.

        Args:
            groups: Dictionary mapping group letters to team lists.
            source: Source of the data (e.g., "fifa.com", "team_mapping.json").

        Returns:
            Formatted output dictionary with groups and metadata.
        """
        # Ensure groups are ordered A-L
        ordered_groups = {
            letter: groups[letter] for letter in self.GROUP_LETTERS
        }

        return {
            "groups": ordered_groups,
            "source": source,
            "scraped_at": datetime.now(timezone.utc).isoformat(),
            "meta": {
                "total_groups": len(ordered_groups),
                "teams_per_group": self.TEAMS_PER_GROUP,
                "total_teams": sum(len(t) for t in ordered_groups.values()),
                "tbd_spots": sum(
                    1
                    for teams in ordered_groups.values()
                    for team in teams
                    if "TBD" in team or "Playoff" in team
                ),
            },
        }

    def verify_against_mapping(self) -> dict[str, Any]:
        """Verify scraped groups against team_mapping.json.

        Returns:
            Dictionary with verification results including any differences.
        """
        scraped = self._try_scrape_fifa()
        mapping = self._load_from_team_mapping()

        result = {
            "scraped_available": scraped is not None,
            "mapping_available": mapping is not None,
            "match": False,
            "differences": [],
        }

        if not scraped or not mapping:
            return result

        # Compare groups
        result["match"] = True
        for letter in self.GROUP_LETTERS:
            scraped_teams = set(scraped.get(letter, []))
            mapping_teams = set(mapping.get(letter, []))

            if scraped_teams != mapping_teams:
                result["match"] = False
                result["differences"].append(
                    {
                        "group": letter,
                        "scraped": list(scraped_teams),
                        "mapping": list(mapping_teams),
                        "only_in_scraped": list(scraped_teams - mapping_teams),
                        "only_in_mapping": list(mapping_teams - scraped_teams),
                    }
                )

        return result
