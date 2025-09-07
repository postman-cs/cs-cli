"""Team calls infrastructure modules."""

# Use lazy imports to avoid dependency issues during package discovery
__all__ = [
    "GongAuthenticator", 
    "BrowserCookieExtractor", 
    "CSRFManager", 
    "GongCookies",
    "GongConfig", 
    "HTTPConfig", 
    "AuthConfig", 
    "get_config", 
    "set_config", 
    "reset_config",
    "HTTPClientPool", 
    "GongHTTPClient"
]

def __getattr__(name):
    if name in ["GongAuthenticator", "BrowserCookieExtractor", "CSRFManager", "GongCookies"]:
        from .auth import GongAuthenticator, BrowserCookieExtractor, CSRFManager, GongCookies
        return locals()[name]
    elif name in ["GongConfig", "HTTPConfig", "AuthConfig", "get_config", "set_config", "reset_config"]:
        from .config import GongConfig, HTTPConfig, AuthConfig, get_config, set_config, reset_config
        return locals()[name]
    elif name in ["HTTPClientPool", "GongHTTPClient"]:
        from .http import HTTPClientPool, GongHTTPClient
        return locals()[name]
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")