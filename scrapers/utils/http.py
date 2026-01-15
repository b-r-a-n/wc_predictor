"""HTTP utilities for scrapers."""

import time
from functools import wraps
from typing import Any, Callable

import requests

from scrapers.config.settings import USER_AGENT, TIMEOUT, RATE_LIMIT_DELAY


def create_session() -> requests.Session:
    """Create a requests Session with default headers.

    Returns:
        A configured requests.Session instance.
    """
    session = requests.Session()
    session.headers.update(
        {
            "User-Agent": USER_AGENT,
            "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            "Accept-Language": "en-US,en;q=0.5",
            "Accept-Encoding": "gzip, deflate, br",
            "Connection": "keep-alive",
            "Upgrade-Insecure-Requests": "1",
        }
    )
    return session


class RateLimiter:
    """Simple rate limiter for HTTP requests."""

    def __init__(self, delay: float = RATE_LIMIT_DELAY) -> None:
        """Initialize the rate limiter.

        Args:
            delay: Minimum seconds between requests.
        """
        self.delay = delay
        self._last_request_time: float = 0.0

    def wait(self) -> None:
        """Wait if necessary to respect rate limit."""
        now = time.time()
        elapsed = now - self._last_request_time
        if elapsed < self.delay:
            time.sleep(self.delay - elapsed)
        self._last_request_time = time.time()

    def __call__(self, func: Callable[..., Any]) -> Callable[..., Any]:
        """Decorator to rate limit a function.

        Args:
            func: Function to rate limit.

        Returns:
            Wrapped function with rate limiting.
        """

        @wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            self.wait()
            return func(*args, **kwargs)

        return wrapper


# Global rate limiter instance
rate_limiter = RateLimiter()


def rate_limited(func: Callable[..., Any]) -> Callable[..., Any]:
    """Decorator to apply global rate limiting to a function.

    Args:
        func: Function to rate limit.

    Returns:
        Wrapped function with rate limiting.
    """
    return rate_limiter(func)
