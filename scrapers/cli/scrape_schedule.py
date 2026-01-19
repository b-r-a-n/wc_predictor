"""CLI command to generate FIFA World Cup 2026 schedule."""

import sys
from pathlib import Path

import click
from rich.console import Console
from rich.table import Table

from ..config.settings import OUTPUT_DIR
from ..sources.base import ScraperError
from ..sources.schedule_scraper import ScheduleScraper
from ..utils.logging_config import setup_logging


# Path to team mapping config
CONFIG_DIR = Path(__file__).parent.parent / "config"
TEAM_MAPPING_FILE = CONFIG_DIR / "team_mapping.json"


console = Console()


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
    "--copy-to-web",
    "-c",
    is_flag=True,
    help="Copy output to web/public/data directory.",
)
def main(output_dir: Path, verbose: bool, copy_to_web: bool) -> None:
    """Generate FIFA World Cup 2026 match schedule.

    Creates a schedule.json file containing all 104 matches with dates,
    times, venues, and matchup information.
    """
    setup_logging()

    console.print("[bold blue]FIFA World Cup 2026 Schedule Generator[/bold blue]")
    console.print()

    try:
        scraper = ScheduleScraper(
            output_dir=output_dir,
            team_mapping_path=TEAM_MAPPING_FILE if TEAM_MAPPING_FILE.exists() else None,
        )
        data = scraper.scrape()
    except ScraperError as e:
        console.print(f"[bold red]Scraper error:[/bold red] {e}")
        sys.exit(1)
    except Exception as e:
        console.print(f"[bold red]Unexpected error:[/bold red] {e}")
        sys.exit(1)

    # Save output
    output_path = scraper.save(data)
    console.print(f"[bold green]Saved to:[/bold green] {output_path}")

    # Copy to web directory if requested
    if copy_to_web:
        web_data_dir = Path(__file__).parent.parent.parent / "web" / "public" / "data"
        if web_data_dir.exists():
            import shutil
            web_output = web_data_dir / "schedule.json"
            shutil.copy(output_path, web_output)
            console.print(f"[bold green]Copied to:[/bold green] {web_output}")
        else:
            console.print(f"[yellow]Warning: Web data directory not found: {web_data_dir}[/yellow]")

    # Display summary
    matches = data.get("matches", [])
    console.print()
    console.print("[bold]Summary:[/bold]")
    console.print(f"  Total matches: {len(matches)}")

    # Count by round
    round_counts: dict[str, int] = {}
    for match in matches:
        round_type = match.get("round", "unknown")
        round_counts[round_type] = round_counts.get(round_type, 0) + 1

    console.print("  Matches by round:")
    for round_type, count in sorted(round_counts.items()):
        console.print(f"    {round_type}: {count}")

    # Verbose output - show match table
    if verbose:
        console.print()

        # Group stage summary
        table = Table(title="Group Stage Matches (first 12)")
        table.add_column("Match", style="cyan", width=6)
        table.add_column("Date", width=12)
        table.add_column("Time", width=6)
        table.add_column("Venue", width=20)
        table.add_column("Group", width=6)
        table.add_column("Matchup", width=20)

        group_matches = [m for m in matches if m.get("round") == "group_stage"][:12]
        for match in group_matches:
            table.add_row(
                str(match.get("matchNumber", "")),
                match.get("date", ""),
                match.get("time", ""),
                match.get("venueId", ""),
                match.get("groupId", ""),
                f"{match.get('homePlaceholder', '?')} vs {match.get('awayPlaceholder', '?')}",
            )

        console.print(table)

        # Knockout summary
        console.print()
        table2 = Table(title="Knockout Stage Matches")
        table2.add_column("Match", style="cyan", width=6)
        table2.add_column("Date", width=12)
        table2.add_column("Time", width=6)
        table2.add_column("Round", width=15)
        table2.add_column("Venue", width=20)
        table2.add_column("Slot", width=5)

        knockout_matches = [m for m in matches if m.get("round") != "group_stage"]
        for match in knockout_matches:
            table2.add_row(
                str(match.get("matchNumber", "")),
                match.get("date", ""),
                match.get("time", ""),
                match.get("round", ""),
                match.get("venueId", ""),
                str(match.get("knockoutSlot", "")),
            )

        console.print(table2)

    console.print()
    console.print(f"[dim]Generated at: {data.get('lastUpdated', 'unknown')}[/dim]")


if __name__ == "__main__":
    main()
