"""Configuration settings for scrapers."""

from pathlib import Path

# Base paths
SCRAPERS_DIR = Path(__file__).parent.parent
PROJECT_ROOT = SCRAPERS_DIR.parent

# Output paths
OUTPUT_DIR = SCRAPERS_DIR / "output"
DATA_DIR = PROJECT_ROOT / "data"
MAPPING_FILE = DATA_DIR / "team_mapping.json"

# HTTP settings
USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/120.0.0.0 Safari/537.36"
)
TIMEOUT = 30  # seconds
RATE_LIMIT_DELAY = 2.0  # seconds between requests

# Ensure output directory exists
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
