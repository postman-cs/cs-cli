"""CS Transcript CLI - Team Calls Extractor.

A standalone tool for extracting team calls from Gong and saving them as markdown transcripts.
"""

__version__ = "1.0.0"
__author__ = "Customer Success Team"
__description__ = "Extract team calls from Gong and save as markdown transcripts"

# Lazy imports to avoid dependency issues during package discovery
def __getattr__(name):
    if name == "cli":
        from .cli import cli
        return cli
    elif name == "main":
        from .cli import main
        return main
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")

__all__ = ["cli", "main", "__version__"]