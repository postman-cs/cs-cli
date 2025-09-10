"""Firefox cookie extraction for Gong authentication."""

import asyncio
import base64
import json
import re
import time
from typing import Dict, Optional

import browser_cookie3
import structlog
from pydantic import BaseModel

from .config import AuthConfig, get_config

logger = structlog.get_logger()


class GongCookies(BaseModel):
    """Gong authentication cookies."""
    cell: str
    session_cookies: Dict[str, str]
    extracted_at: float
    browser: str  # Track which browser was used


class FirefoxCookieExtractor:
    """Extract cookies from Firefox using browser_cookie3."""
    
    def __init__(self) -> None:
        pass
    
    async def extract_gong_cookies(self) -> Optional[GongCookies]:
        """Extract Gong cookies from Firefox."""
        try:
            logger.info("Extracting Gong cookies from Firefox")
            
            # Extract cookies from Firefox for Gong domains
            cookies_dict = {}
            cookie_domains = set()
            
            # Get all Firefox cookies, then filter for Gong domains
            all_cookies = browser_cookie3.firefox()
            for cookie in all_cookies:
                # Include cookies from any gong.io domain
                if 'gong.io' in cookie.domain.lower():
                    cookies_dict[cookie.name] = cookie.value
                    cookie_domains.add(cookie.domain)
            
            logger.debug(f"Found {len(cookies_dict)} cookies in Firefox")
            logger.debug(f"Cookie domains: {list(cookie_domains)}")
            
            if not cookies_dict:
                logger.error("No .gong.io cookies found in Firefox")
                return None
            
            cell_value = None
            
            # Method 1: Try to find and decode "cell" cookie (JWT)
            if "cell" in cookies_dict:
                cell_value = self._decode_cell_cookie(cookies_dict["cell"])
                logger.debug(f"Extracted cell from JWT cookie: {cell_value}")
            
            # Method 2: If no cell cookie, extract from domain names
            if not cell_value:
                cell_value = self._extract_cell_from_domains(cookie_domains)
                logger.debug(f"Extracted cell from domain names: {cell_value}")
            
            if not cell_value:
                logger.error("Could not determine cell value from Firefox cookies")
                return None
            
            logger.info("Successfully extracted Gong cookies from Firefox", 
                       cell=cell_value, 
                       cookies_count=len(cookies_dict))
            
            return GongCookies(
                cell=cell_value,
                session_cookies=cookies_dict,
                extracted_at=time.time(),
                browser="firefox"
            )
            
        except Exception as e:
            logger.error(f"Failed to extract cookies from Firefox: {str(e)}")
            logger.info("Make sure you're logged into Gong in Firefox")
            return None
    
    def _decode_cell_cookie(self, cell_cookie: str) -> Optional[str]:
        """Decode JWT cell cookie to extract cell value."""
        try:
            # JWT format: header.payload.signature
            parts = cell_cookie.split(".")
            if len(parts) != 3:
                return None
            
            # Decode the payload (middle part)
            payload = parts[1]
            # Add padding if needed for base64 decoding
            padding = 4 - (len(payload) % 4)
            if padding != 4:
                payload += "=" * padding
            
            decoded_bytes = base64.urlsafe_b64decode(payload)
            payload_data = json.loads(decoded_bytes.decode("utf-8"))
            
            return payload_data.get("cell")
            
        except Exception as e:
            logger.debug("Failed to decode cell cookie", error=str(e))
            return None
    
    def _extract_cell_from_domains(self, domains: set) -> Optional[str]:
        """Extract cell value from cookie domain names."""
        try:
            # Search in any domain containing "gong" for cell information
            for domain in domains:
                if "gong" not in domain.lower():
                    continue
                    
                logger.debug(f"Analyzing domain for cell: {domain}")
                
                # Pattern 1: Direct cell domains like us-14496.app.gong.io
                if ".app.gong" in domain and not domain.startswith("resource."):
                    # Extract the cell part (everything before .app.gong...)
                    cell_part = domain.split(".app.gong")[0].replace(".", "")
                    # Skip generic prefixes
                    if (cell_part and 
                        not cell_part.startswith("resource") and 
                        not cell_part.startswith("gcell") and
                        cell_part not in ["app", "www", "api"]):
                        logger.debug(f"Found cell from domain {domain}: {cell_part}")
                        return cell_part
                
                # Pattern 2: gcell patterns like resource.gcell-nam-01.app.gong.io
                if "gcell-" in domain:
                    parts = domain.split(".")
                    for part in parts:
                        if part.startswith("gcell-"):
                            cell_value = part.replace("gcell-", "")
                            if cell_value:
                                logger.debug(f"Found gcell from domain {domain}: {cell_value}")
                                return cell_value
                
                # Pattern 3: Cell in any part of gong domain (e.g., cell-123.something.gong.com)
                parts = domain.split(".")
                for part in parts:
                    # Look for cell-like patterns (alphanumeric with dashes)
                    if (len(part) > 3 and 
                        any(c.isdigit() for c in part) and 
                        any(c.isalpha() for c in part) and
                        part not in ["gong", "app", "www", "api", "resource"]):
                        logger.debug(f"Found potential cell from domain {domain}: {part}")
                        return part
                            
        except Exception as e:
            logger.debug("Failed to extract cell from domains", error=str(e))
            
        return None


class CSRFManager:
    """Manage CSRF tokens for Gong API."""
    
    def __init__(self, http_client, config: Optional[AuthConfig] = None) -> None:
        self.http_client = http_client
        self.config = config or get_config().auth
        self.csrf_token: Optional[str] = None
        self.token_expires_at: float = 0.0
        self.token_lock = asyncio.Lock()
        self.refresh_attempts = 0
        self.max_refresh_attempts = self.config.retry_attempts
    
    async def get_csrf_token(self, cell: str, force_refresh: bool = False) -> Optional[str]:
        """Get valid CSRF token, refreshing if needed.
        
        Args:
            cell: Gong cell identifier
            force_refresh: Force a token refresh regardless of expiry
        """
        async with self.token_lock:
            # Force refresh if requested or token is expired/missing
            if force_refresh or not self._is_token_valid():
                await self._refresh_csrf_token(cell)
            
            return self.csrf_token
    
    def _is_token_valid(self) -> bool:
        """Check if current token is still valid with configurable buffer."""
        buffer_seconds = self.config.csrf_token_buffer_minutes * 60
        return (
            self.csrf_token is not None and 
            time.time() < (self.token_expires_at - buffer_seconds)
        )
    
    def invalidate_token(self) -> None:
        """Invalidate current token to force refresh on next request."""
        self.csrf_token = None
        self.token_expires_at = 0.0
        logger.debug("CSRF token invalidated")
    
    async def _refresh_csrf_token(self, cell: str) -> None:
        """Fetch new CSRF token from Gong API with retry logic."""
        url = f"https://{cell}.app.gong.io/ajax/common/rtkn"
        
        # CSRF token requests need proper AJAX headers
        headers = {
            "X-Requested-With": "XMLHttpRequest",
            "Referer": f"https://{cell}.app.gong.io/",
            "Accept": "application/json, text/javascript, */*; q=0.01"
        }
        
        for attempt in range(self.max_refresh_attempts):
            try:
                response = await self.http_client.get(url, headers=headers)
                if response.status_code == 200:
                    data = response.json()
                    self.csrf_token = data.get("token")  # Fixed: use 'token' not 'requestToken'
                    # Token expires based on config
                    token_ttl_seconds = self.config.csrf_token_ttl_minutes * 60
                    self.token_expires_at = time.time() + token_ttl_seconds
                    self.refresh_attempts = 0
                    logger.info("CSRF token refreshed", 
                               cell=cell,
                               expires_in_minutes=self.config.csrf_token_ttl_minutes,
                               token_length=len(self.csrf_token) if self.csrf_token else 0)
                    return
                else:
                    logger.error("Failed to fetch CSRF token", 
                               status=response.status_code,
                               attempt=attempt + 1)
                    
            except Exception as e:
                logger.error("CSRF token fetch failed", 
                           error=str(e),
                           attempt=attempt + 1)
            
            # Wait before retry with exponential backoff
            if attempt < self.max_refresh_attempts - 1:
                backoff_delay = self.config.retry_backoff_seconds * (self.config.retry_backoff_base ** attempt)
                await asyncio.sleep(backoff_delay)
        
        # All attempts failed
        self.csrf_token = None
        self.token_expires_at = 0.0
        logger.error("CSRF token refresh failed after all attempts")


class GongAuthenticator:
    """Main authentication manager for Gong API with Firefox support."""
    
    def __init__(self, http_client, config: Optional[AuthConfig] = None) -> None:
        self.http_client = http_client
        self.config = config or get_config().auth
        self.extractor = FirefoxCookieExtractor()
        self.csrf_manager = CSRFManager(http_client, self.config)
        self.gong_cookies: Optional[GongCookies] = None
        self.base_url: Optional[str] = None
        self.workspace_id: Optional[str] = None
    
    async def authenticate(self) -> bool:
        """Perform complete authentication flow."""
        logger.info("Starting Gong authentication")
        
        # Extract cookies from Firefox
        self.gong_cookies = await self.extractor.extract_gong_cookies()
        if not self.gong_cookies:
            logger.error("Authentication failed: No valid Gong cookies found in Firefox")
            logger.info("Make sure you're logged into Gong in Firefox")
            return False
        
        self.base_url = f"https://{self.gong_cookies.cell}.app.gong.io"
        
        # Set cookies for all domains where they were originally found
        # This ensures cell-specific authentication cookies are available
        domains_to_set = {".gong.io", ".app.gong.io", f".{self.gong_cookies.cell}.app.gong.io"}
        
        for domain in domains_to_set:
            await self.http_client.set_cookies(self.gong_cookies.session_cookies, domain=domain)
            logger.debug(f"Set {len(self.gong_cookies.session_cookies)} cookies for domain: {domain}")
        
        # Get initial CSRF token
        csrf_token = await self.csrf_manager.get_csrf_token(self.gong_cookies.cell)
        if not csrf_token:
            return False
        
        # Extract workspace ID from home page
        self.workspace_id = await self.get_workspace_id()
        if self.workspace_id:
            logger.info("Extracted workspace ID", workspace_id=self.workspace_id)
        
        logger.info("Authentication successful", 
                   cell=self.gong_cookies.cell,
                   browser="firefox",
                   workspace_id=self.workspace_id)
        return True
    
    async def get_read_headers(self) -> Dict[str, str]:
        """Get headers for read-only requests (GET) - no CSRF token needed.
        
        Returns:
            Headers for GET requests without CSRF token
        """
        if not self.gong_cookies:
            raise RuntimeError("Not authenticated")
        
        return {
            "X-Requested-With": "XMLHttpRequest",
            "Referer": self.base_url,
        }
    
    async def get_authenticated_headers(self, retry_on_failure: bool = True) -> Dict[str, str]:
        """Get headers for state-changing requests (POST/PUT/DELETE) with CSRF token.
        
        Args:
            retry_on_failure: Whether to retry with fresh token on initial failure
        """
        if not self.gong_cookies:
            raise RuntimeError("Not authenticated")
        
        csrf_token = await self.csrf_manager.get_csrf_token(self.gong_cookies.cell)
        if not csrf_token:
            if retry_on_failure:
                # Try once more with forced refresh
                logger.warning("CSRF token missing, attempting forced refresh")
                csrf_token = await self.csrf_manager.get_csrf_token(
                    self.gong_cookies.cell, 
                    force_refresh=True
                )
            
            if not csrf_token:
                raise RuntimeError("Failed to get CSRF token after refresh attempt")
        
        return {
            "X-Requested-With": "XMLHttpRequest",
            "X-CSRF-Token": csrf_token,
            "Referer": self.base_url,
        }
    
    async def handle_auth_error(self, status_code: int, is_post_request: bool = True) -> bool:
        """Handle authentication errors by refreshing tokens.
        
        Args:
            status_code: HTTP status code that triggered the error
            is_post_request: Whether this was a POST/PUT/DELETE request needing CSRF tokens
            
        Returns:
            True if tokens were refreshed successfully
        """
        if status_code in [401, 403]:
            if is_post_request:
                logger.warning("Authentication error on POST request, refreshing CSRF token",
                              status=status_code)
                
                # Invalidate current token
                self.csrf_manager.invalidate_token()
                
                # Try to get fresh token
                if self.gong_cookies:
                    csrf_token = await self.csrf_manager.get_csrf_token(
                        self.gong_cookies.cell,
                        force_refresh=True
                    )
                    return csrf_token is not None
            else:
                logger.warning("Authentication error on GET request - likely session issue",
                              status=status_code)
                # For GET requests, 401/403 likely means session cookies are invalid
                # We could potentially refresh the entire authentication here
                return False
        
        return False
    
    def get_base_url(self) -> str:
        """Get the base URL for API calls."""
        if not self.base_url:
            raise RuntimeError("Not authenticated")
        return self.base_url
    
    async def get_workspace_id(self) -> Optional[str]:
        """Extract workspace ID from Gong home page."""
        if not self.base_url:
            logger.warning("Cannot get workspace ID without base URL")
            return None
        
        try:
            # Fetch the home page
            url = f"{self.base_url}/home"
            headers = await self.get_read_headers()
            response = await self.http_client.get(url, headers=headers)
            
            if response.status_code != 200:
                logger.warning("Failed to fetch home page for workspace ID", status=response.status_code)
                return None
            
            # Parse the HTML to find the pageContext script
            html_content = response.text
            
            # Look for the pageContext JavaScript object
            pattern = r'workspaceId:\s*"(\d+)"'
            match = re.search(pattern, html_content)
            
            if match:
                workspace_id = match.group(1)
                return workspace_id
            else:
                # Try alternative pattern
                pattern = r'"workspaceId"\s*:\s*"(\d+)"'
                match = re.search(pattern, html_content)
                if match:
                    workspace_id = match.group(1)
                    return workspace_id
                
                logger.warning("Could not find workspace ID in home page")
                return None
                
        except Exception as e:
            logger.error(f"Error extracting workspace ID: {e}")
            return None
    
    def get_workspace_id_sync(self) -> Optional[str]:
        """Get the cached workspace ID."""
        return self.workspace_id