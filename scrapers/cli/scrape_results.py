"""CLI command to scrape completed match results."""

import sys
from datetime import date, datetime
from pathlib import Path

import click
from rich.console import Console
from rich.table import Table

from ..config.settings import OUTPUT_DIR
from ..sources.base import ScraperError
from ..sources.results_scraper import ResultsScraper
from ..utils.logging_config import setup_logging

CONFIG_DIR = Path(__file__).parent.parent / "config"
TEAM_MAPPING_FILE = CONFIG_DIR / "team_mapping.json"

console = Console()


def _parse_date(ctx, param, value: str | None) -> date | None:
    if value is None:
        return None
    try:
        return datetime.strptime(value, "%Y-%m-%d").date()
    except ValueError:
        raise click.BadParameter("Dates must be in YYYY-MM-DD format.")


@click.command()
@click.option(
    "--output-dir",
    "-o",
    type=click.Path(path_type=Path),
    default=OUTPUT_DIR,
    help="Output directory for scraped data.",
)
@click.option(
    "--schedule",
    "-s",
    type=click.Path(exists=True, path_type=Path),
    default=OUTPUT_DIR / "schedule.json",
    help="Path to schedule.json (defines matchNumbers and placeholders).",
)
@click.option(
    "--groups",
    "-g",
    type=click.Path(exists=True, path_type=Path),
    default=OUTPUT_DIR / "groups.json",
    help="Path to groups.json (defines group composition / positions).",
)
@click.option(
    "--league",
    type=str,
    default=None,
    help="ESPN soccer league slug (default: fifa.world).",
)
@click.option(
    "--start",
    callback=_parse_date,
    default=None,
    help="First date to scrape (YYYY-MM-DD). Defaults to the group-stage window.",
)
@click.option(
    "--end",
    callback=_parse_date,
    default=None,
    help="Last date to scrape (YYYY-MM-DD). Defaults to the group-stage window.",
)
@click.option("--verbose", "-v", is_flag=True, help="Enable verbose output.")
def main(
    output_dir: Path,
    schedule: Path,
    groups: Path,
    league: str | None,
    start: date | None,
    end: date | None,
    verbose: bool,
) -> None:
    """Scrape completed group-stage results from ESPN.

    Maps each finished match to its schedule matchNumber so the web app can
    pin already-played games. Re-run daily as more matches complete; the output
    is a full snapshot of every completed group-stage match found.
    """
    import logging

    setup_logging(level=logging.DEBUG if verbose else logging.INFO)

    console.print("[bold blue]Match Results Scraper[/bold blue]")
    console.print("[dim]Source: ESPN soccer scoreboard[/dim]")
    console.print()

    try:
        scraper = ResultsScraper(
            output_dir=output_dir,
            schedule_path=schedule,
            groups_path=groups,
            team_mapping_path=TEAM_MAPPING_FILE,
            league=league,
        )
        data = scraper.scrape(start=start, end=end)
    except ScraperError as e:
        console.print(f"[bold red]Scraper error:[/bold red] {e}")
        sys.exit(1)
    except Exception as e:  # noqa: BLE001
        console.print(f"[bold red]Unexpected error:[/bold red] {e}")
        sys.exit(1)

    output_path = scraper.save(data)
    console.print()
    console.print(f"[bold green]Saved to:[/bold green] {output_path}")

    matches = data.get("matches", [])
    if matches:
        table = Table(title=f"Completed Matches ({len(matches)})")
        table.add_column("#", justify="right", style="dim")
        table.add_column("Group", justify="center")
        table.add_column("Score", justify="center", style="green")
        table.add_column("Date", style="dim")
        for m in matches:
            table.add_row(
                str(m["matchNumber"]),
                m["groupId"],
                f'{m["homeScore"]}–{m["awayScore"]}',
                m.get("date", ""),
            )
        console.print()
        console.print(table)
    else:
        console.print(
            "[yellow]No completed group-stage matches found in the date range.[/yellow]"
        )

    console.print()
    console.print("[dim]Copy into the web app with:[/dim]")
    console.print(
        f"[dim]  cp {output_path} web/public/data/results.json[/dim]"
    )


if __name__ == "__main__":
    main()
