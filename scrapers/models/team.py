"""Pydantic models matching the Rust wc-core schema."""

from enum import Enum
from typing import Annotated, Literal

from pydantic import BaseModel, Field, field_validator


class Confederation(str, Enum):
    """FIFA confederation - matches Rust Confederation enum."""

    UEFA = "UEFA"
    CONMEBOL = "CONMEBOL"
    CONCACAF = "CONCACAF"
    CAF = "CAF"
    AFC = "AFC"
    OFC = "OFC"


class Team(BaseModel):
    """A national team with all relevant statistics.

    Matches the Rust Team struct in wc-core/src/team.rs.
    """

    id: Annotated[int, Field(ge=0, le=47, description="Unique identifier (0-47)")]
    name: Annotated[str, Field(min_length=1, description="Full team name")]
    code: Annotated[str, Field(pattern=r"^[A-Z]{3}$", description="FIFA country code")]
    confederation: Confederation
    elo_rating: Annotated[float, Field(gt=0, description="World Football ELO rating")]
    market_value_millions: Annotated[
        float, Field(ge=0, description="Squad market value in millions of euros")
    ]
    fifa_ranking: Annotated[
        int, Field(ge=1, le=211, description="FIFA world ranking position")
    ]
    world_cup_wins: Annotated[
        int, Field(ge=0, description="Number of World Cup titles won")
    ]

    class Config:
        use_enum_values = True


class GroupId(BaseModel):
    """Group identifier wrapper - matches Rust GroupId(char)."""

    value: Annotated[
        str,
        Field(pattern=r"^[A-L]$", description="Group letter A-L"),
    ]

    def __str__(self) -> str:
        return self.value


class Group(BaseModel):
    """A group of 4 teams - matches Rust Group struct."""

    id: Annotated[str, Field(pattern=r"^[A-L]$", description="Group identifier A-L")]
    teams: Annotated[
        list[int], Field(min_length=4, max_length=4, description="4 team IDs")
    ]

    @field_validator("teams")
    @classmethod
    def validate_team_ids(cls, v: list[int]) -> list[int]:
        """Validate that all team IDs are in valid range."""
        for team_id in v:
            if not 0 <= team_id <= 47:
                raise ValueError(f"Team ID {team_id} must be between 0 and 47")
        return v


class TournamentData(BaseModel):
    """Complete tournament data with 48 teams and 12 groups.

    This is the top-level structure expected by the Rust code.
    """

    teams: Annotated[
        list[Team],
        Field(min_length=48, max_length=48, description="All 48 qualified teams"),
    ]
    groups: Annotated[
        list[Group],
        Field(min_length=12, max_length=12, description="All 12 groups"),
    ]

    @field_validator("groups")
    @classmethod
    def validate_groups(cls, v: list[Group]) -> list[Group]:
        """Validate group letters are A-L in order."""
        expected_ids = [chr(ord("A") + i) for i in range(12)]
        actual_ids = [g.id for g in v]
        if actual_ids != expected_ids:
            raise ValueError(f"Groups must be A-L in order, got {actual_ids}")
        return v

    @field_validator("teams")
    @classmethod
    def validate_unique_team_ids(cls, v: list[Team]) -> list[Team]:
        """Validate that all team IDs are unique."""
        ids = [t.id for t in v]
        if len(ids) != len(set(ids)):
            raise ValueError("Team IDs must be unique")
        return v
