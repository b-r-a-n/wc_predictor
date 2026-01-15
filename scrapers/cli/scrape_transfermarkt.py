"""CLI command to scrape Transfermarkt market values."""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

import click
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TaskProgressColumn
from rich.table import Table

from scrapers.sources.transfermarkt_scraper import TransfermarktScraper
from scrapers.sources.base import ScraperError
from scrapers.config.settings import OUTPUT_DIR


# Path to team mapping config
CONFIG_DIR = Path(__file__).parent.parent / "config"
TEAM_MAPPING_FILE = CONFIG_DIR / "team_mapping.json"

console = Console()


def load_team_mapping() -> dict:
    """Load team mapping from config file.

    Returns:
        The team mapping dictionary.

    Raises:
        click.ClickException: If the file cannot be loaded.
    """
    if not TEAM_MAPPING_FILE.exists():
        raise click.ClickException(
            f"Team mapping file not found: {TEAM_MAPPING_FILE}"
        )

    with open(TEAM_MAPPING_FILE, "r", encoding="utf-8") as f:
        return json.load(f)


def get_scrapeable_teams(team_mapping: dict) -> list[dict]:
    """Get list of teams that can be scraped from Transfermarkt.

    Args:
        team_mapping: The full team mapping dictionary.

    Returns:
        List of team entries that have valid Transfermarkt aliases.
    """
    scrapeable = []
    for team in team_mapping.get("teams", []):
        aliases = team.get("aliases", {})
        tm_slug = aliases.get("transfermarkt")
        tm_id = aliases.get("transfermarkt_id")

        # Skip TBD teams and teams without valid Transfermarkt data
        if tm_slug and tm_slug != "TBD" and tm_id is not None:
            scrapeable.append(team)

    return scrapeable


@click.command()
@click.option(
    "--output",
    "-o",
    type=click.Path(),
    default=None,
    help="Output file path (default: output/transfermarkt_values.json)",
)
@click.option(
    "--team",
    "-t",
    type=str,
    default=None,
    help="Scrape only a specific team by canonical name",
)
@click.option(
    "--dry-run",
    is_flag=True,
    help="Show teams that would be scraped without making requests",
)
def scrape_transfermarkt(output: str | None, team: str | None, dry_run: bool) -> None:
    """Scrape national team market values from Transfermarkt.

    This command fetches the total squad market value for each World Cup 2026
    team from transfermarkt.us and saves the results to a JSON file.

    Note: Transfermarkt uses Cloudflare protection, so this scraper uses
    cloudscraper to bypass it. A 2-second delay is enforced between requests
    to be respectful to the server.
    """
    console.print("[bold blue]Transfermarkt Market Value Scraper[/bold blue]")
    console.print()

    # Load team mapping
    try:
        team_mapping = load_team_mapping()
    except click.ClickException:
        raise
    except Exception as e:
        raise click.ClickException(f"Failed to load team mapping: {e}")

    # Get scrapeable teams
    teams = get_scrapeable_teams(team_mapping)

    if not teams:
        raise click.ClickException("No teams with valid Transfermarkt data found")

    # Filter to specific team if requested
    if team:
        teams = [t for t in teams if t["canonical_name"].lower() == team.lower()]
        if not teams:
            raise click.ClickException(f"Team not found: {team}")

    console.print(f"Found [green]{len(teams)}[/green] teams to scrape")
    console.print()

    # Dry run - just show teams
    if dry_run:
        table = Table(title="Teams to Scrape")
        table.add_column("Team", style="cyan")
        table.add_column("Slug", style="magenta")
        table.add_column("ID", style="green")

        for t in teams:
            table.add_row(
                t["canonical_name"],
                t["aliases"]["transfermarkt"],
                str(t["aliases"]["transfermarkt_id"]),
            )

        console.print(table)
        return

    # Initialize scraper
    output_dir = Path(output).parent if output else OUTPUT_DIR
    scraper = TransfermarktScraper(output_dir=output_dir)

    # Scrape teams
    results: dict[str, float] = {}
    errors: list[str] = []

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TaskProgressColumn(),
        console=console,
    ) as progress:
        task = progress.add_task("Scraping teams...", total=len(teams))

        for team_data in teams:
            canonical_name = team_data["canonical_name"]
            tm_slug = team_data["aliases"]["transfermarkt"]
            tm_id = team_data["aliases"]["transfermarkt_id"]

            progress.update(task, description=f"Scraping {canonical_name}...")

            try:
                name, value = scraper.scrape_team(canonical_name, tm_slug, tm_id)
                results[name] = value
            except ScraperError as e:
                errors.append(f"{canonical_name}: {e}")
                console.print(f"[red]Error scraping {canonical_name}: {e}[/red]")
                # Fail fast - stop on first error
                raise click.ClickException(str(e))

            progress.advance(task)

    console.print()

    # Build output data
    output_data = {
        "teams": results,
        "source": "transfermarkt.us",
        "scraped_at": datetime.now(timezone.utc).isoformat(),
        "team_count": len(results),
    }

    # Determine output path
    output_path = Path(output) if output else OUTPUT_DIR / "transfermarkt_values.json"
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Save results
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)

    console.print(f"[green]Saved results to {output_path}[/green]")
    console.print()

    # Show summary
    table = Table(title="Market Values (Top 10)")
    table.add_column("Team", style="cyan")
    table.add_column("Value", style="green", justify="right")

    # Sort by value descending and show top 10
    sorted_teams = sorted(results.items(), key=lambda x: x[1], reverse=True)[:10]
    for name, value in sorted_teams:
        if value >= 1000:
            value_str = f"{value / 1000:.2f}bn"
        else:
            value_str = f"{value:.2f}m"
        table.add_row(name, value_str)

    console.print(table)
    console.print()
    console.print(f"[bold]Total teams scraped:[/bold] {len(results)}")


if __name__ == "__main__":
    scrape_transfermarkt()
