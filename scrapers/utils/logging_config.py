"""Logging configuration with rich console output."""

import logging
from typing import Optional

from rich.console import Console
from rich.logging import RichHandler


def setup_logging(
    level: int = logging.INFO,
    log_file: Optional[str] = None,
) -> None:
    """Configure logging with rich console output.

    Args:
        level: Logging level (default: INFO).
        log_file: Optional path to log file.
    """
    console = Console(stderr=True)

    handlers: list[logging.Handler] = [
        RichHandler(
            console=console,
            show_time=True,
            show_path=False,
            markup=True,
            rich_tracebacks=True,
        )
    ]

    if log_file:
        file_handler = logging.FileHandler(log_file)
        file_handler.setFormatter(
            logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
        )
        handlers.append(file_handler)

    logging.basicConfig(
        level=level,
        format="%(message)s",
        datefmt="[%X]",
        handlers=handlers,
    )

    # Suppress noisy loggers
    logging.getLogger("urllib3").setLevel(logging.WARNING)
    logging.getLogger("requests").setLevel(logging.WARNING)
