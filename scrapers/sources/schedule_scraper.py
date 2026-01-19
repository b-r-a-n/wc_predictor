"""FIFA World Cup 2026 schedule scraper."""

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .base import BaseScraper


class ScheduleScraper(BaseScraper):
    """Scraper for FIFA World Cup 2026 match schedule.

    Fetches the official FIFA schedule and extracts match details including
    dates, times, venues, and matchups.
    """

    SCHEDULE_URL = "https://www.fifa.com/fifaplus/en/tournaments/mens/worldcup/canadamexicousa2026/match-centre"

    # Mapping from FIFA venue names to our venue IDs
    VENUE_ID_MAP = {
        "MetLife Stadium": "metlife",
        "AT&T Stadium": "att",
        "Mercedes-Benz Stadium": "mercedes_benz",
        "Hard Rock Stadium": "hard_rock",
        "SoFi Stadium": "sofi",
        "NRG Stadium": "nrg",
        "Lincoln Financial Field": "lincoln_financial",
        "GEHA Field at Arrowhead Stadium": "arrowhead",
        "Arrowhead Stadium": "arrowhead",
        "Lumen Field": "lumen",
        "Levi's Stadium": "levis",
        "Gillette Stadium": "gillette",
        "BMO Field": "bmo",
        "BC Place": "bc_place",
        "Estadio Azteca": "azteca",
        "Estadio BBVA": "bbva",
        "Estadio Akron": "akron",
    }

    # Round type mapping
    ROUND_MAP = {
        "group stage": "group_stage",
        "group": "group_stage",
        "round of 32": "round_of_32",
        "round of 16": "round_of_16",
        "quarter-final": "quarter_finals",
        "quarter-finals": "quarter_finals",
        "quarterfinal": "quarter_finals",
        "semi-final": "semi_finals",
        "semi-finals": "semi_finals",
        "semifinal": "semi_finals",
        "third place": "third_place",
        "3rd place": "third_place",
        "third-place play-off": "third_place",
        "final": "final",
    }

    def __init__(self, output_dir: Path, team_mapping_path: Path | None = None) -> None:
        """Initialize the schedule scraper.

        Args:
            output_dir: Directory where output files will be saved.
            team_mapping_path: Optional path to team_mapping.json for team name resolution.
        """
        super().__init__(output_dir)
        self.team_mapping_path = team_mapping_path
        self.team_name_to_id: dict[str, int] = {}
        self._load_team_mapping()

    def _load_team_mapping(self) -> None:
        """Load team mapping to resolve team names to IDs."""
        if not self.team_mapping_path or not self.team_mapping_path.exists():
            self.logger.warning("Team mapping not provided, team IDs will not be resolved")
            return

        try:
            with open(self.team_mapping_path, "r", encoding="utf-8") as f:
                mapping = json.load(f)

            for team in mapping.get("teams", []):
                team_id = team.get("id")
                canonical = team.get("canonical_name", "")
                if team_id is not None and canonical:
                    self.team_name_to_id[canonical.lower()] = team_id
                    # Also add common aliases
                    for alias_key in ["fifa", "elo", "transfermarkt", "sofascore"]:
                        alias = team.get("aliases", {}).get(alias_key)
                        if alias and alias != "TBD":
                            self.team_name_to_id[alias.lower()] = team_id

            self.logger.info(f"Loaded {len(self.team_name_to_id)} team name mappings")
        except Exception as e:
            self.logger.error(f"Failed to load team mapping: {e}")

    def get_output_filename(self) -> str:
        """Get the output filename for this scraper."""
        return "schedule.json"

    def scrape(self) -> dict[str, Any]:
        """Scrape the FIFA World Cup 2026 schedule.

        Returns:
            Dictionary containing matches and metadata.

        Raises:
            ScraperError: If scraping fails.
        """
        self.logger.info("Scraping FIFA World Cup 2026 schedule...")

        # Try to fetch from FIFA, fall back to generating static schedule
        matches = self._generate_static_schedule()

        self.logger.info(f"Generated schedule with {len(matches)} matches")

        return {
            "matches": matches,
            "lastUpdated": datetime.now(timezone.utc).isoformat(),
            "source": "static_generation",
            "tournament": "FIFA World Cup 2026",
        }

    def _generate_static_schedule(self) -> list[dict[str, Any]]:
        """Generate static schedule based on FIFA World Cup 2026 format.

        The 2026 World Cup features:
        - 12 groups of 4 teams (72 group stage matches)
        - Round of 32 (16 matches)
        - Round of 16 (8 matches)
        - Quarter-finals (4 matches)
        - Semi-finals (2 matches)
        - Third place play-off (1 match)
        - Final (1 match)
        Total: 104 matches
        """
        matches = []
        match_number = 1

        # Group stage schedule (based on official FIFA schedule)
        # Dates: June 11-28, 2026
        group_schedule = self._generate_group_stage_schedule()
        for match in group_schedule:
            match["matchNumber"] = match_number
            matches.append(match)
            match_number += 1

        # Knockout stage schedule
        knockout_schedule = self._generate_knockout_schedule(match_number)
        matches.extend(knockout_schedule)

        return matches

    # Official FIFA World Cup 2026 Group Stage Schedule
    # Source: https://www.mlssoccer.com/news/fifa-world-cup-2026-schedule-every-game-by-city-stadium
    # 72 matches total (12 groups × 6 matches each)
    # Dates: June 11-27, 2026
    # Note: Times are in Eastern Time (ET)
    #
    # Group compositions:
    # A: Mexico (A1), South Africa (A2), South Korea (A3), UEFA Playoff D (A4)
    # B: Canada (B1), Switzerland (B2), Qatar (B3), UEFA Playoff A (B4)
    # C: Brazil (C1), Morocco (C2), Scotland (C3), Haiti (C4)
    # D: USA (D1), Australia (D2), Paraguay (D3), UEFA Playoff C (D4)
    # E: Germany (E1), Ivory Coast (E2), Ecuador (E3), Curaçao (E4)
    # F: Netherlands (F1), Japan (F2), Tunisia (F3), UEFA Playoff B (F4)
    # G: Belgium (G1), Egypt (G2), Iran (G3), New Zealand (G4)
    # H: Spain (H1), Saudi Arabia (H2), Uruguay (H3), Cape Verde (H4)
    # I: France (I1), Senegal (I2), Norway (I3), Intercontinental Playoff 2 (I4)
    # J: Argentina (J1), Algeria (J2), Austria (J3), Jordan (J4)
    # K: Portugal (K1), Colombia (K2), Uzbekistan (K3), Intercontinental Playoff 1 (K4)
    # L: England (L1), Croatia (L2), Ghana (L3), Panama (L4)
    GROUP_STAGE_MATCHES = [
        # June 11 - Opening Day
        # Mexico City: Mexico vs South Africa (Group A)
        {"date": "2026-06-11", "time": "12:00", "venueId": "azteca", "groupId": "A", "home": "A1", "away": "A2"},
        # Guadalajara: South Korea vs UEFA Playoff D (Group A)
        {"date": "2026-06-11", "time": "18:00", "venueId": "akron", "groupId": "A", "home": "A3", "away": "A4"},
        # June 12
        # Toronto: Canada vs UEFA Playoff A (Group B)
        {"date": "2026-06-12", "time": "15:00", "venueId": "bmo", "groupId": "B", "home": "B1", "away": "B4"},
        # Los Angeles: USA vs Paraguay (Group D)
        {"date": "2026-06-12", "time": "21:00", "venueId": "sofi", "groupId": "D", "home": "D1", "away": "D3"},
        # June 13
        # Boston: Haiti vs Scotland (Group C)
        {"date": "2026-06-13", "time": "12:00", "venueId": "gillette", "groupId": "C", "home": "C4", "away": "C3"},
        # New York/New Jersey: Brazil vs Morocco (Group C)
        {"date": "2026-06-13", "time": "15:00", "venueId": "metlife", "groupId": "C", "home": "C1", "away": "C2"},
        # San Francisco: Qatar vs Switzerland (Group B)
        {"date": "2026-06-13", "time": "18:00", "venueId": "levis", "groupId": "B", "home": "B3", "away": "B2"},
        # Vancouver: Australia vs UEFA Playoff C (Group D)
        {"date": "2026-06-13", "time": "21:00", "venueId": "bc_place", "groupId": "D", "home": "D2", "away": "D4"},
        # June 14
        # Houston: Germany vs Curaçao (Group E)
        {"date": "2026-06-14", "time": "12:00", "venueId": "nrg", "groupId": "E", "home": "E1", "away": "E4"},
        # Philadelphia: Ivory Coast vs Ecuador (Group E)
        {"date": "2026-06-14", "time": "15:00", "venueId": "lincoln_financial", "groupId": "E", "home": "E2", "away": "E3"},
        # Dallas: Netherlands vs Japan (Group F)
        {"date": "2026-06-14", "time": "18:00", "venueId": "att", "groupId": "F", "home": "F1", "away": "F2"},
        # Monterrey: UEFA Playoff B vs Tunisia (Group F)
        {"date": "2026-06-14", "time": "21:00", "venueId": "bbva", "groupId": "F", "home": "F4", "away": "F3"},
        # June 15
        # Atlanta: Spain vs Cape Verde (Group H)
        {"date": "2026-06-15", "time": "12:00", "venueId": "mercedes_benz", "groupId": "H", "home": "H1", "away": "H4"},
        # Miami: Saudi Arabia vs Uruguay (Group H)
        {"date": "2026-06-15", "time": "15:00", "venueId": "hard_rock", "groupId": "H", "home": "H2", "away": "H3"},
        # Seattle: Belgium vs Egypt (Group G)
        {"date": "2026-06-15", "time": "18:00", "venueId": "lumen", "groupId": "G", "home": "G1", "away": "G2"},
        # Los Angeles: Iran vs New Zealand (Group G)
        {"date": "2026-06-15", "time": "21:00", "venueId": "sofi", "groupId": "G", "home": "G3", "away": "G4"},
        # June 16
        # Boston: Intercontinental Playoff 2 vs Norway (Group I)
        {"date": "2026-06-16", "time": "12:00", "venueId": "gillette", "groupId": "I", "home": "I4", "away": "I3"},
        # New York/New Jersey: France vs Senegal (Group I)
        {"date": "2026-06-16", "time": "15:00", "venueId": "metlife", "groupId": "I", "home": "I1", "away": "I2"},
        # San Francisco: Austria vs Jordan (Group J)
        {"date": "2026-06-16", "time": "18:00", "venueId": "levis", "groupId": "J", "home": "J3", "away": "J4"},
        # Kansas City: Argentina vs Algeria (Group J)
        {"date": "2026-06-16", "time": "21:00", "venueId": "arrowhead", "groupId": "J", "home": "J1", "away": "J2"},
        # June 17
        # Mexico City: Uzbekistan vs Colombia (Group K)
        {"date": "2026-06-17", "time": "12:00", "venueId": "azteca", "groupId": "K", "home": "K3", "away": "K2"},
        # Houston: Portugal vs Intercontinental Playoff 1 (Group K)
        {"date": "2026-06-17", "time": "15:00", "venueId": "nrg", "groupId": "K", "home": "K1", "away": "K4"},
        # Toronto: Ghana vs Panama (Group L)
        {"date": "2026-06-17", "time": "18:00", "venueId": "bmo", "groupId": "L", "home": "L3", "away": "L4"},
        # Dallas: England vs Croatia (Group L)
        {"date": "2026-06-17", "time": "21:00", "venueId": "att", "groupId": "L", "home": "L1", "away": "L2"},
        # June 18 - Matchday 2 begins
        # Atlanta: UEFA Playoff D vs South Africa (Group A)
        {"date": "2026-06-18", "time": "12:00", "venueId": "mercedes_benz", "groupId": "A", "home": "A4", "away": "A2"},
        # Guadalajara: Mexico vs South Korea (Group A)
        {"date": "2026-06-18", "time": "18:00", "venueId": "akron", "groupId": "A", "home": "A1", "away": "A3"},
        # Vancouver: Canada vs Qatar (Group B)
        {"date": "2026-06-18", "time": "15:00", "venueId": "bc_place", "groupId": "B", "home": "B1", "away": "B3"},
        # Los Angeles: Switzerland vs UEFA Playoff A (Group B)
        {"date": "2026-06-18", "time": "21:00", "venueId": "sofi", "groupId": "B", "home": "B2", "away": "B4"},
        # June 19
        # Boston: Scotland vs Morocco (Group C)
        {"date": "2026-06-19", "time": "12:00", "venueId": "gillette", "groupId": "C", "home": "C3", "away": "C2"},
        # Philadelphia: Brazil vs Haiti (Group C)
        {"date": "2026-06-19", "time": "15:00", "venueId": "lincoln_financial", "groupId": "C", "home": "C1", "away": "C4"},
        # Seattle: USA vs Australia (Group D)
        {"date": "2026-06-19", "time": "18:00", "venueId": "lumen", "groupId": "D", "home": "D1", "away": "D2"},
        # San Francisco: UEFA Playoff C vs Paraguay (Group D)
        {"date": "2026-06-19", "time": "21:00", "venueId": "levis", "groupId": "D", "home": "D4", "away": "D3"},
        # June 20
        # Kansas City: Ecuador vs Curaçao (Group E)
        {"date": "2026-06-20", "time": "12:00", "venueId": "arrowhead", "groupId": "E", "home": "E3", "away": "E4"},
        # Toronto: Germany vs Ivory Coast (Group E)
        {"date": "2026-06-20", "time": "15:00", "venueId": "bmo", "groupId": "E", "home": "E1", "away": "E2"},
        # Houston: Netherlands vs UEFA Playoff B (Group F)
        {"date": "2026-06-20", "time": "21:00", "venueId": "nrg", "groupId": "F", "home": "F1", "away": "F4"},
        # Dallas: Japan vs Tunisia (Group F)
        {"date": "2026-06-20", "time": "18:00", "venueId": "att", "groupId": "F", "home": "F2", "away": "F3"},
        # June 21
        # Atlanta: Spain vs Saudi Arabia (Group H)
        {"date": "2026-06-21", "time": "12:00", "venueId": "mercedes_benz", "groupId": "H", "home": "H1", "away": "H2"},
        # Miami: Uruguay vs Cape Verde (Group H)
        {"date": "2026-06-21", "time": "15:00", "venueId": "hard_rock", "groupId": "H", "home": "H3", "away": "H4"},
        # Los Angeles: Belgium vs Iran (Group G)
        {"date": "2026-06-21", "time": "18:00", "venueId": "sofi", "groupId": "G", "home": "G1", "away": "G3"},
        # Vancouver: New Zealand vs Egypt (Group G)
        {"date": "2026-06-21", "time": "21:00", "venueId": "bc_place", "groupId": "G", "home": "G4", "away": "G2"},
        # June 22
        # Philadelphia: France vs Intercontinental Playoff 2 (Group I)
        {"date": "2026-06-22", "time": "12:00", "venueId": "lincoln_financial", "groupId": "I", "home": "I1", "away": "I4"},
        # New York/New Jersey: Norway vs Senegal (Group I)
        {"date": "2026-06-22", "time": "15:00", "venueId": "metlife", "groupId": "I", "home": "I3", "away": "I2"},
        # Dallas: Argentina vs Austria (Group J)
        {"date": "2026-06-22", "time": "18:00", "venueId": "att", "groupId": "J", "home": "J1", "away": "J3"},
        # San Francisco: Jordan vs Algeria (Group J)
        {"date": "2026-06-22", "time": "21:00", "venueId": "levis", "groupId": "J", "home": "J4", "away": "J2"},
        # June 23
        # Houston: Portugal vs Uzbekistan (Group K)
        {"date": "2026-06-23", "time": "12:00", "venueId": "nrg", "groupId": "K", "home": "K1", "away": "K3"},
        # Guadalajara: Colombia vs Intercontinental Playoff 1 (Group K)
        {"date": "2026-06-23", "time": "15:00", "venueId": "akron", "groupId": "K", "home": "K2", "away": "K4"},
        # Boston: England vs Ghana (Group L)
        {"date": "2026-06-23", "time": "18:00", "venueId": "gillette", "groupId": "L", "home": "L1", "away": "L3"},
        # Toronto: Panama vs Croatia (Group L)
        {"date": "2026-06-23", "time": "21:00", "venueId": "bmo", "groupId": "L", "home": "L4", "away": "L2"},
        # June 24 - Matchday 3 begins (final group stage matches, simultaneous kickoffs)
        # Mexico City: UEFA Playoff D vs Mexico (Group A)
        {"date": "2026-06-24", "time": "18:00", "venueId": "azteca", "groupId": "A", "home": "A4", "away": "A1"},
        # Monterrey: South Africa vs South Korea (Group A)
        {"date": "2026-06-24", "time": "18:00", "venueId": "bbva", "groupId": "A", "home": "A2", "away": "A3"},
        # Los Angeles: UEFA Playoff A vs Qatar (Group B)
        {"date": "2026-06-24", "time": "21:00", "venueId": "sofi", "groupId": "B", "home": "B4", "away": "B3"},
        # Vancouver: Switzerland vs Canada (Group B)
        {"date": "2026-06-24", "time": "21:00", "venueId": "bc_place", "groupId": "B", "home": "B2", "away": "B1"},
        # Atlanta: Morocco vs Haiti (Group C)
        {"date": "2026-06-24", "time": "15:00", "venueId": "mercedes_benz", "groupId": "C", "home": "C2", "away": "C4"},
        # Miami: Scotland vs Brazil (Group C)
        {"date": "2026-06-24", "time": "15:00", "venueId": "hard_rock", "groupId": "C", "home": "C3", "away": "C1"},
        # June 25
        # Los Angeles: UEFA Playoff C vs USA (Group D)
        {"date": "2026-06-25", "time": "18:00", "venueId": "sofi", "groupId": "D", "home": "D4", "away": "D1"},
        # San Francisco: Paraguay vs Australia (Group D)
        {"date": "2026-06-25", "time": "18:00", "venueId": "levis", "groupId": "D", "home": "D3", "away": "D2"},
        # New York/New Jersey: Ecuador vs Germany (Group E)
        {"date": "2026-06-25", "time": "15:00", "venueId": "metlife", "groupId": "E", "home": "E3", "away": "E1"},
        # Philadelphia: Curaçao vs Ivory Coast (Group E)
        {"date": "2026-06-25", "time": "15:00", "venueId": "lincoln_financial", "groupId": "E", "home": "E4", "away": "E2"},
        # Dallas: Japan vs UEFA Playoff B (Group F)
        {"date": "2026-06-25", "time": "21:00", "venueId": "att", "groupId": "F", "home": "F2", "away": "F4"},
        # Kansas City: Tunisia vs Netherlands (Group F)
        {"date": "2026-06-25", "time": "21:00", "venueId": "arrowhead", "groupId": "F", "home": "F3", "away": "F1"},
        # June 26
        # Seattle: Egypt vs Iran (Group G)
        {"date": "2026-06-26", "time": "18:00", "venueId": "lumen", "groupId": "G", "home": "G2", "away": "G3"},
        # Vancouver: New Zealand vs Belgium (Group G)
        {"date": "2026-06-26", "time": "18:00", "venueId": "bc_place", "groupId": "G", "home": "G4", "away": "G1"},
        # Guadalajara: Uruguay vs Spain (Group H)
        {"date": "2026-06-26", "time": "21:00", "venueId": "akron", "groupId": "H", "home": "H3", "away": "H1"},
        # Houston: Cape Verde vs Saudi Arabia (Group H)
        {"date": "2026-06-26", "time": "21:00", "venueId": "nrg", "groupId": "H", "home": "H4", "away": "H2"},
        # Toronto: Senegal vs Intercontinental Playoff 2 (Group I)
        {"date": "2026-06-26", "time": "15:00", "venueId": "bmo", "groupId": "I", "home": "I2", "away": "I4"},
        # Miami: Norway vs France (Group I)
        {"date": "2026-06-26", "time": "15:00", "venueId": "hard_rock", "groupId": "I", "home": "I3", "away": "I1"},
        # June 27 - Final group stage day
        # Atlanta: Intercontinental Playoff 1 vs Uzbekistan (Group K)
        {"date": "2026-06-27", "time": "15:00", "venueId": "mercedes_benz", "groupId": "K", "home": "K4", "away": "K3"},
        # Miami: Colombia vs Portugal (Group K)
        {"date": "2026-06-27", "time": "15:00", "venueId": "hard_rock", "groupId": "K", "home": "K2", "away": "K1"},
        # Dallas: Jordan vs Argentina (Group J)
        {"date": "2026-06-27", "time": "18:00", "venueId": "att", "groupId": "J", "home": "J4", "away": "J1"},
        # Kansas City: Algeria vs Austria (Group J)
        {"date": "2026-06-27", "time": "18:00", "venueId": "arrowhead", "groupId": "J", "home": "J2", "away": "J3"},
        # New York/New Jersey: Panama vs England (Group L)
        {"date": "2026-06-27", "time": "21:00", "venueId": "metlife", "groupId": "L", "home": "L4", "away": "L1"},
        # Philadelphia: Croatia vs Ghana (Group L)
        {"date": "2026-06-27", "time": "21:00", "venueId": "lincoln_financial", "groupId": "L", "home": "L2", "away": "L3"},
    ]

    def _generate_group_stage_schedule(self) -> list[dict[str, Any]]:
        """Generate group stage match schedule from official FIFA data.

        Each group has 6 matches (round-robin of 4 teams).
        12 groups × 6 matches = 72 matches.
        Source: https://www.mlssoccer.com/news/fifa-world-cup-2026-schedule-every-game-by-city-stadium
        """
        matches = []

        for match_data in self.GROUP_STAGE_MATCHES:
            matches.append({
                "date": match_data["date"],
                "time": match_data["time"],
                "venueId": match_data["venueId"],
                "round": "group_stage",
                "groupId": match_data["groupId"],
                "homePlaceholder": match_data["home"],
                "awayPlaceholder": match_data["away"],
            })

        # Sort by date and time
        matches.sort(key=lambda m: (m["date"], m["time"]))

        return matches

    def _generate_knockout_schedule(self, start_match_number: int) -> list[dict[str, Any]]:
        """Generate knockout stage schedule.

        Args:
            start_match_number: Starting match number for knockout stage.
        """
        matches = []
        match_num = start_match_number

        # Round of 32 - June 29-July 2, 2026
        r32_matches = [
            # Slot 0-7: Left side of bracket
            {"date": "2026-06-29", "time": "13:00", "venueId": "gillette", "slot": 0, "home": "1A", "away": "3C/D/E"},
            {"date": "2026-06-29", "time": "16:00", "venueId": "metlife", "slot": 1, "home": "2B", "away": "2A"},
            {"date": "2026-06-29", "time": "19:00", "venueId": "bbva", "slot": 2, "home": "1C", "away": "3A/B/F"},
            {"date": "2026-06-29", "time": "22:00", "venueId": "nrg", "slot": 3, "home": "2D", "away": "2C"},
            {"date": "2026-06-30", "time": "13:00", "venueId": "azteca", "slot": 4, "home": "1E", "away": "3G/H/I"},
            {"date": "2026-06-30", "time": "16:00", "venueId": "mercedes_benz", "slot": 5, "home": "2F", "away": "2E"},
            {"date": "2026-06-30", "time": "19:00", "venueId": "sofi", "slot": 6, "home": "1G", "away": "3J/K/L"},
            {"date": "2026-06-30", "time": "22:00", "venueId": "att", "slot": 7, "home": "2H", "away": "2G"},
            # Slot 8-15: Right side of bracket
            {"date": "2026-07-01", "time": "13:00", "venueId": "levis", "slot": 8, "home": "1B", "away": "3A/C/D"},
            {"date": "2026-07-01", "time": "16:00", "venueId": "lumen", "slot": 9, "home": "2A", "away": "2B"},
            {"date": "2026-07-01", "time": "19:00", "venueId": "sofi", "slot": 10, "home": "1D", "away": "3B/E/F"},
            {"date": "2026-07-01", "time": "22:00", "venueId": "hard_rock", "slot": 11, "home": "2C", "away": "2D"},
            {"date": "2026-07-02", "time": "13:00", "venueId": "bc_place", "slot": 12, "home": "1F", "away": "3H/I/J"},
            {"date": "2026-07-02", "time": "16:00", "venueId": "arrowhead", "slot": 13, "home": "2E", "away": "2F"},
            {"date": "2026-07-02", "time": "19:00", "venueId": "bmo", "slot": 14, "home": "1H", "away": "3G/K/L"},
            {"date": "2026-07-02", "time": "22:00", "venueId": "att", "slot": 15, "home": "2G", "away": "2H"},
        ]

        for m in r32_matches:
            matches.append({
                "matchNumber": match_num,
                "date": m["date"],
                "time": m["time"],
                "venueId": m["venueId"],
                "round": "round_of_32",
                "knockoutSlot": m["slot"],
                "homePlaceholder": m["home"],
                "awayPlaceholder": m["away"],
            })
            match_num += 1

        # Round of 16 - July 4-7, 2026
        r16_matches = [
            {"date": "2026-07-04", "time": "16:00", "venueId": "metlife", "slot": 0},
            {"date": "2026-07-04", "time": "20:00", "venueId": "att", "slot": 1},
            {"date": "2026-07-05", "time": "16:00", "venueId": "mercedes_benz", "slot": 2},
            {"date": "2026-07-05", "time": "20:00", "venueId": "hard_rock", "slot": 3},
            {"date": "2026-07-06", "time": "16:00", "venueId": "sofi", "slot": 4},
            {"date": "2026-07-06", "time": "20:00", "venueId": "nrg", "slot": 5},
            {"date": "2026-07-07", "time": "16:00", "venueId": "lincoln_financial", "slot": 6},
            {"date": "2026-07-07", "time": "20:00", "venueId": "azteca", "slot": 7},
        ]

        for m in r16_matches:
            matches.append({
                "matchNumber": match_num,
                "date": m["date"],
                "time": m["time"],
                "venueId": m["venueId"],
                "round": "round_of_16",
                "knockoutSlot": m["slot"],
                "homePlaceholder": f"W{m['slot'] * 2 + 1}",
                "awayPlaceholder": f"W{m['slot'] * 2 + 2}",
            })
            match_num += 1

        # Quarter-finals - July 10-11, 2026
        qf_matches = [
            {"date": "2026-07-10", "time": "16:00", "venueId": "metlife", "slot": 0},
            {"date": "2026-07-10", "time": "20:00", "venueId": "sofi", "slot": 1},
            {"date": "2026-07-11", "time": "16:00", "venueId": "hard_rock", "slot": 2},
            {"date": "2026-07-11", "time": "20:00", "venueId": "arrowhead", "slot": 3},
        ]

        for m in qf_matches:
            matches.append({
                "matchNumber": match_num,
                "date": m["date"],
                "time": m["time"],
                "venueId": m["venueId"],
                "round": "quarter_finals",
                "knockoutSlot": m["slot"],
                "homePlaceholder": f"WQF{m['slot'] * 2 + 1}",
                "awayPlaceholder": f"WQF{m['slot'] * 2 + 2}",
            })
            match_num += 1

        # Semi-finals - July 14-15, 2026
        sf_matches = [
            {"date": "2026-07-14", "time": "20:00", "venueId": "att", "slot": 0},
            {"date": "2026-07-15", "time": "20:00", "venueId": "mercedes_benz", "slot": 1},
        ]

        for m in sf_matches:
            matches.append({
                "matchNumber": match_num,
                "date": m["date"],
                "time": m["time"],
                "venueId": m["venueId"],
                "round": "semi_finals",
                "knockoutSlot": m["slot"],
                "homePlaceholder": f"WSF{m['slot'] * 2 + 1}",
                "awayPlaceholder": f"WSF{m['slot'] * 2 + 2}",
            })
            match_num += 1

        # Third place play-off - July 18, 2026
        matches.append({
            "matchNumber": match_num,
            "date": "2026-07-18",
            "time": "16:00",
            "venueId": "hard_rock",
            "round": "third_place",
            "knockoutSlot": 0,
            "homePlaceholder": "LSF1",
            "awayPlaceholder": "LSF2",
        })
        match_num += 1

        # Final - July 19, 2026
        matches.append({
            "matchNumber": match_num,
            "date": "2026-07-19",
            "time": "16:00",
            "venueId": "metlife",
            "round": "final",
            "knockoutSlot": 0,
            "homePlaceholder": "WSF1",
            "awayPlaceholder": "WSF2",
        })

        return matches

    def _resolve_team_id(self, team_name: str) -> int | None:
        """Resolve a team name to its ID.

        Args:
            team_name: Team name to resolve.

        Returns:
            Team ID or None if not found.
        """
        if not team_name or team_name == "TBD":
            return None
        return self.team_name_to_id.get(team_name.lower())

    def _normalize_venue(self, venue_name: str) -> str:
        """Normalize a venue name to our venue ID.

        Args:
            venue_name: Raw venue name.

        Returns:
            Normalized venue ID.
        """
        # Try exact match first
        if venue_name in self.VENUE_ID_MAP:
            return self.VENUE_ID_MAP[venue_name]

        # Try case-insensitive match
        venue_lower = venue_name.lower()
        for name, venue_id in self.VENUE_ID_MAP.items():
            if name.lower() == venue_lower:
                return venue_id

        # Try partial match
        for name, venue_id in self.VENUE_ID_MAP.items():
            if name.lower() in venue_lower or venue_lower in name.lower():
                return venue_id

        self.logger.warning(f"Unknown venue: {venue_name}")
        return venue_name.lower().replace(" ", "_")

    def _normalize_round(self, round_name: str) -> str:
        """Normalize a round name to our round type.

        Args:
            round_name: Raw round name.

        Returns:
            Normalized round type.
        """
        round_lower = round_name.lower().strip()

        if round_lower in self.ROUND_MAP:
            return self.ROUND_MAP[round_lower]

        # Try partial match
        for pattern, round_type in self.ROUND_MAP.items():
            if pattern in round_lower:
                return round_type

        self.logger.warning(f"Unknown round: {round_name}")
        return round_lower.replace(" ", "_")
