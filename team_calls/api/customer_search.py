"""Gong Customer Search API client for finding calls by customer name."""

import json
from typing import Dict, List, Any, Optional

import structlog
from rich.console import Console
from rich.prompt import Prompt
from rich.table import Table

from ..infra.http import HTTPClientPool
from ..infra.auth import GongAuthenticator
from ..infra.config import GongConfig

logger = structlog.get_logger()
console = Console()




class GongCustomerSearchClient:
    """Client for Gong Customer Search and Call Filtering APIs."""
    
    def __init__(self, 
                 http_client: HTTPClientPool, 
                 auth: GongAuthenticator,
                 config: Optional[GongConfig] = None):
        self.http = http_client
        self.auth = auth
        self.config = config
        # TODO: Make workspace_id configurable - extract from existing config or env
        self.workspace_id = "5562739194953732039"
    
    async def search_customers(self, partial_name: str) -> List[str]:
        """
        Search for customers using autocomplete API.
        
        Args:
            partial_name: Partial customer/company name to search for
            
        Returns:
            List of company names matching the search
        """
        base_url = self.auth.get_base_url()
        url = f"{base_url}/conversations/ajax/text-filter-suggestions"
        
        params = {
            "workspace-id": self.workspace_id,
            "filter-name": "LeadCompanyOrAccount", 
            "partial-text": partial_name
        }
        
        logger.info("Searching for customers", partial_name=partial_name)
        
        try:
            # Get authenticated headers
            headers = await self.auth.get_authenticated_headers(retry_on_failure=True)
            
            # Add required headers for autocomplete requests
            headers.update({
                "accept": "application/json; charset=utf-8",
                "sec-fetch-dest": "empty",
                "sec-fetch-mode": "cors", 
                "sec-fetch-site": "same-origin"
            })
            
            response = await self.http.get(url, params=params, headers=headers)
            
            if response.status_code == 200:
                data = response.json()
                # Extract company names from suggestions
                suggestions = data.get("suggestions", [])
                company_names = [item.get("text", "") for item in suggestions if item.get("text")]
                
                logger.info(f"Found {len(company_names)} customer suggestions: {company_names[:5]}")
                return company_names
            else:
                logger.error("Customer search failed", 
                           status_code=response.status_code,
                           response_text=response.text[:500])
                return []
                
        except Exception as e:
            logger.error(f"Error searching customers: {e}")
            return []
    
    async def resolve_customer_companies(self, customer_name: str) -> List[str]:
        """
        Resolve customer name to list of possible company names for filtering.
        
        Args:
            customer_name: Customer name to resolve
            
        Returns:
            List of company names that can be used in call filtering
        """
        # Search for customer - API returns company names directly
        company_names = await self.search_customers(customer_name)
        
        if not company_names:
            logger.warning(f"No results found for customer: {customer_name}")
            return []
        
        # Find best matches - exact match first, then partial matches
        exact_matches = [name for name in company_names if customer_name.lower() in name.lower()]
        
        # If we have exact matches, prefer those, otherwise return all suggestions
        final_matches = exact_matches if exact_matches else company_names
        
        logger.info(f"Resolved customer '{customer_name}' to companies: {final_matches}")
        return final_matches
    
    async def select_customer_company(self, customer_name: str, company_names: List[str]) -> Optional[str]:
        """
        Prompt user to select the correct company when multiple matches are found.
        
        Args:
            customer_name: Original customer name searched for
            company_names: List of matching company names
            
        Returns:
            Selected company name or None if cancelled
        """
        if not company_names:
            return None
            
        if len(company_names) == 1:
            # Only one match, use it automatically
            console.print(f"\n[green]Found customer:[/green] {company_names[0]}")
            return company_names[0]
        
        # Multiple matches - show selection table
        console.print(f"\n[yellow]I found {len(company_names)} companies matching '{customer_name}'[/yellow]")
        console.print("[dim]Which one are you looking for?[/dim]\n")
        
        # Create a nice table for selection
        table = Table(show_header=False, show_edge=False)
        table.add_column("", style="bright_cyan", width=4)
        table.add_column("", style="white")
        
        # Show up to 10 options
        display_count = min(len(company_names), 10)
        for idx, company in enumerate(company_names[:display_count], 1):
            table.add_row(f"{idx}.", company)
        
        console.print(table)
        
        if len(company_names) > 10:
            console.print(f"\n[dim]Showing first 10 of {len(company_names)} matches[/dim]")
        
        # Add option to search again
        console.print(f"\n[dim]{display_count + 1}. None of these - search again[/dim]")
        console.print("[dim]0. Cancel and exit[/dim]\n")
        
        # Get user choice
        while True:
            choice = Prompt.ask(
                "Type a number and press Enter",
                default="1"
            )
            
            try:
                choice_num = int(choice)
                
                if choice_num == 0:
                    console.print("\n[yellow]Cancelled - no files will be extracted.[/yellow]")
                    return None
                elif choice_num == display_count + 1:
                    # User wants to search again
                    return "SEARCH_AGAIN"
                elif 1 <= choice_num <= display_count:
                    selected = company_names[choice_num - 1]
                    console.print(f"\n[green]✓ Selected:[/green] {selected}\n")
                    return selected
                else:
                    console.print("[red]Please enter a number from the list above.[/red]")
            except ValueError:
                console.print("[red]Please enter a number (like 1 or 2).[/red]")
    
    async def get_customer_calls(self, 
                               customer_name: str,
                               page_size: int = 10,
                               calls_offset: int = 0,
                               interactive: bool = True) -> Dict[str, Any]:
        """
        Get calls filtered by customer name with pagination.
        
        Args:
            customer_name: Customer name to filter by
            page_size: Number of calls per page (default 10, same as Gong UI)
            calls_offset: Offset for pagination
            interactive: Whether to prompt for selection when multiple matches found
            
        Returns:
            Dictionary containing calls data and pagination info
        """
        # First resolve customer name to company names
        company_names = await self.resolve_customer_companies(customer_name)
        
        if not company_names:
            logger.warning(f"Could not resolve customer name: {customer_name}")
            console.print(f"[red]No customers found matching '{customer_name}'[/red]")
            return {"calls": [], "hasMore": False, "totalCount": 0}
        
        # Handle interactive selection if enabled
        if interactive:
            selected_company = await self.select_customer_company(customer_name, company_names)
            
            if selected_company is None:
                # User cancelled
                return {"calls": [], "hasMore": False, "totalCount": 0}
            elif selected_company == "SEARCH_AGAIN":
                # User wants to search for a different customer
                console.print("\n[cyan]Let's try a different search.[/cyan]")
                new_customer = Prompt.ask("Enter the customer name")
                return await self.get_customer_calls(new_customer, page_size, calls_offset, interactive)
            else:
                # Use the selected company
                company_names = [selected_company]
        
        # Build the search filter payload
        search_filter = {
            "search": {
                "type": "And",
                "filters": [{
                    "type": "LeadCompanyOrAccount",
                    "names": company_names
                }]
            }
        }
        
        payload = {
            "pageSize": page_size,
            "callsOffset": calls_offset,
            "callsSearchJson": json.dumps(search_filter)
        }
        
        base_url = self.auth.get_base_url()
        url = f"{base_url}/conversations/ajax/results"
        
        params = {
            "workspace-id": self.workspace_id
        }
        
        # Show progress to user
        if calls_offset == 0:  # Only show on first page
            console.print("[dim]Downloading calls and emails...[/dim]")
        
        logger.info("Fetching customer calls", 
                   customer=customer_name,
                   companies=company_names,
                   page_size=page_size,
                   offset=calls_offset)
        
        try:
            # Get authenticated headers
            headers = await self.auth.get_authenticated_headers(retry_on_failure=True)
            
            # Add required headers for POST requests
            headers.update({
                "accept": "application/json; charset=utf-8",
                "content-type": "application/json",
                "sec-fetch-dest": "empty",
                "sec-fetch-mode": "cors",
                "sec-fetch-site": "same-origin"
            })
            
            response = await self.http.post(url, params=params, headers=headers, json=payload)
            
            if response.status_code == 200:
                data = response.json()
                
                # Extract calls from response (structure may vary)
                calls = self._extract_calls_from_response(data)
                
                logger.info(f"Successfully retrieved {len(calls)} calls for customer {customer_name}")
                
                return {
                    "calls": calls,
                    "hasMore": len(calls) == page_size,  # Assume more if we got a full page
                    "totalCount": len(calls),  # This might be available in response
                    "companies": company_names
                }
            else:
                logger.error("Customer calls API request failed",
                           status_code=response.status_code, 
                           response_text=response.text[:500])
                return {"calls": [], "hasMore": False, "totalCount": 0}
                
        except Exception as e:
            logger.error(f"Error fetching customer calls: {e}")
            return {"calls": [], "hasMore": False, "totalCount": 0}
    
    def _extract_calls_from_response(self, api_response: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Extract call data from the conversations API response.
        
        Args:
            api_response: Raw API response data
            
        Returns:
            List of call info dictionaries
        """
        calls = []
        
        # Check for the actual response structure from Gong customer search API
        if "callItemsStream" in api_response:
            calls_data = api_response["callItemsStream"]
            logger.debug(f"Found callItemsStream with {len(calls_data) if isinstance(calls_data, list) else 'unknown'} items")
        elif "items" in api_response:
            calls_data = api_response["items"]
            logger.debug(f"Found items with {len(calls_data) if isinstance(calls_data, list) else 'unknown'} items")
        elif "calls" in api_response:
            calls_data = api_response["calls"]
        elif "results" in api_response:
            calls_data = api_response["results"]
        elif "data" in api_response:
            calls_data = api_response["data"]
        else:
            # Try to find calls in the response structure
            logger.debug("Exploring API response structure", keys=list(api_response.keys()))
            calls_data = []
        
        for item in calls_data if isinstance(calls_data, list) else []:
            if isinstance(item, dict):
                # Log available fields for debugging
                logger.debug(f"Call item fields: {list(item.keys())}")
                
                # Extract title and customer info
                title = item.get("title", "") or item.get("name", "")
                customer_name = item.get("customerAccountName", "") or item.get("accountName", "") or item.get("customer", "")
                
                # Check for customer info in CRM data (primary source)
                if not customer_name and "crmData" in item:
                    crm_data = item["crmData"]
                    if isinstance(crm_data, dict):
                        # Look for accounts array
                        if "accounts" in crm_data and isinstance(crm_data["accounts"], list) and len(crm_data["accounts"]) > 0:
                            first_account = crm_data["accounts"][0]
                            if isinstance(first_account, dict):
                                customer_name = first_account.get("name", "")
                        
                        # Fallback to other CRM data fields
                        if not customer_name:
                            customer_name = crm_data.get("accountName", "") or crm_data.get("companyName", "")
                
                # If still no customer name, extract from title (many titles contain customer names)
                if not customer_name and title:
                    # Look for patterns like "CustomerName - meeting" or "Meeting with CustomerName"  
                    if " - " in title:
                        potential_customer = title.split(" - ")[0].strip()
                        # Only use if it looks like a company name (not just a generic word)
                        if len(potential_customer) > 3 and potential_customer.lower() not in ["call", "meeting", "sync", "demo"]:
                            customer_name = potential_customer
                    elif "with " in title.lower():
                        # Try to extract after "with"
                        parts = title.lower().split("with ")
                        if len(parts) > 1:
                            potential_customer = parts[1].split(" ")[0].strip()
                            if len(potential_customer) > 3:
                                customer_name = potential_customer
                
                call_info = {
                    "id": item.get("id") or item.get("callId") or item.get("call_id"),
                    "title": title,
                    "generatedTitle": item.get("generatedTitle", ""),
                    "customer_name": customer_name,
                    "date": item.get("effectiveStartDateTime", "") or item.get("startDate", "") or item.get("date", ""),
                    "duration": item.get("duration", 0),
                    "participants": item.get("participants", []) or item.get("attendees", []),
                    "call_url": item.get("callUrl", "") or item.get("url", ""),
                    "raw_data": item  # Keep original for debugging
                }
                
                # Only add if we have a valid call ID
                if call_info["id"]:
                    calls.append(call_info)
                    logger.debug(f"Found customer call: {call_info['title']} - {call_info['customer_name']} - {call_info['date']}")
                else:
                    logger.debug(f"Skipping item without valid ID: {item.keys()}")
        
        logger.debug(f"Extracted {len(calls)} calls from customer search response")
        return calls