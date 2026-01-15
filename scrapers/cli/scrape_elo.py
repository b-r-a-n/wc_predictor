"""CLI command to scrape ELO ratings."""

import json
import sys
from pathlib import Path

import click
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.table import Table

from ..config.settings import OUTPUT_DIR
from ..sources.base import ScraperError
from ..sources.elo_scraper import EloScraper
from ..utils.logging_config import setup_logging


# Path to team mapping config
CONFIG_DIR = Path(__file__).parent.parent / "config"
TEAM_MAPPING_FILE = CONFIG_DIR / "team_mapping.json"


console = Console()


def load_team_mapping() -> dict:
    """Load team mapping from config file.

    Returns:
        Dictionary containing team mapping data.

    Raises:
        click.ClickException: If the mapping file cannot be loaded.
    """
    if not TEAM_MAPPING_FILE.exists():
        raise click.ClickException(
            f"Team mapping file not found: {TEAM_MAPPING_FILE}"
        )

    try:
        with open(TEAM_MAPPING_FILE, "r", encoding="utf-8") as f:
            return json.load(f)
    except json.JSONDecodeError as e:
        raise click.ClickException(f"Invalid JSON in team mapping file: {e}")


def get_elo_team_names(team_mapping: dict) -> list[str]:
    """Get list of team names for ELO scraping.

    Args:
        team_mapping: The loaded team mapping dictionary.

    Returns:
        List of team names to use when querying international-football.net.
    """
    team_names = []

    for team in team_mapping.get("teams", []):
        aliases = team.get("aliases", {})
        elo_name = aliases.get("elo")

        # Skip TBD teams
        if elo_name and elo_name != "TBD":
            team_names.append(elo_name)

    return team_names


def build_elo_to_canonical_map(team_mapping: dict) -> dict[str, str]:
    """Build a mapping from ELO names to canonical names.

    Args:
        team_mapping: The loaded team mapping dictionary.

    Returns:
        Dictionary mapping ELO site names to canonical names.
    """
    elo_to_canonical = {}

    for team in team_mapping.get("teams", []):
        canonical_name = team.get("canonical_name", "")
        aliases = team.get("aliases", {})
        elo_name = aliases.get("elo", "")

        if elo_name and elo_name != "TBD":
            elo_to_canonical[elo_name] = canonical_name

    return elo_to_canonical


@click.command()
@click.option(
    "--output-dir",
    "-o",
    type=click.Path(path_type=Path),
    default=OUTPUT_DIR,
    help="Output directory for scraped data.",
)
@click.option(
    "--verbose",
    "-v",
    is_flag=True,
    help="Enable verbose output.",
)
@click.option(
    "--team",
    "-t",
    type=str,
    default=None,
    help="Scrape only a specific team by name.",
)
def main(output_dir: Path, verbose: bool, team: str | None) -> None:
    """Scrape ELO ratings from international-football.net.

    Fetches the latest ELO ratings for national football teams and saves
    them to a JSON file. This scrapes individual country pages since the
    main eloratings.net site uses JavaScript rendering.
    """
    setup_logging()

    console.print("[bold blue]ELO Ratings Scraper[/bold blue]")
    console.print("[dim]Source: international-football.net[/dim]")
    console.print()

    # Load team mapping
    console.print("[dim]Loading team mapping...[/dim]")
    team_mapping = load_team_mapping()
    team_names = get_elo_team_names(team_mapping)
    elo_to_canonical = build_elo_to_canonical_map(team_mapping)
    console.print(f"[green]Loaded {len(team_names)} teams to scrape[/green]")

    # Filter to single team if specified
    if team:
        if team not in team_names:
            # Try to find by canonical name
            for elo_name, canonical in elo_to_canonical.items():
                if canonical.lower() == team.lower():
                    team_names = [elo_name]
                    break
            else:
                raise click.ClickException(f"Team not found in mapping: {team}")
        else:
            team_names = [team]
        console.print(f"[yellow]Scraping single team: {team_names[0]}[/yellow]")

    # Run scraper
    console.print()
    console.print(f"[dim]Fetching ELO ratings for {len(team_names)} teams...[/dim]")
    console.print("[dim](Rate limited: 1 second between requests)[/dim]")
    console.print()

    try:
        scraper = EloScraper(output_dir=output_dir)
        data = scraper.scrape(team_names=team_names)
    except ScraperError as e:
        console.print(f"[bold red]Scraper error:[/bold red] {e}")
        sys.exit(1)
    except Exception as e:
        console.print(f"[bold red]Unexpected error:[/bold red] {e}")
        sys.exit(1)

    # Map ELO names to canonical names for output
    scraped_teams = data.get("teams", {})
    canonical_teams = {}
    for elo_name, rating in scraped_teams.items():
        canonical_name = elo_to_canonical.get(elo_name, elo_name)
        canonical_teams[canonical_name] = rating

    # Update data with canonical names
    data["teams"] = canonical_teams

    # Save output
    output_path = scraper.save(data)
    console.print()
    console.print(f"[bold green]Saved to:[/bold green] {output_path}")

    # Display summary table
    if verbose and canonical_teams:
        console.print()
        table = Table(title="ELO Ratings")
        table.add_column("Team", style="cyan")
        table.add_column("ELO Rating", justify="right", style="green")

        # Sort by rating descending
        sorted_teams = sorted(
            canonical_teams.items(),
            key=lambda x: x[1],
            reverse=True
        )

        for team_name, rating in sorted_teams[:20]:
            table.add_row(team_name, str(rating))

        console.print(table)

        if len(sorted_teams) > 20:
            console.print(f"[dim]... and {len(sorted_teams) - 20} more teams[/dim]")

    # Summary
    console.print()
    console.print("[bold]Summary:[/bold]")
    console.print(f"  Teams scraped: {len(canonical_teams)}")
    console.print(f"  Source: {data.get('source', 'unknown')}")
    console.print(f"  Scraped at: {data.get('scraped_at', 'unknown')}")


if __name__ == "__main__":
    main()
