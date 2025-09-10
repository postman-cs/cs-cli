"""Email body content enhancement and fetching."""

import asyncio
from typing import List, Optional

import structlog

from ..infra.auth import GongAuthenticator
from ..infra.config import GongConfig
from ..infra.http import HTTPClientPool
from ..models import Email
from ..formatters.html_processor import HTMLProcessor

logger = structlog.get_logger()


class EmailEnhancer:
    """Enhanced email content processor with body fetching."""

    def __init__(
        self,
        http_client: HTTPClientPool,
        auth: GongAuthenticator,
        config: Optional[GongConfig] = None,
        batch_size: int = 50,
    ) -> None:
        self.http = http_client
        self.auth = auth
        self.config = config
        self.batch_size = batch_size
        
        # HTML processor for converting email bodies
        self.html_processor = HTMLProcessor()

    async def enhance_emails_with_bodies(
        self, emails: List[Email], fetch_bodies: bool = True
    ) -> List[Email]:
        """
        Enhance emails with full body content.
        
        Args:
            emails: List of emails to enhance
            fetch_bodies: Whether to fetch full email bodies
            
        Returns:
            List of enhanced emails with body content
        """
        if not emails:
            return []
            
        logger.info("Starting email enhancement", count=len(emails), fetch_bodies=fetch_bodies)

        # Filter emails that need enhancement (no body content)
        emails_to_enhance = [
            email for email in emails 
            if not email.body_text or not email.body_text.strip()
        ]

        if not emails_to_enhance:
            logger.info("No emails need body enhancement")
            return emails

        logger.info("Emails to enhance", count=len(emails_to_enhance))

        # Process in batches to control memory usage
        enhanced_emails = {}
        for batch_start in range(0, len(emails_to_enhance), self.batch_size):
            batch_end = min(batch_start + self.batch_size, len(emails_to_enhance))
            batch = emails_to_enhance[batch_start:batch_end]

            logger.debug(
                f"Processing email batch {batch_start // self.batch_size + 1}", 
                batch_size=len(batch)
            )

            # Process batch concurrently
            tasks = [
                self._enhance_single_email(email, fetch_body=fetch_bodies) 
                for email in batch
            ]
            
            results = await asyncio.gather(*tasks, return_exceptions=True)

            # Collect successful results
            for i, result in enumerate(results):
                email_id = batch[i].id
                if isinstance(result, Exception):
                    logger.error("Email enhancement failed", email_id=email_id, error=str(result))
                elif result and result.body_text:
                    enhanced_emails[email_id] = result

        # Update original emails list with enhanced data
        for i, email in enumerate(emails):
            if email.id in enhanced_emails:
                emails[i] = enhanced_emails[email.id]

        logger.info(
            "Email enhancement completed",
            enhanced=len(enhanced_emails),
            total=len(emails)
        )

        return emails

    async def _enhance_single_email(self, email: Email, *, fetch_body: bool) -> Optional[Email]:
        """Enhance a single email with full body content."""
        try:
            headers = await self.auth.get_read_headers()
            base_url = self.auth.get_base_url()

            if not fetch_body:
                return email

            # Fetch email content using the email-expanded endpoint
            endpoint = f"{base_url}/ajax/account/email-expanded"
            params = {
                "id": email.id,
                "account-id": email.account_id,
                "customer-type": "ACCOUNT",
                "workspace-id": self.config.extraction.workspace_id if self.config else "5562739194953732039",
            }

            response = await self.http.get(endpoint, params=params, headers=headers)

            if response.status_code != 200:
                # Fallback to snippet if available
                if email.snippet and email.snippet.strip():
                    email.body_text = email.snippet
                    logger.info(
                        "Using snippet fallback for email",
                        email_id=email.id,
                        status_code=response.status_code,
                    )
                    return email
                else:
                    logger.warning(
                        "Email enhancement failed - no fallback available",
                        email_id=email.id,
                        status_code=response.status_code,
                    )
                    return None

            content = response.json()

            # Process HTML content to clean text
            if fetch_body:
                html_content = content.get("body")
                if html_content:
                    # Include quoted conversation if available
                    quote = content.get("quote")
                    if quote:
                        html_content = f"{html_content}<hr/>{quote}"

                    # Convert HTML to text using processor
                    converted_text = await self.html_processor.process_content(html_content)
                    if converted_text:
                        email.body_text = converted_text

            # Update metadata if available
            if content.get("subject"):
                email.subject = content["subject"]
            if content.get("synopsis"):
                email.snippet = content["synopsis"]

            return email

        except (TimeoutError, ConnectionError, ValueError, KeyError) as e:
            logger.error(
                "Email enhancement error",
                email_id=email.id,
                error=str(e),
                error_type=type(e).__name__,
            )
            return None
