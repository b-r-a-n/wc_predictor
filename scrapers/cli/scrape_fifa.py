"""CLI command to scrape FIFA world rankings."""

import json
import sys
from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from scrapers.config.settings import OUTPUT_DIR, DATA_DIR
from scrapers.sources.base import ScraperError
from scrapers.sources.fifa_scraper import FifaScraper
from scrapers.utils.logging_config import setup_logging


console = Console()


def load_team_mapping() -> dict[str, str]:
    """Load team name mapping from data directory.

    Returns:
        Dict mapping FIFA names to internal names, or empty dict if not found.
    """
    # Try to load from teams.json to get known team names
    teams_file = DATA_DIR / "teams.json"
    if teams_file.exists():
        try:
            with open(teams_file, encoding="utf-8") as f:
                data = json.load(f)
                # Return dict of name -> name (for comparison)
                return {team["name"]: team["name"] for team in data.get("teams", [])}
        except (json.JSONDecodeError, KeyError) as e:
            console.print(f"[yellow]Warning: Could not load team mapping: {e}[/yellow]")

    return {}


def display_rankings(rankings: dict[str, int], known_teams: set[str]) -> None:
    """Display rankings in a formatted table.

    Args:
        rankings: Dict mapping team name to ranking position.
        known_teams: Set of team names from our data.
    """
    table = Table(title="FIFA World Rankings", show_header=True, header_style="bold cyan")
    table.add_column("Rank", style="dim", width=6, justify="right")
    table.add_column("Team", style="bold")
    table.add_column("Status", justify="center")

    # Sort by rank
    sorted_rankings = sorted(rankings.items(), key=lambda x: x[1])

    # Show top 50 teams
    for team_name, rank in sorted_rankings[:50]:
        if team_name in known_teams:
            status = "[green]In Data[/green]"
        else:
            status = "[dim]-[/dim]"
        table.add_row(str(rank), team_name, status)

    console.print(table)

    # Show summary
    matched = sum(1 for name in rankings if name in known_teams)
    console.print(
        f"\n[bold]Summary:[/bold] {len(rankings)} teams scraped, "
        f"{matched} match known teams in data"
    )


@click.command()
@click.option(
    "--output",
    "-o",
    type=click.Path(path_type=Path),
    default=None,
    help="Output file path. Defaults to output/fifa_rankings.json",
)
@click.option(
    "--quiet",
    "-q",
    is_flag=True,
    help="Suppress detailed output, only show errors.",
)
@click.option(
    "--verbose",
    "-v",
    is_flag=True,
    help="Enable verbose logging.",
)
def main(output: Path | None, quiet: bool, verbose: bool) -> None:
    """Scrape FIFA world rankings and save to JSON.

    Fetches the latest FIFA world rankings from fifa.com and saves
    them in a format suitable for the World Cup simulator.

    Output format:
        {
            "teams": {"Argentina": 1, "France": 2, ...},
            "source": "fifa.com",
            "scraped_at": "2024-01-15T12:00:00+00:00"
        }
    """
    import logging

    # Setup logging
    log_level = logging.DEBUG if verbose else logging.INFO
    if quiet:
        log_level = logging.ERROR
    setup_logging(level=log_level)

    # Determine output directory
    output_dir = output.parent if output else OUTPUT_DIR
    output_dir.mkdir(parents=True, exist_ok=True)

    if not quiet:
        console.print(
            Panel(
                "[bold blue]FIFA World Rankings Scraper[/bold blue]\n"
                "Fetching latest rankings from fifa.com",
                expand=False,
            )
        )

    # Load team mapping for reference
    team_mapping = load_team_mapping()
    known_teams = set(team_mapping.keys())

    if not quiet and known_teams:
        console.print(f"[dim]Loaded {len(known_teams)} teams from data file[/dim]\n")

    # Create scraper and run
    scraper = FifaScraper(output_dir=output_dir)

    try:
        with console.status("[bold green]Scraping FIFA rankings...") as status:
            data = scraper.scrape()

        # Save to file
        if output:
            # Custom output path - save directly
            with open(output, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            output_path = output
        else:
            output_path = scraper.save(data)

        if not quiet:
            console.print(f"[green]Successfully scraped FIFA rankings![/green]\n")

            # Display the rankings table
            display_rankings(data["teams"], known_teams)

            console.print(f"\n[bold]Source:[/bold] {data['source']}")
            console.print(f"[bold]Scraped at:[/bold] {data['scraped_at']}")
            console.print(f"\n[bold green]Saved to:[/bold green] {output_path}")
        else:
            # In quiet mode, just print the output path
            click.echo(str(output_path))

    except ScraperError as e:
        console.print(f"\n[bold red]Scraping failed:[/bold red] {e}")
        console.print(
            "\n[yellow]Troubleshooting tips:[/yellow]\n"
            "1. Check your internet connection\n"
            "2. Verify https://www.fifa.com/fifa-world-ranking/men is accessible\n"
            "3. The FIFA API may have changed - check for scraper updates\n"
            "4. Try again later if the site is temporarily unavailable"
        )
        sys.exit(1)

    except Exception as e:
        console.print(f"\n[bold red]Unexpected error:[/bold red] {e}")
        if verbose:
            console.print_exception()
        sys.exit(1)


if __name__ == "__main__":
    main()
