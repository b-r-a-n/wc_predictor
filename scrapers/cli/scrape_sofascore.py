"""CLI command to scrape Sofascore form data."""

import json
import sys
from pathlib import Path

import click
from rich.console import Console
from rich.table import Table

from ..config.settings import OUTPUT_DIR
from ..sources.base import ScraperError
from ..sources.sofascore_scraper import SofascoreScraper
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


def get_sofascore_team_ids(team_mapping: dict) -> dict[str, int]:
    """Get mapping of canonical team names to Sofascore IDs.

    Args:
        team_mapping: The loaded team mapping dictionary.

    Returns:
        Dict mapping canonical names to Sofascore IDs.
    """
    team_ids = {}

    for team in team_mapping.get("teams", []):
        canonical_name = team.get("canonical_name", "")
        aliases = team.get("aliases", {})
        sofascore_id = aliases.get("sofascore_id")

        # Skip TBD teams or teams without Sofascore ID
        if sofascore_id and canonical_name and "TBD" not in canonical_name:
            team_ids[canonical_name] = sofascore_id

    return team_ids


@click.command()
@click.option(
    "--output-dir",
    "-o",
    type=click.Path(path_type=Path),
    default=OUTPUT_DIR,
    help="Output directory for scraped data.",
)
@click.option(
    "--team",
    "-t",
    type=str,
    default=None,
    help="Scrape only a specific team by canonical name.",
)
@click.option(
    "--verbose",
    "-v",
    is_flag=True,
    help="Enable verbose output.",
)
def main(output_dir: Path, team: str | None, verbose: bool) -> None:
    """Scrape team form data from Sofascore.

    Fetches recent match results for national football teams and calculates
    a form score based on points per game (W=3, D=1, L=0).
    """
    setup_logging()

    console.print("[bold blue]Sofascore Form Scraper[/bold blue]")
    console.print("[dim]Source: sofascore.com[/dim]")
    console.print()

    # Load team mapping
    console.print("[dim]Loading team mapping...[/dim]")
    team_mapping = load_team_mapping()
    team_ids = get_sofascore_team_ids(team_mapping)

    if not team_ids:
        raise click.ClickException(
            "No Sofascore IDs found in team mapping. "
            "Make sure teams have 'sofascore_id' in their aliases."
        )

    console.print(f"[green]Found {len(team_ids)} teams with Sofascore IDs[/green]")

    # Filter to single team if specified
    if team:
        if team in team_ids:
            team_ids = {team: team_ids[team]}
        else:
            # Try case-insensitive match
            for canonical, sofascore_id in list(team_ids.items()):
                if canonical.lower() == team.lower():
                    team_ids = {canonical: sofascore_id}
                    break
            else:
                raise click.ClickException(f"Team not found in mapping: {team}")
        console.print(f"[yellow]Scraping single team: {list(team_ids.keys())[0]}[/yellow]")

    # Run scraper
    console.print()
    console.print(f"[dim]Fetching form data for {len(team_ids)} teams...[/dim]")
    console.print("[dim](Rate limited: 2 seconds between requests)[/dim]")
    console.print()

    try:
        scraper = SofascoreScraper(output_dir=output_dir)
        data = scraper.scrape(team_ids=team_ids)
    except ScraperError as e:
        console.print(f"[bold red]Scraper error:[/bold red] {e}")
        sys.exit(1)
    except Exception as e:
        console.print(f"[bold red]Unexpected error:[/bold red] {e}")
        sys.exit(1)

    # Save output
    output_path = scraper.save(data)
    console.print()
    console.print(f"[bold green]Saved to:[/bold green] {output_path}")

    # Display summary table
    teams_data = data.get("teams", {})
    matches_info = data.get("matches_info", {})

    if teams_data:
        console.print()
        table = Table(title="Team Form Scores")
        table.add_column("Team", style="cyan")
        table.add_column("Form", justify="right", style="green")
        table.add_column("Record", justify="center")
        table.add_column("Results", justify="left", style="dim")

        # Sort by form score descending
        sorted_teams = sorted(
            teams_data.items(),
            key=lambda x: x[1],
            reverse=True
        )

        display_count = 20 if not verbose else len(sorted_teams)
        for team_name, form_score in sorted_teams[:display_count]:
            info = matches_info.get(team_name, {})
            record = f"{info.get('wins', 0)}W-{info.get('draws', 0)}D-{info.get('losses', 0)}L"
            results = info.get("results", "")
            table.add_row(team_name, f"{form_score:.2f}", record, results)

        console.print(table)

        if len(sorted_teams) > display_count:
            console.print(f"[dim]... and {len(sorted_teams) - display_count} more teams[/dim]")

    # Summary
    console.print()
    console.print("[bold]Summary:[/bold]")
    console.print(f"  Teams scraped: {len(teams_data)}")
    console.print(f"  Source: {data.get('source', 'unknown')}")
    console.print(f"  Scraped at: {data.get('scraped_at', 'unknown')}")

    if teams_data:
        avg_form = sum(teams_data.values()) / len(teams_data)
        console.print(f"  Average form: {avg_form:.2f}")


if __name__ == "__main__":
    main()
