"""CLI command to scrape completed match results."""

import sys
from datetime import date, datetime
from pathlib import Path

import click
from rich.console import Console
from rich.table import Table

from ..config.settings import OUTPUT_DIR, PROJECT_ROOT
from ..sources.base import ScraperError
from ..sources.results_scraper import ResultsScraper
from ..utils.logging_config import setup_logging

CONFIG_DIR = Path(__file__).parent.parent / "config"
TEAM_MAPPING_FILE = CONFIG_DIR / "team_mapping.json"
# Committed data shipped with the web app — always present, including in CI.
DEFAULT_SCHEDULE = PROJECT_ROOT / "web" / "public" / "data" / "schedule.json"
DEFAULT_VENUES = PROJECT_ROOT / "web" / "public" / "data" / "venues.json"

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
    default=DEFAULT_SCHEDULE,
    help="Path to schedule.json (defines matchNumbers and placeholders).",
)
@click.option(
    "--groups",
    "-g",
    type=click.Path(exists=True, path_type=Path),
    default=None,
    help="Path to groups.json. Defaults to the draw in team_mapping.json.",
)
@click.option(
    "--venues",
    type=click.Path(exists=True, path_type=Path),
    default=DEFAULT_VENUES,
    help="Path to venues.json (used to map knockout matches by venue).",
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
    venues: Path,
    league: str | None,
    start: date | None,
    end: date | None,
    verbose: bool,
) -> None:
    """Scrape completed results (group stage and knockout) from ESPN.

    Maps each finished match to its schedule matchNumber so the web app can
    pin already-played games. Re-run daily as more matches complete; the output
    is a full snapshot of every completed match found.
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
            venues_path=venues,
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
        round_labels = {
            "round_of_32": "R32",
            "round_of_16": "R16",
            "quarter_finals": "QF",
            "semi_finals": "SF",
            "third_place": "3rd",
            "final": "Final",
        }
        table = Table(title=f"Completed Matches ({len(matches)})")
        table.add_column("#", justify="right", style="dim")
        table.add_column("Stage", justify="center")
        table.add_column("Score", justify="center", style="green")
        table.add_column("Date", style="dim")
        for m in matches:
            stage = m.get("groupId") or round_labels.get(m.get("round", ""), "KO")
            score = f'{m["homeScore"]}–{m["awayScore"]}'
            if "winnerTeamId" in m:
                # Mark which side won (knockout results may be decided on pens).
                score += " W:home" if m["winnerTeamId"] == m["homeTeamId"] else " W:away"
            table.add_row(
                str(m["matchNumber"]),
                stage,
                score,
                m.get("date", ""),
            )
        console.print()
        console.print(table)
    else:
        console.print(
            "[yellow]No completed matches found in the date range.[/yellow]"
        )

    console.print()
    console.print("[dim]Copy into the web app with:[/dim]")
    console.print(
        f"[dim]  cp {output_path} web/public/data/results.json[/dim]"
    )


if __name__ == "__main__":
    main()
