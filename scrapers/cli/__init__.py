"""CLI commands for scrapers."""

from scrapers.cli.scrape_elo import main as scrape_elo
from scrapers.cli.scrape_fifa import main as scrape_fifa
from scrapers.cli.scrape_groups import scrape_groups
from scrapers.cli.merge_data import merge_data
from scrapers.cli.validate import validate

__all__ = [
    "scrape_elo",
    "scrape_fifa",
    "scrape_groups",
    "merge_data",
    "validate",
]
