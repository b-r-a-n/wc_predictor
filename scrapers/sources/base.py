"""Base scraper abstract class."""

import json
import logging
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Any

import requests

from scrapers.config.settings import USER_AGENT, TIMEOUT
from scrapers.utils.http import create_session


class ScraperError(Exception):
    """Exception raised when scraping fails."""

    pass


class BaseScraper(ABC):
    """Abstract base class for all scrapers."""

    def __init__(self, output_dir: Path) -> None:
        """Initialize the scraper.

        Args:
            output_dir: Directory where output files will be saved.
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.session = create_session()
        self.logger = logging.getLogger(self.__class__.__name__)

    @abstractmethod
    def scrape(self) -> Any:
        """Execute the scraping operation.

        Returns:
            The scraped data in the appropriate format.

        Raises:
            ScraperError: If scraping fails.
        """
        pass

    @abstractmethod
    def get_output_filename(self) -> str:
        """Get the output filename for this scraper.

        Returns:
            The filename (without path) for the output file.
        """
        pass

    def save(self, data: Any) -> Path:
        """Save data to a JSON file.

        Args:
            data: Data to save. Can be a dict, list, or Pydantic model.

        Returns:
            Path to the saved file.
        """
        output_path = self.output_dir / self.get_output_filename()

        # Handle Pydantic models
        if hasattr(data, "model_dump"):
            data = data.model_dump(mode="json")

        with open(output_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

        self.logger.info(f"Saved data to {output_path}")
        return output_path

    def fail_fast(self, message: str) -> None:
        """Raise a ScraperError with the given message.

        Args:
            message: Error message describing the failure.

        Raises:
            ScraperError: Always raised with the given message.
        """
        self.logger.error(message)
        raise ScraperError(message)
