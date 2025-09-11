#!/usr/bin/env python3
"""CLI for team calls extraction."""

import asyncio
import logging
import sys
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Tuple

import click
import structlog
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TimeElapsedColumn
from rich.prompt import Prompt
from rich.table import Table

from .infra.auth import GongAuthenticator
from .infra.config import GongConfig
from .infra.http import HTTPClientPool

from .api.library_client import GongLibraryClient, CallDetailsFetcher
from .api.customer_search import GongCustomerSearchClient
from .api.timeline_extractor import TimelineExtractor
from .api.email_enhancer import EmailEnhancer
from .formatters.markdown import CallMarkdownFormatter, CallSummaryReporter
from .models import Email

# Setup logging and console
console = Console()
logger = structlog.get_logger()

# Configure quiet logging by default
logging.basicConfig(level=logging.WARNING)
structlog.configure(
    wrapper_class=structlog.make_filtering_bound_logger(logging.WARNING),
)


class TeamCallsExtractor:
    """Main orchestrator for team calls extraction."""
    
    def __init__(self, config: GongConfig):
        self.config = config
        self.http: HTTPClientPool = None
        self.auth: GongAuthenticator = None
        self.library_client: GongLibraryClient = None
        self.details_fetcher: CallDetailsFetcher = None
        self.customer_search_client: GongCustomerSearchClient = None
        self.timeline_extractor: TimelineExtractor = None
        self.email_enhancer: EmailEnhancer = None
        self.formatter = CallMarkdownFormatter()
        self.summary_reporter = CallSummaryReporter()
    
    async def setup(self) -> None:
        """Initialize all components."""
        console.print("[yellow]Setting up team calls extractor...[/yellow]")
        
        # Initialize HTTP client pool (using async context manager pattern)
        self.http = HTTPClientPool(config=self.config.http)
        await self.http.__aenter__()
        
        # Initialize authenticator
        self.auth = GongAuthenticator(self.http, config=self.config.auth)
        await self.auth.authenticate()
        
        # Initialize API clients
        self.library_client = GongLibraryClient(
            http_client=self.http,
            auth=self.auth,
            config=self.config
        )
        
        self.details_fetcher = CallDetailsFetcher(
            http_client=self.http,
            auth=self.auth,
            config=self.config
        )
        
        self.customer_search_client = GongCustomerSearchClient(
            http_client=self.http,
            auth=self.auth,
            config=self.config
        )
        
        self.timeline_extractor = TimelineExtractor(
            http_client=self.http,
            auth=self.auth,
            config=self.config
        )
        
        self.email_enhancer = EmailEnhancer(
            http_client=self.http,
            auth=self.auth,
            config=self.config
        )
        
        console.print("[green]Setup complete![/green]")
    
    async def extract_team_calls(self, 
                                call_stream_id: str = "195005774106634129",
                                days_back: int = None,
                                from_date: str = None,
                                to_date: str = None) -> List[Dict[str, Any]]:
        """
        Extract team calls from Gong call stream with pagination support.
        
        Args:
            call_stream_id: Call stream ID to extract calls from
            days_back: Number of days back to include calls from (ignored if from_date/to_date provided)
            from_date: Start date for extraction (YYYY-MM-DD format, optional)
            to_date: End date for extraction (YYYY-MM-DD format, optional)
            
        Returns:
            List of detailed call information
        """
        # Determine date range display message
        if from_date or to_date:
            date_range_desc = f"from {from_date or 'beginning'} to {to_date or 'now'}"
        else:
            days_back = days_back or 7  # Default to 7 days if none specified
            date_range_desc = f"last {days_back} days"
        
        console.print(f"[cyan]Extracting calls from {date_range_desc}...[/cyan]")
        
        # Step 1: Get all calls from call stream with pagination
        all_calls = []
        offset = 0
        api_page_size = 10  # Gong API returns 10 calls per page
        
        while True:
            # Pass date parameters based on what was provided
            if from_date is not None or to_date is not None:
                stream_response = await self.library_client.get_library_calls(
                    call_stream_id=call_stream_id,
                    from_date=from_date,
                    to_date=to_date,
                    offset=offset
                )
            else:
                stream_response = await self.library_client.get_library_calls(
                    call_stream_id=call_stream_id,
                    days_back=days_back,
                    offset=offset
                )
            
            # Break if no response or no calls returned
            if not stream_response or not stream_response.get("calls"):
                break
            
            calls_batch = stream_response["calls"]
            
            # Break if we got 0 calls (end of data)
            if len(calls_batch) == 0:
                break
                
            all_calls.extend(calls_batch)
            console.print(f"[cyan]Retrieved {len(calls_batch)} calls (offset: {offset}, total: {len(all_calls)})[/cyan]")
            
            # Increment offset by API page size
            offset += api_page_size
        
        if not all_calls:
            console.print("[red]No calls found in call stream![/red]")
            return []
        
        # Filter calls by date client-side if needed
        if days_back and days_back > 0:
            filtered_calls = self.library_client._filter_calls_by_date(all_calls, days_back)
            console.print(f"[green]Found {len(all_calls)} total calls, {len(filtered_calls)} in last {days_back} days[/green]")
            library_calls = filtered_calls
        else:
            console.print(f"[green]Found {len(all_calls)} calls in call stream[/green]")
            library_calls = all_calls
        
        # Step 2: Get detailed information for each call
        detailed_calls = []
        
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
            TimeElapsedColumn(),
            console=console
        ) as progress:
            
            details_task = progress.add_task(
                "Fetching call details...", 
                total=len(library_calls)
            )
            
            for i, call in enumerate(library_calls):
                call_id = call.get("id")
                if not call_id:
                    progress.update(details_task, advance=1)
                    continue
                
                progress.update(details_task, 
                              description=f"Fetching details for call {i+1}/{len(library_calls)}")
                
                # Get detailed call information (mainly for transcript)
                details = await self.details_fetcher.get_call_details(call_id)
                
                # Preserve library data and add transcript + attendees from details
                merged_call = dict(call)  # Start with library data
                if details:
                    # Add transcript from detailed call data
                    if details.get("transcript"):
                        merged_call["transcript"] = details["transcript"]
                    else:
                        merged_call["transcript"] = "No transcript available."
                    
                    # Use attendees from detailed call data (which has actual names)
                    if details.get("attendees"):
                        merged_call["attendees"] = details["attendees"]
                else:
                    merged_call["transcript"] = "No transcript available."
                
                # Ensure we have attendees (fallback if detailed extraction failed)
                if "attendees" not in merged_call:
                    merged_call["attendees"] = []
                
                detailed_calls.append(merged_call)
                
                progress.update(details_task, advance=1)
        
        console.print(f"[green]Successfully extracted details for {len(detailed_calls)} calls[/green]")
        return detailed_calls
    
    async def extract_customer_calls(self, 
                                   customer_name: str,
                                   days_back: int = None,
                                   from_date: str = None,
                                   to_date: str = None) -> Tuple[List[Dict[str, Any]], str]:
        """
        Extract calls for a specific customer with pagination support.
        
        Args:
            customer_name: Customer/company name to search for
            days_back: Number of days back (ignored if from_date/to_date provided)
            from_date: Start date (YYYY-MM-DD format, optional)
            to_date: End date (YYYY-MM-DD format, optional)
            
        Returns:
            Tuple of (detailed call information, resolved customer name from text-suggestions)
        """
        # For customer searches, default to 60 days (not 7 like folder searches)
        if from_date or to_date:
            date_range_desc = f"from {from_date or 'beginning'} to {to_date or 'now'}"
        else:
            days_back = days_back or 90  # Customer searches default to 90 days
            date_range_desc = f"last {days_back} days"
        
        console.print(f"[cyan]Extracting calls for customer '{customer_name}' from {date_range_desc}...[/cyan]")
        
        # Step 1: Calculate date range for filtering during pagination
        from datetime import datetime, timedelta
        
        if from_date or to_date:
            # Use custom date range
            start_date = None
            end_date = None
            
            if from_date:
                try:
                    start_date = datetime.strptime(from_date, "%Y-%m-%d")
                except ValueError:
                    console.print(f"[red]Invalid from_date format: {from_date}. Using beginning of time.[/red]")
                    start_date = None
                    
            if to_date:
                try:
                    # Add 23:59:59 to include the entire end date
                    end_date = datetime.strptime(to_date, "%Y-%m-%d").replace(hour=23, minute=59, second=59)
                except ValueError:
                    console.print(f"[red]Invalid to_date format: {to_date}. Using current time.[/red]")
                    end_date = None
        else:
            # Use days_back
            if days_back and days_back > 0:
                start_date = datetime.now() - timedelta(days=days_back)
                end_date = None  # Up to now
            else:
                # No filtering
                start_date = None
                end_date = None
        
        # Step 2: Get customer calls with smart pagination (stop when dates get too old)
        filtered_calls = []
        resolved_customer_name = customer_name  # Default fallback
        offset = 0
        api_page_size = 10  # Same as folder API
        
        while True:
            # Get customer calls using the new API
            customer_response = await self.customer_search_client.get_customer_calls(
                customer_name=customer_name,
                page_size=api_page_size,
                calls_offset=offset
            )
            
            # Capture resolved customer name from first response
            if offset == 0 and customer_response.get("companies"):
                # Use the first resolved company name as the authoritative customer name
                resolved_customer_name = customer_response["companies"][0]
                console.print(f"[green]Resolved customer name: '{resolved_customer_name}'[/green]")
            
            # Break if no response or no calls returned
            if not customer_response or not customer_response.get("calls"):
                break
            
            calls_batch = customer_response["calls"]
            
            # Break if we got 0 calls (end of data)
            if len(calls_batch) == 0:
                break
            
            # Filter calls in this batch by date and check if we should stop pagination
            batch_filtered_calls = []
            should_stop_pagination = False
            
            for call in calls_batch:
                # Try to get call date from multiple possible fields
                call_date_str = call.get("userTimezoneActivityTime") or call.get("date") or call.get("started_at") or call.get("startTime")
                if not call_date_str:
                    # Keep calls without dates to be safe
                    batch_filtered_calls.append(call)
                    continue
                
                try:
                    # Try to parse the date string with multiple formats
                    call_date = None
                    for fmt in [
                        "%Y/%m/%d %H:%M:%S",      # Gong userTimezoneActivityTime format  
                        "%Y-%m-%dT%H:%M:%S.%fZ",  # ISO with microseconds
                        "%Y-%m-%dT%H:%M:%SZ",     # ISO without microseconds  
                        "%Y-%m-%d %H:%M:%S",      # Standard format
                        "%Y-%m-%d",               # Date only
                        "%m/%d/%Y",               # US date format
                    ]:
                        try:
                            call_date = datetime.strptime(call_date_str, fmt)
                            break
                        except ValueError:
                            continue
                    
                    if call_date is None:
                        # Couldn't parse date, keep the call to be safe
                        batch_filtered_calls.append(call)
                        continue
                    
                    # Check if call is too old - if so, we can stop pagination 
                    # (assuming calls are sorted by date desc)
                    if start_date and call_date < start_date:
                        should_stop_pagination = True
                        break  # Stop processing this batch
                    
                    # Apply date filtering
                    include_call = True
                    
                    if end_date and call_date > end_date:
                        include_call = False
                    
                    if include_call:
                        batch_filtered_calls.append(call)
                        
                except Exception:
                    # Error parsing, keep the call to be safe
                    batch_filtered_calls.append(call)
            
            # Add the filtered calls from this batch
            filtered_calls.extend(batch_filtered_calls)
            console.print(f"[cyan]Retrieved {len(calls_batch)} calls, {len(batch_filtered_calls)} in date range (offset: {offset}, total: {len(filtered_calls)})[/cyan]")
            
            # Stop pagination if we've hit calls that are too old
            if should_stop_pagination:
                console.print("[yellow]Stopping pagination - reached calls outside date range[/yellow]")
                break
            
            # Check if there are more calls to fetch
            if not customer_response.get("hasMore", False):
                break
            
            # Increment offset by API page size
            offset += api_page_size
        
        if not filtered_calls:
            console.print(f"[red]No calls found for customer '{customer_name}' in the specified date range![/red]")
            return []
        
        console.print(f"[green]Found {len(filtered_calls)} calls for customer '{customer_name}'[/green]")
        
        # Step 2: Get detailed information for each call (same as folder extraction)
        detailed_calls = []
        
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
            TimeElapsedColumn(),
            console=console
        ) as progress:
            
            details_task = progress.add_task(
                "Fetching call details...", 
                total=len(filtered_calls)
            )
            
            for i, call in enumerate(filtered_calls):
                call_id = call.get("id")
                if not call_id:
                    progress.update(details_task, advance=1)
                    continue
                
                progress.update(details_task, 
                              description=f"Fetching details for call {i+1}/{len(filtered_calls)}")
                
                # Get detailed call information (mainly for transcript)
                details = await self.details_fetcher.get_call_details(call_id)
                
                # Preserve customer data and add transcript + attendees from details
                merged_call = dict(call)  # Start with customer data
                if details:
                    # Add transcript from detailed call data
                    if details.get("transcript"):
                        merged_call["transcript"] = details["transcript"]
                    else:
                        merged_call["transcript"] = "No transcript available."
                    
                    # Use attendees from detailed call data (which has actual names)
                    if details.get("attendees"):
                        merged_call["attendees"] = details["attendees"]
                else:
                    merged_call["transcript"] = "No transcript available."
                
                # Ensure we have attendees (fallback if detailed extraction failed)
                if "attendees" not in merged_call:
                    merged_call["attendees"] = []
                
                detailed_calls.append(merged_call)
                
                progress.update(details_task, advance=1)
        
        console.print(f"[green]Successfully extracted details for {len(detailed_calls)} calls for customer '{resolved_customer_name}'[/green]")
        return detailed_calls, resolved_customer_name

    async def extract_customer_communications(self, 
                                            customer_name: str,
                                            days_back: int = None,
                                            from_date: str = None,
                                            to_date: str = None,
                                            include_emails: bool = True,
                                            emails_only: bool = False,
                                            fetch_email_bodies: bool = False) -> Tuple[List[Dict[str, Any]], List[Email], str]:
        """
        Extract both calls and emails for a customer using timeline extraction.
        
        Args:
            customer_name: Customer/company name to search for
            days_back: Number of days back (ignored if from_date/to_date provided)
            from_date: Start date (YYYY-MM-DD format, optional)
            to_date: End date (YYYY-MM-DD format, optional)
            include_emails: Whether to include emails in results
            emails_only: Whether to extract only emails (no calls)
            fetch_email_bodies: Whether to fetch full email body content
            
        Returns:
            Tuple of (calls, emails, resolved_customer_name)
        """
        # For customer searches, default to 60 days (not 7 like folder searches)
        if from_date or to_date:
            date_range_desc = f"from {from_date or 'beginning'} to {to_date or 'now'}"
        else:
            days_back = days_back or 90  # Customer searches default to 90 days
            date_range_desc = f"last {days_back} days"

        console.print(f"[cyan]Extracting communications for customer '{customer_name}' from {date_range_desc}...[/cyan]")
        if emails_only:
            console.print("[yellow]Extracting only emails (calls will be ignored)[/yellow]")
        elif include_emails:
            console.print("[yellow]Including emails with advanced BDR/SPAM filtering[/yellow]")

        # Step 1: Calculate date range
        from datetime import datetime, timedelta
        
        if from_date or to_date:
            start_date = None
            end_date = None
            
            if from_date:
                try:
                    start_date = datetime.strptime(from_date, "%Y-%m-%d")
                except ValueError:
                    console.print(f"[red]Invalid from_date format: {from_date}. Using beginning of time.[/red]")
                    start_date = None
                    
            if to_date:
                try:
                    end_date = datetime.strptime(to_date, "%Y-%m-%d").replace(hour=23, minute=59, second=59)
                except ValueError:
                    console.print(f"[red]Invalid to_date format: {to_date}. Using current time.[/red]")
                    end_date = None
        else:
            if days_back and days_back > 0:
                start_date = datetime.now() - timedelta(days=days_back)
                end_date = None  # Up to now
            else:
                start_date = None
                end_date = None

        # Step 2: Find customer accounts using existing search functionality
        # Fetch a few calls to extract account IDs from their CRM data
        customer_response = await self.customer_search_client.get_customer_calls(
            customer_name=customer_name,
            page_size=10,  # Get 10 calls to extract account IDs from
            calls_offset=0
        )
        
        if not customer_response:
            console.print(f"[red]No customer found matching '{customer_name}'[/red]")
            return [], [], customer_name
        
        # Capture resolved customer name from text-suggestions endpoint
        resolved_customer_name = customer_name  # Default fallback
        if customer_response.get("companies"):
            resolved_customer_name = customer_response["companies"][0]
            console.print(f"[green]Resolved customer name: '{resolved_customer_name}'[/green]")
        
        # Use account IDs from the autocomplete endpoint
        account_ids = customer_response.get("account_ids", [])
        
        if not account_ids:
            console.print(f"[red]No accounts found for customer '{customer_name}'[/red]")
            return [], []
        
        console.print(f"[green]Found {len(account_ids)} accounts for customer '{customer_name}'[/green]")
        
        # Step 3: Use timeline extractor to get communications from these accounts
        all_calls = []
        all_emails = []
        
        for account_id in account_ids:
            try:
                calls, emails = await self.timeline_extractor.extract_account_timeline(
                    account_id=account_id,
                    start_date=start_date or datetime.now() - timedelta(days=60),
                    end_date=end_date
                )
                
                if not emails_only:
                    all_calls.extend(calls)
                if include_emails or emails_only:
                    all_emails.extend(emails)
                    
            except Exception as e:
                logger.error(f"Failed to extract timeline for account {account_id}: {e}")
                continue
        
        console.print(f"[green]Timeline extraction complete: {len(all_calls)} calls, {len(all_emails)} emails[/green]")
        
        # Step 4: Enhance email bodies if requested
        if (include_emails or emails_only) and fetch_email_bodies and all_emails:
            console.print("[cyan]Fetching email body content...[/cyan]")
            all_emails = await self.email_enhancer.enhance_emails_with_bodies(
                all_emails, fetch_bodies=True
            )
            console.print("[green]Email body enhancement complete[/green]")
        
        # Step 5: For calls, get detailed information (transcripts) if not emails-only
        detailed_calls = []
        if not emails_only and all_calls:
            with Progress(
                SpinnerColumn(),
                TextColumn("[progress.description]{task.description}"),
                BarColumn(),
                TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
                TimeElapsedColumn(),
                console=console
            ) as progress:
                
                details_task = progress.add_task(
                    "Fetching call details...", 
                    total=len(all_calls)
                )
                
                for i, call in enumerate(all_calls):
                    progress.update(details_task, 
                                  description=f"Fetching details for call {i+1}/{len(all_calls)}")
                    
                    # Convert Call model to dict format expected by existing code
                    call_dict = {
                        "id": call.id,
                        "accountId": call.account_id,
                        "title": call.title,
                        "duration": call.duration,
                        "scheduledStart": call.scheduled_start,
                        "participants": [
                            {
                                "name": p.name,
                                "email": p.email,
                                "title": p.title,
                                "company": p.company
                            }
                            for p in call.participants
                        ] if call.participants else [],
                        "attendees": [
                            {
                                "name": p.name,
                                "email": p.email,
                                "title": p.title,
                                "company": p.company
                            }
                            for p in call.participants
                        ] if call.participants else []
                    }
                    
                    # Get detailed call information (mainly for transcript)
                    details = await self.details_fetcher.get_call_details(call.id)
                    
                    if details:
                        if details.get("transcript"):
                            call_dict["transcript"] = details["transcript"]
                        else:
                            call_dict["transcript"] = "No transcript available."
                    else:
                        call_dict["transcript"] = "No transcript available."
                    
                    detailed_calls.append(call_dict)
                    progress.update(details_task, advance=1)
        
        console.print(f"[green]Successfully extracted {len(detailed_calls)} calls and {len(all_emails)} emails for customer '{resolved_customer_name}'[/green]")
        return detailed_calls, all_emails, resolved_customer_name
    
    async def save_calls_as_markdown(self, calls: List[Dict[str, Any]], customer_name: str = None) -> List[Path]:
        """Save calls as individual markdown files."""
        console.print("[cyan]Generating markdown files...[/cyan]")
        
        # Use customer name for directory if provided
        saved_files = self.formatter.save_multiple_calls(calls, custom_dir_name=customer_name)
        
        console.print(f"[green]Saved {len(saved_files)} markdown files[/green]")
        
        # Generate summary report in the same directory on desktop
        desktop_path = Path.home() / "Desktop"
        if customer_name:
            sanitized_name = self.formatter._sanitize_filename(customer_name)
            summary_path = desktop_path / f"ct_{sanitized_name}" / "SUMMARY.md"
        else:
            today = datetime.now().strftime('%Y-%m-%d')
            summary_path = desktop_path / f"team-calls-{today}" / "SUMMARY.md"
        
        # Note: resolved_customer_name not available here for folder extractions, 
        # will use fallback extraction from call titles
        self.summary_reporter.generate_summary_report(calls, summary_path)
        console.print(f"[green]Summary report saved to {summary_path}[/green]")
        
        return saved_files
    
    async def save_calls_as_markdown_with_resolved_name(self, calls: List[Dict[str, Any]], 
                                                        customer_name: str = None, 
                                                        resolved_customer_name: str = None) -> List[Path]:
        """Save calls as individual markdown files using resolved customer name for summary."""
        console.print("[cyan]Generating markdown files...[/cyan]")
        
        # Use customer name for directory if provided
        saved_files = self.formatter.save_multiple_calls(calls, custom_dir_name=customer_name)
        
        console.print(f"[green]Saved {len(saved_files)} markdown files[/green]")
        
        # Generate summary report in the same directory with resolved customer name on desktop
        desktop_path = Path.home() / "Desktop"
        if customer_name:
            sanitized_name = self.formatter._sanitize_filename(customer_name)
            summary_path = desktop_path / f"ct_{sanitized_name}" / "SUMMARY.md"
        else:
            today = datetime.now().strftime('%Y-%m-%d')
            summary_path = desktop_path / f"team-calls-{today}" / "SUMMARY.md"
        
        # Use resolved customer name for better summary accuracy
        self.summary_reporter.generate_summary_report(calls, summary_path, resolved_customer_name)
        console.print(f"[green]Summary report saved to {summary_path}[/green]")
        
        return saved_files
    
    async def save_emails_as_markdown(self, emails: List[Email], customer_name: str = None) -> List[Path]:
        """Save emails as individual markdown files in batches."""
        console.print("[cyan]Generating email markdown files...[/cyan]")
        
        # Use the formatter's email saving functionality
        saved_files = self.formatter.save_emails_as_markdown(emails, customer_name or "Unknown-Customer", custom_dir_name=customer_name)
        
        console.print(f"[green]Saved {len(emails)} emails across {len(saved_files)} batch files[/green]")
        
        return saved_files
    
    async def cleanup(self) -> None:
        """Cleanup resources."""
        if self.http:
            await self.http.__aexit__(None, None, None)


def interactive_mode() -> Tuple[str, int, str]:
    """Interactive mode for users who don't provide command-line arguments."""
    console.print("\n[bold cyan]CS-CLI: Customer Success Deep Research Tool[/bold cyan]")
    console.print("[dim]Let's find insights from your customer conversations[/dim]\n")
    
    # Get customer name
    customer = Prompt.ask("[cyan]What customer are you looking for?[/cyan]")
    
    # Get days with a sensible default
    console.print("\n[cyan]How far back should I look?[/cyan]")
    console.print("[dim]Common choices: 30 days (1 month), 90 days (3 months), 180 days (6 months)[/dim]")
    days_str = Prompt.ask("Number of days", default="90")
    
    try:
        days = int(days_str)
    except ValueError:
        console.print("[yellow]Using default: 90 days[/yellow]")
        days = 90
    
    # Get content type with a menu
    console.print("\n[cyan]What would you like to analyze?[/cyan]\n")
    
    table = Table(show_header=False, show_edge=False)
    table.add_column("", style="bright_cyan", width=4)
    table.add_column("", style="white")
    
    table.add_row("1.", "Calls only")
    table.add_row("2.", "Emails only")
    table.add_row("3.", "Both calls and emails (recommended)")
    
    console.print(table)
    
    choice = Prompt.ask("\nType a number and press Enter", default="3")
    
    content_map = {
        "1": "calls",
        "2": "emails", 
        "3": "calls emails"
    }
    
    content_type = content_map.get(choice, "calls emails")
    
    # Show summary
    console.print(f"\n[green]✓ Looking for:[/green] {customer}")
    console.print(f"[green]✓ Time period:[/green] Last {days} days")
    content_display = "Calls and emails" if content_type == "calls emails" else content_type.capitalize()
    console.print(f"[green]✓ Content:[/green] {content_display}\n")
    
    return customer, days, content_type


def parse_arguments(args: tuple) -> Tuple[str, int, str]:
    """
    Smart parsing: Last non-hyphenated number is days
    Returns: (customer, days, content_type)
    """
    if not args:
        return None, None, ""
    
    # Step 1: Extract content keywords
    content_keywords = []
    remaining_args = []
    
    for arg in args:
        if arg.lower() in ['calls', 'emails']:
            content_keywords.append(arg.lower())
        else:
            remaining_args.append(arg)
    
    # Step 2: Smart number detection with hyphen awareness
    days = None
    days_index = None
    
    for i in reversed(range(len(remaining_args))):
        if remaining_args[i].isdigit():
            # Check if this is part of a hyphenated customer name
            # Look for pattern: number - number (like "7 - 11")
            is_hyphenated = False
            
            # Check if preceded by a hyphen (with potential number before it)
            if i > 0 and remaining_args[i-1] in ['-', '–', '—']:
                # Check if there's a number before the hyphen
                if i > 1 and remaining_args[i-2].isdigit():
                    is_hyphenated = True
            
            # Also check if this number is followed by a hyphen and another number
            if i < len(remaining_args) - 2:
                if remaining_args[i+1] in ['-', '–', '—'] and remaining_args[i+2].isdigit():
                    is_hyphenated = True
            
            if is_hyphenated:
                continue  # Part of customer name like "7 - 11"
            
            days = int(remaining_args[i])
            days_index = i
            break
    
    # Step 3: Everything else is the customer name
    if days_index is not None:
        customer_parts = remaining_args[:days_index] + remaining_args[days_index+1:]
    else:
        customer_parts = remaining_args
    
    customer = " ".join(customer_parts) if customer_parts else None
    content_type = " ".join(content_keywords)
    
    return customer, days, content_type


@click.command()
@click.argument('args', nargs=-1, required=False)
@click.option('--debug', is_flag=True, hidden=True, help="Enable debug logging")
def main(args: tuple = None, debug: bool = False) -> None:
    """Extract customer communications from Gong and save as markdown files.
    
    Run without arguments for interactive mode: just type 'cs-cli' and press Enter!
    
    Arguments can be in any order - the last number is always treated as days:
      cs-cli                              ✓ Interactive mode
      cs-cli Fiserv 180 emails            ✓ Standard order
      cs-cli 180 Fiserv emails            ✓ Days first
      cs-cli emails Fiserv 180            ✓ Content first
      cs-cli 7 - 11 365 calls             ✓ Customer with numbers
      cs-cli Fortune 500 30               ✓ Customer with numbers
    
    Examples:
      cs-cli Postman 30                   Get last 30 days of Postman
      cs-cli Wells Fargo calls 90         Get last 90 days of Wells Fargo calls
      cs-cli emails 7 - 11 365            Get last 365 days of 7-Eleven emails
    """
    
    async def async_main():
        # Parse arguments using smart parser
        customer, days, content_type = parse_arguments(args)
        
        # If no arguments provided, launch interactive mode
        if customer is None:
            customer, days, content_type = interactive_mode()
        
        # Default to 90 days if not specified
        if days is None and customer is not None:
            days = 90
        
        # Parse content type
        content_type_clean = content_type.lower() if content_type else ""
        valid_types = ["", "calls", "emails", "calls emails", "emails calls"]
        if content_type_clean not in valid_types:
            console.print("[red]Error: Content type must be 'calls', 'emails', or both[/red]")
            console.print(f"[yellow]You provided: '{content_type}'[/yellow]")
            console.print()
            console.print("[green]TIP: Just run 'cs-cli' (no arguments) for interactive mode![/green]")
            return
        
        # Determine what to extract
        extract_calls = "calls" in content_type_clean
        extract_emails = "emails" in content_type_clean
        emails_only = content_type_clean == "emails"
        
        if debug:
            logging.root.setLevel(logging.DEBUG)
            structlog.configure(
                wrapper_class=structlog.make_filtering_bound_logger(logging.DEBUG),
            )
        
        # Setup configuration
        config = GongConfig.from_env()
        if debug:
            config.debug = True
        
        # Initialize extractor
        extractor = TeamCallsExtractor(config)
        
        try:
            # Setup components
            await extractor.setup()
            
            # Extract communications using timeline extraction (always for emails)
            if extract_emails:
                console.print(f"[cyan]Extracting {content_type_clean} for '{customer}' (last {days} days)[/cyan]")
                console.print("[yellow]Using timeline extraction with advanced BDR/SPAM filtering[/yellow]")
                
                calls, emails, resolved_customer_name = await extractor.extract_customer_communications(
                    customer_name=customer,
                    days_back=days,
                    include_emails=extract_emails,
                    emails_only=emails_only,
                    fetch_email_bodies=True  # Always fetch email bodies when emails requested
                )
            else:
                # Calls only - use faster call-specific extraction
                console.print(f"[cyan]Extracting calls for '{customer}' (last {days} days)[/cyan]")
                calls, resolved_customer_name = await extractor.extract_customer_calls(
                    customer_name=customer,
                    days_back=days
                )
                emails = []
            
            # Check if we found anything
            if not calls and not emails:
                console.print(f"[yellow]No {content_type_clean} found for '{resolved_customer_name}' in the last {days} days[/yellow]")
                return
            
            if emails_only and not emails:
                console.print(f"[yellow]No emails found for '{resolved_customer_name}' in the last {days} days[/yellow]")
                return
            
            # Save results
            saved_files = []
            
            # Save calls
            if calls and extract_calls:
                call_files = await extractor.save_calls_as_markdown_with_resolved_name(
                    calls, 
                    customer_name=resolved_customer_name, 
                    resolved_customer_name=resolved_customer_name
                )
                saved_files.extend(call_files)
            
            # Save emails  
            if emails and extract_emails:
                email_files = await extractor.save_emails_as_markdown(emails, customer_name=resolved_customer_name)
                saved_files.extend(email_files)
            
            # Display results
            console.print("\n[bold green]Extraction Complete![/bold green]")
            
            if emails_only:
                console.print(f"Extracted {len(emails)} emails for '{resolved_customer_name}'")
                if emails:
                    emails_with_bodies = sum(1 for email in emails if email.body_text and email.body_text.strip())
                    console.print(f"[dim]{emails_with_bodies}/{len(emails)} emails have full body content[/dim]")
            elif extract_calls and extract_emails:
                console.print(f"Extracted {len(calls)} calls and {len(emails)} emails for '{resolved_customer_name}'")
                if emails:
                    emails_with_bodies = sum(1 for email in emails if email.body_text and email.body_text.strip())
                    console.print(f"[dim]Advanced BDR/SPAM filtering applied - {emails_with_bodies}/{len(emails)} emails have full content[/dim]")
            else:
                console.print(f"Extracted {len(calls)} calls for '{resolved_customer_name}'")
            
            console.print(f"Saved {len(saved_files)} markdown files")
            
            if saved_files:
                output_directory = saved_files[0].parent
                console.print(f"Output directory: [bold]{output_directory}[/bold]")
            
        except KeyboardInterrupt:
            console.print("\n[yellow]Extraction interrupted[/yellow]")
            sys.exit(1)
        except Exception as e:
            console.print(f"[red]Extraction failed: {e}[/red]")
            if debug:
                import traceback
                console.print(traceback.format_exc())
            sys.exit(1)
        finally:
            await extractor.cleanup()
    
    # Run the async function
    asyncio.run(async_main())


def cli():
    """Entry point for CLI."""
    main()


if __name__ == "__main__":
    main()