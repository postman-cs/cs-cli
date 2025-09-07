#!/usr/bin/env python3
"""CLI for team calls extraction."""

import asyncio
import sys
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any

import click
import structlog
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TimeElapsedColumn

from .infra.auth import GongAuthenticator
from .infra.config import GongConfig
from .infra.http import HTTPClientPool

from .api.library_client import GongLibraryClient, CallDetailsFetcher
from .api.customer_search import GongCustomerSearchClient
from .formatters.markdown import CallMarkdownFormatter, CallSummaryReporter

# Setup logging and console
console = Console()
logger = structlog.get_logger()

# Configure quiet logging by default
import logging
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
                                   to_date: str = None) -> List[Dict[str, Any]]:
        """
        Extract calls for a specific customer with pagination support.
        
        Args:
            customer_name: Customer/company name to search for
            days_back: Number of days back (ignored if from_date/to_date provided)
            from_date: Start date (YYYY-MM-DD format, optional)
            to_date: End date (YYYY-MM-DD format, optional)
            
        Returns:
            List of detailed call information for the customer
        """
        # For customer searches, default to 60 days (not 7 like folder searches)
        if from_date or to_date:
            date_range_desc = f"from {from_date or 'beginning'} to {to_date or 'now'}"
        else:
            days_back = days_back or 60  # Customer searches default to 60 days
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
        offset = 0
        api_page_size = 10  # Same as folder API
        
        while True:
            # Get customer calls using the new API
            customer_response = await self.customer_search_client.get_customer_calls(
                customer_name=customer_name,
                page_size=api_page_size,
                calls_offset=offset
            )
            
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
                console.print(f"[yellow]Stopping pagination - reached calls outside date range[/yellow]")
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
        
        console.print(f"[green]Successfully extracted details for {len(detailed_calls)} calls for customer '{customer_name}'[/green]")
        return detailed_calls
    
    async def save_calls_as_markdown(self, calls: List[Dict[str, Any]], customer_name: str = None) -> List[Path]:
        """Save calls as individual markdown files."""
        console.print("[cyan]Generating markdown files...[/cyan]")
        
        # Use customer name for directory if provided
        saved_files = self.formatter.save_multiple_calls(calls, custom_dir_name=customer_name)
        
        console.print(f"[green]Saved {len(saved_files)} markdown files[/green]")
        
        # Generate summary report in the same directory
        if customer_name:
            sanitized_name = self.formatter._sanitize_filename(customer_name)
            summary_path = Path(f"{sanitized_name}") / "SUMMARY.md"
        else:
            today = datetime.now().strftime('%Y-%m-%d')
            summary_path = Path(f"team-calls-{today}") / "SUMMARY.md"
        
        self.summary_reporter.generate_summary_report(calls, summary_path)
        console.print(f"[green]Summary report saved to {summary_path}[/green]")
        
        return saved_files
    
    async def cleanup(self) -> None:
        """Cleanup resources."""
        if self.http:
            await self.http.__aexit__(None, None, None)


@click.command()
@click.option(
    "--folder-id",
    default="195005774106634129",
    help="Gong library folder ID to extract calls from",
)
@click.option(
    "--days",
    default=7,
    help="Number of days back to extract calls from (ignored if --from-date/--to-date provided)",
    type=int,
)
@click.option(
    "--from-date",
    default=None,
    help="Start date for call extraction (YYYY-MM-DD format, overrides --days)",
    type=str,
)
@click.option(
    "--to-date", 
    default=None,
    help="End date for call extraction (YYYY-MM-DD format, overrides --days)",
    type=str,
)
@click.option(
    "--debug",
    is_flag=True,
    help="Enable debug logging",
)
@click.option(
    "--output-dir",
    default=None,
    help="Output directory for markdown files",
    type=click.Path(),
)
@click.option(
    "--customer",
    default=None,
    help="Extract calls for a specific customer (e.g., '7-11', 'Postman'). Overrides --folder-id.",
    type=str,
)
def main(folder_id: str, days: int, from_date: str, to_date: str, debug: bool, output_dir: str, customer: str) -> None:
    """Extract team calls from Gong and save as markdown files.
    
    Extract calls either by folder ID (default) or by customer name.
    Examples:
      customer-transcripts --customer "7-11" --days 30
      customer-transcripts --folder-id "123456789" --from-date 2024-01-01
    """
    
    async def async_main():
        if debug:
            # Enable debug logging
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
            
            # Determine extraction strategy: customer vs folder
            if customer:
                # Extract calls by customer name
                console.print(f"[cyan]Using customer-based extraction for: {customer}[/cyan]")
                if from_date is not None or to_date is not None:
                    calls = await extractor.extract_customer_calls(
                        customer_name=customer,
                        from_date=from_date,
                        to_date=to_date
                    )
                else:
                    calls = await extractor.extract_customer_calls(
                        customer_name=customer,
                        days_back=days
                    )
            else:
                # Extract calls by folder (original method)
                console.print(f"[cyan]Using folder-based extraction[/cyan]")
                if from_date is not None or to_date is not None:
                    # Use custom date range (including empty strings for unlimited range)
                    display_from = from_date if from_date else 'beginning'
                    display_to = to_date if to_date else 'now'
                    console.print(f"[cyan]Using custom date range: {display_from} to {display_to}[/cyan]")
                    calls = await extractor.extract_team_calls(
                        call_stream_id=folder_id,
                        from_date=from_date,
                        to_date=to_date
                    )
                else:
                    # Use days back from today
                    console.print(f"[cyan]Using {days} days back from today[/cyan]")
                    calls = await extractor.extract_team_calls(
                        call_stream_id=folder_id,
                        days_back=days
                    )
            
            if not calls:
                console.print("[yellow]No calls to process[/yellow]")
                return
            
            # Set custom output directory if provided
            if output_dir:
                extractor.formatter.output_dir = Path(output_dir)
            
            # Save as markdown with customer name for directory naming if available
            saved_files = await extractor.save_calls_as_markdown(calls, customer_name=customer)
            
            # Display results
            console.print("\\n[bold green]Extraction Complete![/bold green]")
            if customer:
                console.print(f"Extracted {len(calls)} calls for customer '{customer}'")
            else:
                if from_date or to_date:
                    date_range = f"{from_date or 'beginning'} to {to_date or 'now'}"
                    console.print(f"Extracted {len(calls)} calls from {date_range}")
                else:
                    console.print(f"Extracted {len(calls)} calls from last {days} days")
            console.print(f"Saved {len(saved_files)} markdown files")
            
            if saved_files:
                output_directory = saved_files[0].parent
                console.print(f"Output directory: [bold]{output_directory}[/bold]")
            
        except KeyboardInterrupt:
            console.print("\\n[yellow]Extraction interrupted[/yellow]")
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