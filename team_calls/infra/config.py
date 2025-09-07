"""Minimal configuration for team calls extraction."""

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional
import structlog

logger = structlog.get_logger()


@dataclass
class HTTPConfig:
    """HTTP client pool configuration."""
    pool_size: int = 20
    max_concurrency_per_client: int = 100
    timeout_seconds: float = 30.0
    max_clients: Optional[int] = None
    global_max_concurrency: Optional[int] = None
    
    # HTTP version configuration
    enable_http3: bool = True
    force_http3: bool = False
    
    # TLS configuration for HTTP/3
    tls_version: Optional[str] = None
    impersonate_browser: str = "chrome"
    
    def __post_init__(self):
        if self.max_clients is None:
            self.max_clients = self.pool_size * self.max_concurrency_per_client
        if self.global_max_concurrency is None:
            self.global_max_concurrency = self.max_clients


@dataclass
class AuthConfig:
    """Authentication configuration."""
    csrf_token_ttl_minutes: int = 30
    csrf_token_buffer_minutes: int = 5
    retry_attempts: int = 3
    retry_backoff_base: float = 2.0
    retry_backoff_seconds: float = 1.0


@dataclass 
class GongConfig:
    """Team calls configuration."""
    
    http: HTTPConfig = field(default_factory=HTTPConfig)
    auth: AuthConfig = field(default_factory=AuthConfig)
    debug: bool = False
    
    @classmethod
    def create_default(cls) -> 'GongConfig':
        """Create default configuration."""
        return cls(
            http=HTTPConfig(
                pool_size=10,
                max_concurrency_per_client=25,
                timeout_seconds=30.0,
                enable_http3=False,
                force_http3=False,
                global_max_concurrency=150,
            ),
            auth=AuthConfig(
                csrf_token_ttl_minutes=30,
                csrf_token_buffer_minutes=5,
                retry_attempts=3,
                retry_backoff_base=2.0,
                retry_backoff_seconds=1.0
            )
        )
    
    @classmethod
    def from_env(cls) -> 'GongConfig':
        """Create configuration from environment variables."""
        config = cls.create_default()
        
        if os.getenv("GONG_DEBUG", "").lower() in ("true", "1", "yes"):
            config.debug = True
        
        if http_concurrency := os.getenv("GONG_HTTP_CONCURRENCY"):
            try:
                total_concurrency = int(http_concurrency)
                config.http.global_max_concurrency = max(1, total_concurrency)
                per_client = max(1, total_concurrency // max(1, config.http.pool_size))
                config.http.max_concurrency_per_client = per_client
            except ValueError:
                logger.warning("Invalid GONG_HTTP_CONCURRENCY value", value=http_concurrency)
        
        if config.debug:
            logger.info("Configuration loaded", 
                       debug=config.debug,
                       http_total_concurrency=config.http.pool_size * config.http.max_concurrency_per_client)
        
        return config
    
    def validate(self) -> bool:
        """Validate configuration settings."""
        if self.http.pool_size <= 0:
            raise ValueError("HTTP pool_size must be positive")
        
        if self.http.max_concurrency_per_client <= 0:
            raise ValueError("HTTP max_concurrency_per_client must be positive")
        
        return True


# Singleton instance for global access
_global_config: Optional[GongConfig] = None


def get_config() -> GongConfig:
    """Get the global configuration instance."""
    global _global_config
    if _global_config is None:
        _global_config = GongConfig.from_env()
        _global_config.validate()
    return _global_config


def set_config(config: GongConfig) -> None:
    """Set the global configuration instance."""
    global _global_config
    config.validate()
    _global_config = config
    if config.debug:
        logger.info("Global configuration updated")


def reset_config() -> None:
    """Reset the global configuration (mainly for testing)."""
    global _global_config
    _global_config = None