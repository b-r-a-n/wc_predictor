"""CLI command to validate teams.json against the schema."""

import json
import sys
from collections import Counter
from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from scrapers.models.team import TournamentData


console = Console()


class ValidationResult:
    """Container for validation results."""

    def __init__(self) -> None:
        self.passed: list[str] = []
        self.failed: list[str] = []
        self.warnings: list[str] = []

    def add_pass(self, check: str) -> None:
        """Record a passed check."""
        self.passed.append(check)

    def add_fail(self, check: str) -> None:
        """Record a failed check."""
        self.failed.append(check)

    def add_warning(self, message: str) -> None:
        """Record a warning."""
        self.warnings.append(message)

    @property
    def is_valid(self) -> bool:
        """Return True if all checks passed."""
        return len(self.failed) == 0


def validate_team_count(data: dict, result: ValidationResult) -> None:
    """Validate exactly 48 teams are present.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    count = len(teams)

    if count == 48:
        result.add_pass(f"Team count: {count} (expected 48)")
    else:
        result.add_fail(f"Team count: {count} (expected 48)")


def validate_group_count(data: dict, result: ValidationResult) -> None:
    """Validate exactly 12 groups are present.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    groups = data.get("groups", [])
    count = len(groups)

    if count == 12:
        result.add_pass(f"Group count: {count} (expected 12)")
    else:
        result.add_fail(f"Group count: {count} (expected 12)")


def validate_team_ids(data: dict, result: ValidationResult) -> None:
    """Validate all team IDs are 0-47 and present.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    ids = [t.get("id") for t in teams]
    expected_ids = set(range(48))
    actual_ids = set(ids)

    # Check for missing IDs
    missing = expected_ids - actual_ids
    if missing:
        result.add_fail(f"Missing team IDs: {sorted(missing)}")
    else:
        result.add_pass("All team IDs 0-47 are present")

    # Check for extra IDs
    extra = actual_ids - expected_ids
    if extra:
        result.add_fail(f"Invalid team IDs (out of range): {sorted(extra)}")


def validate_no_duplicate_ids(data: dict, result: ValidationResult) -> None:
    """Validate no duplicate team IDs.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    ids = [t.get("id") for t in teams]
    counter = Counter(ids)

    duplicates = {id_: count for id_, count in counter.items() if count > 1}
    if duplicates:
        result.add_fail(f"Duplicate team IDs: {duplicates}")
    else:
        result.add_pass("No duplicate team IDs")


def validate_group_team_references(data: dict, result: ValidationResult) -> None:
    """Validate all group team references exist.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    groups = data.get("groups", [])

    valid_ids = {t.get("id") for t in teams}
    invalid_refs = []

    for group in groups:
        group_id = group.get("id", "?")
        team_ids = group.get("teams", [])

        for team_id in team_ids:
            if team_id not in valid_ids:
                invalid_refs.append((group_id, team_id))

    if invalid_refs:
        result.add_fail(f"Invalid group team references: {invalid_refs}")
    else:
        result.add_pass("All group team references are valid")


def validate_group_structure(data: dict, result: ValidationResult) -> None:
    """Validate group structure (4 teams each, A-L in order).

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    groups = data.get("groups", [])

    # Check group IDs are A-L in order
    expected_ids = [chr(ord("A") + i) for i in range(12)]
    actual_ids = [g.get("id") for g in groups]

    if actual_ids != expected_ids:
        result.add_fail(f"Group IDs not A-L in order: {actual_ids}")
    else:
        result.add_pass("Group IDs are A-L in correct order")

    # Check each group has exactly 4 teams
    wrong_counts = []
    for group in groups:
        group_id = group.get("id", "?")
        team_count = len(group.get("teams", []))
        if team_count != 4:
            wrong_counts.append((group_id, team_count))

    if wrong_counts:
        result.add_fail(f"Groups with wrong team count: {wrong_counts}")
    else:
        result.add_pass("All groups have exactly 4 teams")


def validate_elo_ratings(data: dict, result: ValidationResult) -> None:
    """Validate ELO ratings are in valid range (1000-2500).

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    out_of_range = []

    for team in teams:
        name = team.get("name", "Unknown")
        elo = team.get("elo_rating", 0)

        if not (1000 <= elo <= 2500):
            out_of_range.append((name, elo))

    if out_of_range:
        result.add_fail(f"ELO ratings out of range (1000-2500): {out_of_range[:5]}")
        if len(out_of_range) > 5:
            result.add_fail(f"  ... and {len(out_of_range) - 5} more")
    else:
        result.add_pass("All ELO ratings in valid range (1000-2500)")


def validate_fifa_rankings(data: dict, result: ValidationResult) -> None:
    """Validate FIFA rankings are in valid range (1-211).

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    out_of_range = []

    for team in teams:
        name = team.get("name", "Unknown")
        ranking = team.get("fifa_ranking", 0)

        if not (1 <= ranking <= 211):
            out_of_range.append((name, ranking))

    if out_of_range:
        result.add_fail(f"FIFA rankings out of range (1-211): {out_of_range[:5]}")
        if len(out_of_range) > 5:
            result.add_fail(f"  ... and {len(out_of_range) - 5} more")
    else:
        result.add_pass("All FIFA rankings in valid range (1-211)")


def validate_market_values(data: dict, result: ValidationResult) -> None:
    """Validate market values are non-negative.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    negative = []

    for team in teams:
        name = team.get("name", "Unknown")
        value = team.get("market_value_millions", 0)

        if value < 0:
            negative.append((name, value))

    if negative:
        result.add_fail(f"Negative market values: {negative}")
    else:
        result.add_pass("All market values are non-negative")


def validate_confederations(data: dict, result: ValidationResult) -> None:
    """Validate confederations are valid.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    valid_confederations = {"UEFA", "CONMEBOL", "CONCACAF", "CAF", "AFC", "OFC"}
    teams = data.get("teams", [])
    invalid = []

    for team in teams:
        name = team.get("name", "Unknown")
        conf = team.get("confederation", "")

        if conf not in valid_confederations:
            invalid.append((name, conf))

    if invalid:
        result.add_fail(f"Invalid confederations: {invalid}")
    else:
        result.add_pass("All confederations are valid")


def validate_team_codes(data: dict, result: ValidationResult) -> None:
    """Validate team codes are 3-letter uppercase.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    invalid = []

    for team in teams:
        name = team.get("name", "Unknown")
        code = team.get("code", "")

        if not (len(code) == 3 and code.isupper() and code.isalpha()):
            invalid.append((name, code))

    if invalid:
        result.add_fail(f"Invalid team codes (not 3-letter uppercase): {invalid}")
    else:
        result.add_pass("All team codes are valid 3-letter uppercase")


def validate_world_cup_wins(data: dict, result: ValidationResult) -> None:
    """Validate world cup wins are non-negative integers.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    teams = data.get("teams", [])
    invalid = []

    for team in teams:
        name = team.get("name", "Unknown")
        wins = team.get("world_cup_wins")

        if not isinstance(wins, int) or wins < 0:
            invalid.append((name, wins))

    if invalid:
        result.add_fail(f"Invalid world cup wins: {invalid}")
    else:
        result.add_pass("All world cup wins are valid non-negative integers")


def validate_pydantic(data: dict, result: ValidationResult) -> None:
    """Validate against Pydantic TournamentData model.

    Args:
        data: The loaded tournament data.
        result: ValidationResult to update.
    """
    try:
        TournamentData(**data)
        result.add_pass("Pydantic TournamentData validation passed")
    except Exception as e:
        result.add_fail(f"Pydantic validation failed: {e}")


def display_results(result: ValidationResult) -> None:
    """Display validation results in a formatted table.

    Args:
        result: The validation results to display.
    """
    table = Table(title="Validation Results", show_header=True, header_style="bold")
    table.add_column("Status", width=8, justify="center")
    table.add_column("Check")

    for check in result.passed:
        table.add_row("[green]PASS[/green]", check)

    for check in result.failed:
        table.add_row("[red]FAIL[/red]", check)

    console.print(table)

    if result.warnings:
        console.print()
        console.print("[yellow]Warnings:[/yellow]")
        for warning in result.warnings:
            console.print(f"  [dim]- {warning}[/dim]")


def display_summary(data: dict, result: ValidationResult) -> None:
    """Display data summary.

    Args:
        data: The loaded tournament data.
        result: The validation results.
    """
    teams = data.get("teams", [])

    if not teams:
        return

    console.print()
    summary_table = Table(title="Data Summary")
    summary_table.add_column("Metric", style="cyan")
    summary_table.add_column("Value", justify="right", style="green")

    summary_table.add_row("Total teams", str(len(teams)))
    summary_table.add_row("Total groups", str(len(data.get("groups", []))))

    # ELO stats
    elos = [t.get("elo_rating", 0) for t in teams]
    summary_table.add_row("ELO range", f"{min(elos):.0f} - {max(elos):.0f}")
    summary_table.add_row("Average ELO", f"{sum(elos) / len(elos):.0f}")

    # FIFA ranking stats
    rankings = [t.get("fifa_ranking", 0) for t in teams]
    summary_table.add_row("FIFA ranking range", f"{min(rankings)} - {max(rankings)}")

    # Market value stats
    values = [t.get("market_value_millions", 0) for t in teams]
    summary_table.add_row("Total market value", f"{sum(values):.0f}M")
    summary_table.add_row("Average market value", f"{sum(values) / len(values):.1f}M")

    # World Cup wins
    total_wins = sum(t.get("world_cup_wins", 0) for t in teams)
    winners = sum(1 for t in teams if t.get("world_cup_wins", 0) > 0)
    summary_table.add_row("Past World Cup winners", f"{winners} teams ({total_wins} titles)")

    # Confederation breakdown
    conf_counts = Counter(t.get("confederation") for t in teams)
    for conf in ["UEFA", "CONMEBOL", "CONCACAF", "CAF", "AFC", "OFC"]:
        count = conf_counts.get(conf, 0)
        summary_table.add_row(f"{conf} teams", str(count))

    console.print(summary_table)


@click.command()
@click.argument(
    "teams_file",
    type=click.Path(exists=True, path_type=Path),
)
@click.option(
    "--quiet",
    "-q",
    is_flag=True,
    help="Only output pass/fail status",
)
@click.option(
    "--summary",
    "-s",
    is_flag=True,
    help="Show data summary after validation",
)
def validate(teams_file: Path, quiet: bool, summary: bool) -> None:
    """Validate teams.json against the schema.

    TEAMS_FILE is the path to the teams.json file to validate.

    Performs the following checks:
    - Exactly 48 teams
    - Exactly 12 groups
    - All team IDs 0-47 are present
    - No duplicate IDs
    - All group team references exist
    - ELO ratings in valid range (1000-2500)
    - FIFA rankings in valid range (1-211)
    - Market values are non-negative
    - Valid confederations
    - Valid team codes (3-letter uppercase)
    - Valid world cup wins (non-negative integers)
    - Pydantic TournamentData model validation

    Returns exit code 0 on success, 1 on failure.
    """
    if not quiet:
        console.print(
            Panel(
                "[bold blue]Teams.json Validator[/bold blue]\n"
                f"Validating: {teams_file}",
                expand=False,
            )
        )
        console.print()

    # Load the file
    try:
        with open(teams_file, "r", encoding="utf-8") as f:
            data = json.load(f)
    except json.JSONDecodeError as e:
        if quiet:
            click.echo("FAIL: Invalid JSON")
        else:
            console.print(f"[bold red]Invalid JSON:[/bold red] {e}")
        sys.exit(1)

    # Run all validations
    result = ValidationResult()

    validate_team_count(data, result)
    validate_group_count(data, result)
    validate_team_ids(data, result)
    validate_no_duplicate_ids(data, result)
    validate_group_team_references(data, result)
    validate_group_structure(data, result)
    validate_elo_ratings(data, result)
    validate_fifa_rankings(data, result)
    validate_market_values(data, result)
    validate_confederations(data, result)
    validate_team_codes(data, result)
    validate_world_cup_wins(data, result)
    validate_pydantic(data, result)

    # Display results
    if quiet:
        if result.is_valid:
            click.echo("PASS")
        else:
            click.echo("FAIL")
            for check in result.failed:
                click.echo(f"  - {check}")
    else:
        display_results(result)

        console.print()
        if result.is_valid:
            console.print(
                f"[bold green]Validation PASSED[/bold green] "
                f"({len(result.passed)} checks passed)"
            )
        else:
            console.print(
                f"[bold red]Validation FAILED[/bold red] "
                f"({len(result.failed)} checks failed, "
                f"{len(result.passed)} checks passed)"
            )

        if summary:
            display_summary(data, result)

    # Exit with appropriate code
    sys.exit(0 if result.is_valid else 1)


if __name__ == "__main__":
    validate()
