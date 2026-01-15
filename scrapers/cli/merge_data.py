"""CLI command to merge scraped data into final teams.json."""

import json
import sys
from pathlib import Path
from typing import Any

import click
from rich.console import Console
from rich.panel import Panel
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.table import Table

from scrapers.models.team import Team, Group, TournamentData, Confederation


console = Console()


def load_json_file(path: Path, description: str) -> dict:
    """Load a JSON file with error handling.

    Args:
        path: Path to the JSON file.
        description: Description for error messages.

    Returns:
        Parsed JSON data.

    Raises:
        click.ClickException: If file cannot be loaded.
    """
    if not path.exists():
        raise click.ClickException(f"{description} not found: {path}")

    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except json.JSONDecodeError as e:
        raise click.ClickException(f"Invalid JSON in {description}: {e}")


def build_team_lookups(team_mapping: dict) -> dict[str, dict]:
    """Build lookup dictionaries from team mapping.

    Args:
        team_mapping: The loaded team mapping dictionary.

    Returns:
        Dictionary with various lookup maps:
        - by_id: team data indexed by ID
        - by_canonical: team data indexed by canonical_name
        - elo_to_id: ELO name -> team ID
        - fifa_to_id: FIFA name -> team ID
    """
    lookups = {
        "by_id": {},
        "by_canonical": {},
        "elo_to_id": {},
        "fifa_to_id": {},
    }

    for team in team_mapping.get("teams", []):
        team_id = team["id"]
        canonical = team["canonical_name"]
        aliases = team.get("aliases", {})

        lookups["by_id"][team_id] = team
        lookups["by_canonical"][canonical] = team

        elo_name = aliases.get("elo", "")
        if elo_name and elo_name != "TBD":
            lookups["elo_to_id"][elo_name] = team_id

        fifa_name = aliases.get("fifa", "")
        if fifa_name and fifa_name != "TBD":
            lookups["fifa_to_id"][fifa_name] = team_id

    return lookups


def get_elo_rating(
    team: dict,
    elo_data: dict,
    lookups: dict,
) -> float | None:
    """Get ELO rating for a team.

    Args:
        team: Team data from mapping.
        elo_data: Scraped ELO ratings data.
        lookups: Lookup dictionaries.

    Returns:
        ELO rating or None if not found.
    """
    aliases = team.get("aliases", {})
    elo_name = aliases.get("elo", "")

    if not elo_name or elo_name == "TBD":
        return None

    # Check matched_teams first (canonical name -> rating)
    matched = elo_data.get("matched_teams", {})
    canonical = team["canonical_name"]
    if canonical in matched:
        return float(matched[canonical])

    # Fall back to raw teams data (elo name -> rating)
    teams = elo_data.get("teams", {})
    if elo_name in teams:
        return float(teams[elo_name])

    return None


def get_market_value(
    team: dict,
    transfermarkt_data: dict,
) -> float | None:
    """Get market value for a team.

    Args:
        team: Team data from mapping.
        transfermarkt_data: Scraped Transfermarkt data.

    Returns:
        Market value in millions or None if not found.
    """
    canonical = team["canonical_name"]
    teams = transfermarkt_data.get("teams", {})

    if canonical in teams:
        return float(teams[canonical])

    return None


def get_fifa_ranking(
    team: dict,
    fifa_data: dict,
    lookups: dict,
) -> int | None:
    """Get FIFA ranking for a team.

    Args:
        team: Team data from mapping.
        fifa_data: Scraped FIFA rankings data.
        lookups: Lookup dictionaries.

    Returns:
        FIFA ranking or None if not found.
    """
    aliases = team.get("aliases", {})
    fifa_name = aliases.get("fifa", "")

    if not fifa_name or fifa_name == "TBD":
        return None

    teams = fifa_data.get("teams", {})
    if fifa_name in teams:
        return int(teams[fifa_name])

    return None


def build_groups(
    groups_data: dict,
    lookups: dict,
) -> list[dict]:
    """Build groups array with team IDs.

    Args:
        groups_data: Scraped groups data.
        lookups: Lookup dictionaries.

    Returns:
        List of group dictionaries with team IDs.

    Raises:
        click.ClickException: If a team cannot be mapped.
    """
    groups = []
    raw_groups = groups_data.get("groups", {})

    for letter in "ABCDEFGHIJKL":
        team_names = raw_groups.get(letter, [])
        team_ids = []

        for name in team_names:
            # First try canonical name lookup
            if name in lookups["by_canonical"]:
                team_ids.append(lookups["by_canonical"][name]["id"])
            else:
                # Handle TBD teams by searching for matching canonical name
                found = False
                for canonical, team in lookups["by_canonical"].items():
                    if name in canonical or canonical in name:
                        team_ids.append(team["id"])
                        found = True
                        break

                if not found:
                    raise click.ClickException(
                        f"Could not map team '{name}' in Group {letter} to ID"
                    )

        if len(team_ids) != 4:
            raise click.ClickException(
                f"Group {letter} has {len(team_ids)} teams, expected 4"
            )

        groups.append({"id": letter, "teams": team_ids})

    return groups


def get_default_values_for_tbd(team: dict) -> dict:
    """Get default values for TBD playoff teams.

    Uses average values from potential playoff teams.

    Args:
        team: Team data from mapping.

    Returns:
        Dictionary with default elo_rating, market_value, fifa_ranking.
    """
    # Default values based on typical playoff team ranges
    confederation = team.get("confederation", "UEFA")

    if confederation == "UEFA":
        # UEFA playoff teams are generally strong
        return {
            "elo_rating": 1700.0,
            "market_value_millions": 300.0,
            "fifa_ranking": 30,
        }
    else:
        # Intercontinental playoff teams vary more
        return {
            "elo_rating": 1500.0,
            "market_value_millions": 50.0,
            "fifa_ranking": 60,
        }


@click.command()
@click.option(
    "--mapping",
    "-m",
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help="Path to team_mapping.json",
)
@click.option(
    "--elo",
    "-e",
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help="Path to elo_ratings.json",
)
@click.option(
    "--transfermarkt",
    "-t",
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help="Path to transfermarkt_values.json",
)
@click.option(
    "--fifa",
    "-f",
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help="Path to fifa_rankings.json",
)
@click.option(
    "--groups",
    "-g",
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help="Path to groups.json",
)
@click.option(
    "--output",
    "-o",
    type=click.Path(path_type=Path),
    required=True,
    help="Path to output teams.json",
)
@click.option(
    "--allow-tbd-defaults",
    is_flag=True,
    help="Allow default values for TBD playoff teams instead of failing",
)
@click.option(
    "--allow-missing-fifa",
    is_flag=True,
    help="Allow default FIFA rankings for teams not in top 20",
)
@click.option(
    "--verbose",
    "-v",
    is_flag=True,
    help="Enable verbose output",
)
def merge_data(
    mapping: Path,
    elo: Path,
    transfermarkt: Path,
    fifa: Path,
    groups: Path,
    output: Path,
    allow_tbd_defaults: bool,
    allow_missing_fifa: bool,
    verbose: bool,
) -> None:
    """Merge scraped data into final teams.json for the simulator.

    This command combines data from multiple sources:
    - team_mapping.json: Static team info (id, name, code, confederation, world_cup_wins)
    - elo_ratings.json: ELO ratings from eloratings.net
    - transfermarkt_values.json: Market values from Transfermarkt
    - fifa_rankings.json: FIFA world rankings
    - groups.json: Group assignments

    The output matches the schema expected by the Rust wc-core crate.
    """
    console.print(
        Panel(
            "[bold blue]World Cup Data Merger[/bold blue]\n"
            "Combining scraped data into teams.json",
            expand=False,
        )
    )
    console.print()

    # Load all input files
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console,
    ) as progress:
        task = progress.add_task("Loading input files...", total=5)

        progress.update(task, description="Loading team mapping...")
        team_mapping = load_json_file(mapping, "Team mapping")
        progress.advance(task)

        progress.update(task, description="Loading ELO ratings...")
        elo_data = load_json_file(elo, "ELO ratings")
        progress.advance(task)

        progress.update(task, description="Loading Transfermarkt values...")
        transfermarkt_data = load_json_file(transfermarkt, "Transfermarkt values")
        progress.advance(task)

        progress.update(task, description="Loading FIFA rankings...")
        fifa_data = load_json_file(fifa, "FIFA rankings")
        progress.advance(task)

        progress.update(task, description="Loading groups...")
        groups_data = load_json_file(groups, "Groups")
        progress.advance(task)

    console.print("[green]All input files loaded successfully[/green]")
    console.print()

    # Build lookup tables
    lookups = build_team_lookups(team_mapping)
    console.print(f"[dim]Built lookups for {len(lookups['by_id'])} teams[/dim]")

    # Merge team data
    console.print()
    console.print("[bold]Merging team data...[/bold]")

    teams: list[dict[str, Any]] = []
    errors: list[str] = []
    warnings: list[str] = []

    for team_data in team_mapping.get("teams", []):
        team_id = team_data["id"]
        canonical = team_data["canonical_name"]
        is_tbd = team_data.get("playoff", False)

        # Get ELO rating
        elo_rating = get_elo_rating(team_data, elo_data, lookups)

        # Get market value
        market_value = get_market_value(team_data, transfermarkt_data)

        # Get FIFA ranking
        fifa_ranking = get_fifa_ranking(team_data, fifa_data, lookups)

        # Handle missing data
        if is_tbd:
            if elo_rating is None or market_value is None or fifa_ranking is None:
                if allow_tbd_defaults:
                    defaults = get_default_values_for_tbd(team_data)
                    if elo_rating is None:
                        elo_rating = defaults["elo_rating"]
                        warnings.append(f"{canonical}: Using default ELO rating")
                    if market_value is None:
                        market_value = defaults["market_value_millions"]
                        warnings.append(f"{canonical}: Using default market value")
                    if fifa_ranking is None:
                        fifa_ranking = defaults["fifa_ranking"]
                        warnings.append(f"{canonical}: Using default FIFA ranking")
                else:
                    errors.append(
                        f"{canonical}: Missing data (TBD team). "
                        "Use --allow-tbd-defaults to use placeholder values."
                    )
                    continue
        else:
            # Non-TBD teams must have all data (unless allowing missing FIFA)
            missing = []
            if elo_rating is None:
                missing.append("ELO rating")
            if market_value is None:
                missing.append("market value")
            if fifa_ranking is None:
                if allow_missing_fifa:
                    # Estimate FIFA ranking based on ELO rating
                    # Top teams (ELO > 1900) are roughly ranked 1-20
                    # Mid teams (ELO 1700-1900) are roughly ranked 20-50
                    # Lower teams (ELO < 1700) are roughly ranked 50-100
                    if elo_rating and elo_rating > 1900:
                        fifa_ranking = 25
                    elif elo_rating and elo_rating > 1700:
                        fifa_ranking = 45
                    else:
                        fifa_ranking = 70
                    warnings.append(f"{canonical}: Using estimated FIFA ranking ({fifa_ranking})")
                else:
                    missing.append("FIFA ranking")

            if missing:
                errors.append(f"{canonical}: Missing {', '.join(missing)}")
                continue

        # Handle confederation for TBD intercontinental teams
        confederation = team_data["confederation"]
        if confederation == "TBD":
            # Default to the most likely confederation for intercontinental playoffs
            confederation = "CONCACAF"

        # Build team entry
        team_entry = {
            "id": team_id,
            "name": team_data["canonical_name"],
            "code": team_data["fifa_code"],
            "confederation": confederation,
            "elo_rating": elo_rating,
            "market_value_millions": market_value,
            "fifa_ranking": fifa_ranking,
            "world_cup_wins": team_data.get("world_cup_wins", 0),
        }

        teams.append(team_entry)

        if verbose:
            console.print(
                f"  [green]+[/green] {canonical} "
                f"(ELO: {elo_rating:.0f}, Market: {market_value:.1f}M, FIFA: {fifa_ranking})"
            )

    # Report warnings
    if warnings:
        console.print()
        console.print(f"[yellow]Warnings ({len(warnings)}):[/yellow]")
        for warning in warnings[:10]:
            console.print(f"  [dim]- {warning}[/dim]")
        if len(warnings) > 10:
            console.print(f"  [dim]... and {len(warnings) - 10} more[/dim]")

    # Fail fast on errors
    if errors:
        console.print()
        console.print(f"[bold red]Errors ({len(errors)}):[/bold red]")
        for error in errors:
            console.print(f"  [red]- {error}[/red]")
        raise click.ClickException(
            f"Failed to merge data: {len(errors)} teams missing required data"
        )

    console.print()
    console.print(f"[green]Merged {len(teams)} teams successfully[/green]")

    # Build groups
    console.print()
    console.print("[bold]Building groups...[/bold]")

    try:
        groups_list = build_groups(groups_data, lookups)
        console.print(f"[green]Built {len(groups_list)} groups[/green]")
    except click.ClickException:
        raise

    # Sort teams by ID for consistent output
    teams.sort(key=lambda t: t["id"])

    # Build final output
    output_data = {
        "teams": teams,
        "groups": groups_list,
    }

    # Validate with Pydantic
    console.print()
    console.print("[bold]Validating output...[/bold]")

    try:
        validated = TournamentData(**output_data)
        console.print("[green]Validation passed[/green]")
    except Exception as e:
        console.print(f"[bold red]Validation failed:[/bold red] {e}")
        raise click.ClickException(f"Output validation failed: {e}")

    # Save output
    output.parent.mkdir(parents=True, exist_ok=True)

    with open(output, "w", encoding="utf-8") as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)

    console.print()
    console.print(f"[bold green]Saved to:[/bold green] {output}")

    # Display summary
    console.print()
    table = Table(title="Merge Summary")
    table.add_column("Metric", style="cyan")
    table.add_column("Value", justify="right", style="green")

    table.add_row("Total teams", str(len(teams)))
    table.add_row("Total groups", str(len(groups_list)))
    table.add_row("TBD playoff teams", str(sum(1 for t in teams if "TBD" in t["name"])))
    table.add_row(
        "Average ELO",
        f"{sum(t['elo_rating'] for t in teams) / len(teams):.0f}",
    )
    table.add_row(
        "Total market value",
        f"{sum(t['market_value_millions'] for t in teams):.0f}M",
    )

    console.print(table)


if __name__ == "__main__":
    merge_data()
