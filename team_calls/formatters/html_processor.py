"""HTML to text processing for email content."""

import re
from abc import ABC, abstractmethod

import structlog
from lxml import html

logger = structlog.get_logger()


class ContentProcessor(ABC):
    """Abstract base class for content processing."""

    @abstractmethod
    async def process_content(self, content: str) -> str:
        """Process raw content into cleaned text."""
        pass


class HTMLProcessor(ContentProcessor):
    """Optimized HTML to text processor using lxml."""

    def __init__(self):
        """Initialize HTML processor."""
        # Precompiled regex patterns for fallback
        self._script_style_pattern = re.compile(
            r"<(script|style)[^>]*>.*?</\1>", re.DOTALL | re.IGNORECASE
        )
        self._tag_pattern = re.compile(r"<[^>]+>")
        self._whitespace_pattern = re.compile(r"\s+")

    async def process_content(self, html_content: str) -> str:
        """Convert HTML to clean text using lxml with regex fallback."""
        if not html_content or not html_content.strip():
            return ""

        try:
            # Primary: Use lxml for proper HTML parsing
            doc = html.fromstring(html_content)
            text = doc.text_content()
            # Clean up whitespace
            text = self._whitespace_pattern.sub(" ", text).strip()
            return text
        except (ValueError, LookupError) as e:
            logger.debug("lxml parsing failed, using regex fallback", error=str(e))
            # Fallback: Use regex-based conversion
            return self._regex_html_to_text(html_content)

    def _regex_html_to_text(self, html_content: str) -> str:
        """Fallback HTML to text conversion using regex."""
        # Remove script and style elements
        text = self._script_style_pattern.sub("", html_content)
        # Remove HTML tags
        text = self._tag_pattern.sub(" ", text)
        # Clean up whitespace
        text = self._whitespace_pattern.sub(" ", text).strip()
        return text
