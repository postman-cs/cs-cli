"""API clients for team calls extraction."""

# Use lazy imports to avoid dependency issues during package discovery
__all__ = [
    "GongLibraryClient",
    "GongCustomerSearchClient", 
    "CallDetailsClient"
]

def __getattr__(name):
    if name in ["GongLibraryClient"]:
        from .library_client import GongLibraryClient
        return GongLibraryClient
    elif name in ["GongCustomerSearchClient"]:
        from .customer_search import GongCustomerSearchClient
        return GongCustomerSearchClient
    elif name in ["CallDetailsClient"]:
        from .call_details import CallDetailsClient
        return CallDetailsClient
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")