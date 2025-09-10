"""HTTP client with TLS fingerprinting using curl_cffi.

Includes centralized flow control and safer retry/backoff.
"""

import asyncio
import random
from email.utils import parsedate_to_datetime
from typing import Any, Dict, Optional, Union

import structlog
from curl_cffi import AsyncSession
from curl_cffi.const import CurlHttpVersion
from pydantic import BaseModel

from .config import HTTPConfig, get_config

logger = structlog.get_logger()


class GongHTTPClient:
    """High-performance HTTP client."""
    
    def __init__(self, config: Optional[HTTPConfig] = None) -> None:
        self.config = config or get_config().http
        self.session: Optional[AsyncSession] = None
        self.semaphore = asyncio.Semaphore(self.config.max_concurrency_per_client)
        self.cookies: Dict[str, str] = {}
        # Let curl-cffi handle most headers via Firefox impersonation
        # Only set minimal custom headers to avoid conflicts
        self.headers: Dict[str, str] = {}
    
    async def __aenter__(self) -> "GongHTTPClient":
        """Async context manager entry."""
        await self.start()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Async context manager exit."""
        await self.close()
    
    async def start(self) -> None:
        """Initialize the HTTP session."""
        if self.session is not None:
            return
        
        # Determine HTTP version based on config
        if self.config.force_http3:
            http_version = 31  # CurlHttpVersion.V3ONLY - HTTP/3 only, no fallback
            version_desc = "HTTP/3 only"
        elif self.config.enable_http3:
            http_version = CurlHttpVersion.V3  # HTTP/3 with fallback to HTTP/2
            version_desc = "HTTP/3 with fallback"
        else:
            http_version = 2  # HTTP/2
            version_desc = "HTTP/2"
        
        self.session = AsyncSession(
            # Browser impersonation from config (default: chrome)
            impersonate=self.config.impersonate_browser,
            # HTTP version based on configuration
            http_version=http_version,
            # Connection pooling for efficiency - optimized distribution pattern
            max_clients=self.config.max_concurrency_per_client,
            # Timeout configuration from config
            timeout=self.config.timeout_seconds,
        )
        
        logger.info("HTTP client initialized", 
                   concurrency=self.config.max_concurrency_per_client,
                   timeout=self.config.timeout_seconds,
                   http_version=version_desc,
                   impersonate=self.config.impersonate_browser)
    
    async def close(self) -> None:
        """Close the HTTP session."""
        if self.session is not None:
            await self.session.close()
            self.session = None
    
    async def set_cookies(self, cookies: Dict[str, str], domain: str = ".gong.io") -> None:
        """Set cookies for all requests."""
        self.cookies.update(cookies)
        if self.session:
            # Update session cookies with proper domain
            for name, value in cookies.items():
                self.session.cookies.set(name, value, domain=domain)
        logger.debug("Cookies updated", count=len(cookies), domain=domain)
    
    async def update_headers(self, headers: Dict[str, str]) -> None:
        """Update default headers."""
        self.headers.update(headers)
        logger.debug("Headers updated", new_headers=list(headers.keys()))
    
    async def get(self, url: str, **kwargs) -> Any:
        """Perform GET request with concurrency control."""
        async with self.semaphore:
            return await self._request("GET", url, **kwargs)
    
    async def post(self, url: str, **kwargs) -> Any:
        """Perform POST request with concurrency control."""
        async with self.semaphore:
            return await self._request("POST", url, **kwargs)
    
    async def _request(self, method: str, url: str, **kwargs) -> Any:
        """Internal request method with retry logic."""
        if not self.session:
            await self.start()
        
        # Merge headers
        request_headers = self.headers.copy()
        if "headers" in kwargs:
            request_headers.update(kwargs["headers"])
        kwargs["headers"] = request_headers
        
        # Add cookies if not explicitly provided
        if "cookies" not in kwargs and self.cookies:
            kwargs["cookies"] = self.cookies
        
        max_retries = 3
        last_exception = None
        
        for attempt in range(max_retries):
            try:
                response = await self.session.request(method, url, **kwargs)
                
                # Log rate limiting or errors
                if response.status_code == 429:
                    # Honor Retry-After if present; otherwise exponential backoff with jitter
                    retry_after = None
                    try:
                        retry_after = response.headers.get("Retry-After")
                    except Exception:
                        retry_after = None
                    sleep_seconds = None
                    if retry_after:
                        try:
                            sleep_seconds = int(retry_after)
                        except ValueError:
                            try:
                                dt = parsedate_to_datetime(retry_after)
                                sleep_seconds = max(0, (dt - dt.now(dt.tzinfo)).total_seconds())
                            except Exception:
                                sleep_seconds = None
                    if sleep_seconds is None:
                        base = 2 ** attempt
                        sleep_seconds = base + random.uniform(0, 0.5 * base)
                    logger.warning("Rate limited", url=url, attempt=attempt, retry_after=sleep_seconds)
                    await asyncio.sleep(sleep_seconds)
                    continue
                elif response.status_code >= 500:
                    # Retry 5xx errors with jittered backoff
                    logger.warning("Server error - will retry", 
                                 url=url, 
                                 status=response.status_code,
                                 attempt=attempt)
                    if attempt < max_retries - 1:
                        base = 2 ** attempt
                        await asyncio.sleep(base + random.uniform(0, 0.5 * base))
                        continue
                elif response.status_code >= 400:
                    logger.warning("Client error - not retrying", 
                                 url=url, 
                                 status=response.status_code,
                                 attempt=attempt)
                
                return response
                
            except Exception as e:
                last_exception = e
                logger.error("Request failed", 
                           url=url, 
                           method=method,
                           attempt=attempt,
                           error=str(e))
                
                if attempt < max_retries - 1:
                    base = 2 ** attempt
                    await asyncio.sleep(base + random.uniform(0, 0.5 * base))
                else:
                    raise last_exception
        
        raise last_exception or RuntimeError("Request failed after retries")
    
    async def batch_get(self, urls: list[str], **kwargs) -> list[Any]:
        """Perform multiple GET requests concurrently."""
        tasks = [self.get(url, **kwargs) for url in urls]
        return await asyncio.gather(*tasks, return_exceptions=True)
    
    async def batch_post(self, requests: list[tuple[str, dict]], **kwargs) -> list[Any]:
        """Perform multiple POST requests concurrently."""
        tasks = []
        for url, data in requests:
            task_kwargs = kwargs.copy()
            if isinstance(data, dict):
                task_kwargs["json"] = data
            else:
                task_kwargs["data"] = data
            tasks.append(self.post(url, **task_kwargs))
        
        return await asyncio.gather(*tasks, return_exceptions=True)


class HTTPClientPool:
    """Pool of HTTP clients for maximum concurrency."""
    
    def __init__(self, pool_size: int = None, max_concurrency_per_client: int = None, 
                 config: Optional[HTTPConfig] = None) -> None:
        self.config = config or get_config().http
        self.pool_size = pool_size or self.config.pool_size
        self.max_concurrency_per_client = max_concurrency_per_client or self.config.max_concurrency_per_client
        self.clients: list[GongHTTPClient] = []
        self.current_client = 0
        self._global_semaphore: Optional[asyncio.Semaphore] = None
    
    async def __aenter__(self) -> "HTTPClientPool":
        """Initialize client pool."""
        for i in range(self.pool_size):
            client = GongHTTPClient(config=self.config)
            await client.start()
            self.clients.append(client)
        # Global cap across pool
        total_cap = getattr(self.config, "global_max_concurrency", None)
        if total_cap and total_cap > 0:
            self._global_semaphore = asyncio.Semaphore(total_cap)
        
        logger.info("HTTP client pool initialized", 
                   pool_size=self.pool_size,
                   concurrency_per_client=self.max_concurrency_per_client,
                   total_concurrency=self.pool_size * self.max_concurrency_per_client,
                   global_cap=total_cap)
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Close all clients."""
        for client in self.clients:
            await client.close()
    
    async def set_cookies(self, cookies: Dict[str, str], domain: str = ".gong.io") -> None:
        """Set cookies on all clients."""
        for client in self.clients:
            await client.set_cookies(cookies, domain=domain)
    
    async def update_headers(self, headers: Dict[str, str]) -> None:
        """Update headers on all clients."""
        for client in self.clients:
            await client.update_headers(headers)
    
    def get_client(self) -> GongHTTPClient:
        """Get next client using round-robin."""
        client = self.clients[self.current_client]
        self.current_client = (self.current_client + 1) % self.pool_size
        return client
    
    async def get(self, url: str, **kwargs) -> Any:
        """Get request using round-robin client selection (global-capped)."""
        client = self.get_client()
        if self._global_semaphore is None:
            return await client.get(url, **kwargs)
        async with self._global_semaphore:
            return await client.get(url, **kwargs)
    
    async def post(self, url: str, **kwargs) -> Any:
        """Post request using round-robin client selection (global-capped)."""
        client = self.get_client()
        if self._global_semaphore is None:
            return await client.post(url, **kwargs)
        async with self._global_semaphore:
            return await client.post(url, **kwargs)
    
    async def batch_requests(self, 
                           requests: list[tuple[str, str, dict]], 
                           **kwargs) -> list[Any]:
        """Distribute batch requests across all clients."""
        # Group requests by client
        client_requests = [[] for _ in range(self.pool_size)]
        
        for i, (method, url, request_kwargs) in enumerate(requests):
            client_idx = i % self.pool_size
            client_requests[client_idx].append((method, url, request_kwargs))
        
        # Execute on each client
        tasks = []
        for client_idx, client_reqs in enumerate(client_requests):
            if not client_reqs:
                continue
            
            client = self.clients[client_idx]
            client_tasks = []
            
            for method, url, request_kwargs in client_reqs:
                merged_kwargs = {**kwargs, **request_kwargs}
                if method.upper() == "GET":
                    client_tasks.append(client.get(url, **merged_kwargs))
                elif method.upper() == "POST":
                    client_tasks.append(client.post(url, **merged_kwargs))
            
            if client_tasks:
                tasks.extend(client_tasks)
        
        if not tasks:
            return []
        if self._global_semaphore is None:
            return await asyncio.gather(*tasks, return_exceptions=True)
        async def _guard(coro):
            async with self._global_semaphore:
                return await coro
        return await asyncio.gather(*[_guard(t) for t in tasks], return_exceptions=True)