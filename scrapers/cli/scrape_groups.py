"""CLI command for scraping World Cup 2026 group assignments."""

import json
import sys
from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from scrapers.config.settings import OUTPUT_DIR
from scrapers.sources.base import ScraperError
from scrapers.sources.groups_scraper import GroupsScraper
from scrapers.utils.logging_config import setup_logging


console = Console()


@click.command()
@click.option(
    "--output",
    "-o",
    type=click.Path(path_type=Path),
    default=OUTPUT_DIR / "groups.json",
    help="Output file path for groups JSON.",
)
@click.option(
    "--verify",
    "-v",
    is_flag=True,
    help="Verify scraped data against team_mapping.json.",
)
@click.option(
    "--quiet",
    "-q",
    is_flag=True,
    help="Suppress detailed output.",
)
@click.option(
    "--debug",
    is_flag=True,
    help="Enable debug logging.",
)
def scrape_groups(
    output: Path,
    verify: bool,
    quiet: bool,
    debug: bool,
) -> None:
    """Scrape FIFA World Cup 2026 group assignments.

    Fetches official group draw data from FIFA website. Falls back to
    team_mapping.json if FIFA website is unavailable or hard to parse.

    Output includes all 12 groups (A-L) with 4 teams each, plus metadata
    about TBD playoff spots.
    """
    import logging

    setup_logging(level=logging.DEBUG if debug else logging.INFO)

    output_dir = output.parent
    output_dir.mkdir(parents=True, exist_ok=True)

    scraper = GroupsScraper(output_dir)

    if verify:
        _run_verification(scraper, quiet)
        return

    try:
        if not quiet:
            console.print(
                Panel(
                    "[bold blue]World Cup 2026 Groups Scraper[/bold blue]\n"
                    "Fetching official group assignments...",
                    expand=False,
                )
            )

        data = scraper.scrape()
        scraper.save(data)

        if not quiet:
            _display_groups(data)
            console.print(
                f"\n[green]Saved to:[/green] {output_dir / scraper.get_output_filename()}"
            )

    except ScraperError as e:
        console.print(f"[red]Error:[/red] {e}")
        sys.exit(1)
    except Exception as e:
        console.print(f"[red]Unexpected error:[/red] {e}")
        if debug:
            console.print_exception()
        sys.exit(1)


def _display_groups(data: dict) -> None:
    """Display groups data in a formatted table.

    Args:
        data: Groups data dictionary.
    """
    groups = data.get("groups", {})
    meta = data.get("meta", {})

    # Create main groups table
    table = Table(
        title="[bold]FIFA World Cup 2026 Groups[/bold]",
        show_header=True,
        header_style="bold cyan",
    )

    table.add_column("Group", style="bold yellow", width=8)
    table.add_column("Team 1", width=25)
    table.add_column("Team 2", width=25)
    table.add_column("Team 3", width=25)
    table.add_column("Team 4", width=25)

    for letter in "ABCDEFGHIJKL":
        teams = groups.get(letter, ["?"] * 4)
        formatted_teams = []
        for team in teams:
            if "TBD" in team or "Playoff" in team:
                formatted_teams.append(f"[dim italic]{team}[/dim italic]")
            else:
                formatted_teams.append(team)

        table.add_row(f"Group {letter}", *formatted_teams)

    console.print()
    console.print(table)

    # Display metadata
    console.print()
    console.print(f"[bold]Source:[/bold] {data.get('source', 'unknown')}")
    console.print(f"[bold]Scraped at:[/bold] {data.get('scraped_at', 'unknown')}")
    console.print(
        f"[bold]Total teams:[/bold] {meta.get('total_teams', 48)} "
        f"([dim]{meta.get('tbd_spots', 0)} TBD playoff spots[/dim])"
    )


def _run_verification(scraper: GroupsScraper, quiet: bool) -> None:
    """Run verification of scraped data against team_mapping.json.

    Args:
        scraper: GroupsScraper instance.
        quiet: Whether to suppress detailed output.
    """
    if not quiet:
        console.print(
            Panel(
                "[bold blue]Verification Mode[/bold blue]\n"
                "Comparing FIFA website data with team_mapping.json...",
                expand=False,
            )
        )

    result = scraper.verify_against_mapping()

    if not result["scraped_available"]:
        console.print(
            "[yellow]Warning:[/yellow] Could not scrape FIFA website for comparison."
        )
        if result["mapping_available"]:
            console.print("[green]team_mapping.json is available and readable.[/green]")
        return

    if not result["mapping_available"]:
        console.print(
            "[yellow]Warning:[/yellow] team_mapping.json not found or unreadable."
        )
        console.print("[green]FIFA website data is available.[/green]")
        return

    if result["match"]:
        console.print(
            "[bold green]Match![/bold green] "
            "FIFA website and team_mapping.json have identical group assignments."
        )
    else:
        console.print(
            "[bold red]Mismatch![/bold red] "
            "Differences found between FIFA website and team_mapping.json:"
        )

        diff_table = Table(show_header=True, header_style="bold")
        diff_table.add_column("Group")
        diff_table.add_column("FIFA Website")
        diff_table.add_column("team_mapping.json")

        for diff in result["differences"]:
            diff_table.add_row(
                f"Group {diff['group']}",
                "\n".join(diff["scraped"]),
                "\n".join(diff["mapping"]),
            )

        console.print(diff_table)


if __name__ == "__main__":
    scrape_groups()
