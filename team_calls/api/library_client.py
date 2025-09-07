"""Gong Library API client for extracting call data."""

import asyncio
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional

import structlog

from ..infra.http import HTTPClientPool
from ..infra.auth import GongAuthenticator
from ..infra.config import GongConfig

logger = structlog.get_logger()


class GongLibraryClient:
    """Client for Gong Library API endpoints."""
    
    def __init__(self, 
                 http_client: HTTPClientPool, 
                 auth: GongAuthenticator,
                 config: Optional[GongConfig] = None):
        self.http = http_client
        self.auth = auth
        self.config = config
        # Use dynamic base URL from authenticator
        self.base_url = None
    
    async def get_library_calls(self, 
                               call_stream_id: str = "195005774106634129",
                               days_back: int = None,
                               from_date: str = None,
                               to_date: str = None,
                               offset: int = 0) -> Dict[str, Any]:
        """
        Get calls from a Gong call stream for the specified date range with pagination.
        
        Args:
            call_stream_id: Call stream ID to query
            days_back: Number of days back from today to include (ignored if from_date/to_date provided)
            from_date: Start date in YYYY-MM-DD format (optional)
            to_date: End date in YYYY-MM-DD format (optional)
            offset: Pagination offset for retrieving calls
            
        Returns:
            Dictionary containing calls data
        """
        # Get dynamic base URL from authenticator
        base_url = self.auth.get_base_url()
        url = f"{base_url}/callstream/read-content"
        
        # Determine date range based on provided parameters
        if from_date is not None or to_date is not None:
            # Use provided dates (can be None/empty for unlimited range)
            from_date_str = from_date or ""
            to_date_str = to_date or ""
        else:
            # Calculate date range from days_back parameter  
            days_back = days_back or 7  # Default to 7 days if not specified
            to_date_obj = datetime.now()
            from_date_obj = to_date_obj - timedelta(days=days_back)
            from_date_str = from_date_obj.strftime('%Y-%m-%d')
            to_date_str = to_date_obj.strftime('%Y-%m-%d')
        
        # Use call stream parameters with pagination support
        params = {
            "call-stream-id": call_stream_id,
            "offset": offset,
            "from-date": from_date_str,  # Use calculated date range
            "to-date": to_date_str       # Use calculated date range  
        }
        
        logger.info("Fetching call stream data", 
                   call_stream_id=call_stream_id,
                   offset=offset,
                   from_date=from_date_str,
                   to_date=to_date_str,
                   days_back=days_back if not (from_date or to_date) else None,
                   note="Getting calls with date range and pagination support")
        
        try:
            # Get authenticated headers including CSRF token
            headers = await self.auth.get_authenticated_headers(retry_on_failure=True)
            
            # Add required headers based on working curl command
            headers.update({
                "accept": "application/json",
                "content-type": "application/json", 
                # Note: x-requested-with will be provided by authenticator, avoid duplicate
                "referer": f"{base_url}/callstream/read-content?call-stream-id={call_stream_id}",
                "sec-fetch-dest": "empty",
                "sec-fetch-mode": "cors",
                "sec-fetch-site": "same-origin"
            })
            
            # Make the request with proper headers
            response = await self.http.get(url, params=params, headers=headers)
            
            if response.status_code == 200:
                data = response.json()
                
                # Extract calls from the response
                calls = self._extract_call_ids_and_metadata(data)
                
                logger.debug(f"Successfully received call stream data with {len(calls)} calls")
                
                return {
                    "calls": calls
                }
            else:
                logger.error(f"Call stream API request failed", 
                           status_code=response.status_code,
                           response_text=response.text[:500])  # Truncate for readability
                return {"calls": []}
                
        except Exception as e:
            logger.error(f"Error fetching call stream data: {e}")
            return {"calls": []}
    
    def _extract_call_ids_and_metadata(self, api_response: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Extract call IDs and basic metadata from the call stream API response.
        
        Args:
            api_response: Raw API response data
            
        Returns:
            List of call info with IDs and basic metadata
        """
        calls = []
        
        # Based on call stream API response structure
        if "folderContent" in api_response:
            folder_content = api_response["folderContent"]
            if "calls" in folder_content and isinstance(folder_content["calls"], list):
                call_data = folder_content["calls"]
                
                for item in call_data:
                    if isinstance(item, dict):
                        # Clean customer name to remove duplications
                        raw_customer_name = item.get("customerAccountName", "")
                        clean_customer_name = self._deduplicate_customer_name(raw_customer_name)
                        
                        call_info = {
                            "id": item.get("id"),
                            "title": item.get("title", ""),
                            "customer_name": clean_customer_name,
                            "date": item.get("effectiveStartDateTime", ""),
                            "duration": item.get("duration", 0),
                            "participants": item.get("participants", []),
                            "call_url": item.get("callUrl", ""),  # Extract call URL
                            "raw_data": item  # Keep original for debugging
                        }
                        
                        # Only add if we have a valid call ID
                        if call_info["id"]:
                            calls.append(call_info)
                            logger.debug(f"Found call: {call_info['title']} - {call_info['customer_name']} - {call_info['date']}")
        
        logger.debug(f"Extracted {len(calls)} call IDs from call stream response")
        return calls
    
    def _deduplicate_customer_name(self, customer_name: str) -> str:
        """
        Remove duplicated customer names from comma-separated lists.
        
        Examples:
        - "AT&T, AT&T, AT&T, AT&T, AT&T" -> "AT&T"
        - "Deloitte, Deloitte, Deloitte" -> "Deloitte"
        - "rbcnomail.com, RBC, RBC, RBC, RBC, RBC" -> "rbcnomail.com, RBC"
        
        Args:
            customer_name: Raw customer name string, potentially with duplicates
            
        Returns:
            Cleaned customer name with duplicates removed
        """
        if not customer_name:
            return customer_name
        
        # Split by comma and clean each part
        parts = [part.strip() for part in customer_name.split(",")]
        
        # Remove duplicates while preserving order
        seen = set()
        unique_parts = []
        for part in parts:
            # Case-insensitive deduplication
            part_lower = part.lower()
            if part_lower not in seen and part:  # Skip empty parts
                seen.add(part_lower)
                unique_parts.append(part)
        
        return ", ".join(unique_parts)
    
    def _filter_calls_by_date(self, calls: List[Dict[str, Any]], days_back: int) -> List[Dict[str, Any]]:
        """
        Filter calls to only include those from the specified number of days back.
        
        Args:
            calls: List of call dictionaries
            days_back: Number of days back to include
            
        Returns:
            Filtered list of calls
        """
        if not calls or days_back <= 0:
            return calls
        
        # Calculate cutoff date
        cutoff_date = datetime.now() - timedelta(days=days_back)
        
        filtered_calls = []
        for call in calls:
            call_date_str = call.get("userTimezoneActivityTime") or call.get("date") or call.get("started_at")
            if not call_date_str:
                # Keep calls without dates to be safe
                filtered_calls.append(call)
                continue
            
            try:
                # Try to parse the date string
                for fmt in [
                    "%Y/%m/%d %H:%M:%S",      # Gong userTimezoneActivityTime format
                    "%Y-%m-%dT%H:%M:%S.%fZ",  # ISO with microseconds
                    "%Y-%m-%dT%H:%M:%SZ",     # ISO without microseconds
                    "%Y-%m-%d %H:%M:%S",      # Standard format
                    "%Y-%m-%d",               # Date only
                ]:
                    try:
                        call_date = datetime.strptime(call_date_str, fmt)
                        if call_date >= cutoff_date:
                            filtered_calls.append(call)
                        break
                    except ValueError:
                        continue
                else:
                    # Couldn't parse date, keep the call to be safe
                    filtered_calls.append(call)
                    
            except Exception:
                # Error parsing, keep the call
                filtered_calls.append(call)
        
        logger.debug(f"Filtered calls: {len(calls)} -> {len(filtered_calls)} (last {days_back} days)")
        return filtered_calls


class CallDetailsFetcher:
    """Fetches detailed call information using existing pipeline tools."""
    
    def __init__(self, 
                 http_client: HTTPClientPool,
                 auth: GongAuthenticator,
                 config: Optional[GongConfig] = None):
        self.http = http_client
        self.auth = auth
        self.config = config
        
    async def get_call_details(self, call_id: str) -> Optional[Dict[str, Any]]:
        """
        Get detailed call information including transcript.
        
        Args:
            call_id: Gong call ID
            
        Returns:
            Detailed call information or None if failed
        """
        try:
            # Get dynamic base URL from authenticator  
            base_url = self.auth.get_base_url()
            url = f"{base_url}/call/detailed-transcript"
            params = {"call-id": call_id}
            
            logger.debug(f"Fetching call details for {call_id}")
            
            # Get authenticated headers
            headers = await self.auth.get_authenticated_headers(retry_on_failure=True)
            
            response = await self.http.get(url, params=params, headers=headers)
            
            if response.status_code == 200:
                data = response.json()
                return self._extract_call_details(data, call_id)
            else:
                logger.warning(f"Failed to fetch call details for {call_id}",
                             status_code=response.status_code)
                return None
                
        except Exception as e:
            logger.error(f"Error fetching call details for {call_id}: {e}")
            return None
    
    def _extract_call_details(self, api_response: Dict[str, Any], call_id: str) -> Dict[str, Any]:
        """Extract and structure call details from API response."""
        
        # Debug: Log the actual API response structure
        logger.debug(f"Call details API response keys for {call_id}: {list(api_response.keys())}")
        
        # Extract basic call info
        call_info = {
            "id": call_id,
            "title": "",
            "customer_name": "",
            "date": "",
            "attendees": [],
            "transcript": ""
        }
        
        # Navigate the actual API response structure
        if api_response:
            # Extract title
            if "callTitle" in api_response:
                call_info["title"] = api_response["callTitle"]
            
            # Extract customer name from callCustomers
            if "callCustomers" in api_response and api_response["callCustomers"]:
                customers = api_response["callCustomers"]
                if isinstance(customers, list) and len(customers) > 0:
                    # Handle list of customer objects
                    customer = customers[0]
                    if isinstance(customer, dict):
                        call_info["customer_name"] = customer.get("name", "")
                    elif isinstance(customer, str):
                        call_info["customer_name"] = customer
                elif isinstance(customers, str):
                    call_info["customer_name"] = customers
            
            # Extract date
            if "when" in api_response:
                call_info["date"] = api_response["when"]
            
            # Extract attendees from multiple participant categories
            attendees = []
            
            # Company participants (internal team)
            if "companyParticipants" in api_response:
                company_participants = api_response["companyParticipants"]
                if company_participants:
                    # Handle both list and dict structures
                    if isinstance(company_participants, dict):
                        # Flatten dictionary structure
                        participants_list = []
                        for company_name, participants in company_participants.items():
                            if isinstance(participants, list):
                                participants_list.extend(participants)
                        logger.debug(f"Company participants (dict): {len(participants_list)} total from {len(company_participants)} companies")
                    else:
                        participants_list = company_participants
                        logger.debug(f"Company participants (list): {len(participants_list)} participants")
                    
                    for participant in participants_list:
                        if isinstance(participant, dict):
                            name = self._extract_participant_name(participant)
                            if name:
                                attendee = {
                                    "name": name,
                                    "title": participant.get("title", ""),
                                    "company": api_response.get("callCompanyName", ""),
                                    "email": participant.get("emailAddress", "")
                                }
                                attendees.append(attendee)
            
            # Customer participants (client attendees)
            if "customerParticipants" in api_response:
                customer_participants = api_response["customerParticipants"]
                if customer_participants:
                    # Handle both list and dict structures
                    if isinstance(customer_participants, dict):
                        # Flatten dictionary structure
                        participants_list = []
                        for company_name, participants in customer_participants.items():
                            if isinstance(participants, list):
                                # Add company name to each participant for context
                                for participant in participants:
                                    if isinstance(participant, dict):
                                        participant = dict(participant)  # Copy to avoid modifying original
                                        if not participant.get("companyName"):
                                            participant["companyName"] = company_name
                                        participants_list.append(participant)
                        logger.debug(f"Customer participants (dict): {len(participants_list)} total from {len(customer_participants)} companies")
                    else:
                        participants_list = customer_participants
                        logger.debug(f"Customer participants (list): {len(participants_list)} participants")
                    
                    for participant in participants_list:
                        if isinstance(participant, dict):
                            name = self._extract_participant_name(participant)
                            if name:
                                attendee = {
                                    "name": name,
                                    "title": participant.get("title", ""),
                                    "company": participant.get("companyName", ""),
                                    "email": participant.get("emailAddress", "")
                                }
                                attendees.append(attendee)
            
            # Unknown participants
            if "unknownParticipants" in api_response:
                for participant in api_response["unknownParticipants"]:
                    name = self._extract_participant_name(participant)
                    if name:
                        attendee = {
                            "name": name,
                            "title": participant.get("title", ""),
                            "company": "",
                            "email": participant.get("emailAddress", "")
                        }
                        attendees.append(attendee)
            
            call_info["attendees"] = attendees
            
            # Extract and clean transcript from monologues
            if "monologues" in api_response and api_response["monologues"]:
                transcript_parts = []
                monologues = api_response["monologues"]
                
                for monologue in monologues:
                    speaker_name = "Unknown Speaker"
                    
                    # Try to get speaker name - check multiple possible fields
                    if "speakerName" in monologue and monologue["speakerName"]:
                        speaker_name = monologue["speakerName"]
                    elif "speakerShortName" in monologue:
                        short_name = monologue["speakerShortName"]
                        # Look up full name using shortNamesLookup
                        if "shortNamesLookup" in api_response:
                            lookup = api_response["shortNamesLookup"]
                            if short_name in lookup:
                                speaker_name = lookup[short_name].get("name", short_name)
                            else:
                                speaker_name = short_name
                    
                    # Get the monologue text - check multiple possible structures
                    monologue_text = None
                    
                    # Try direct text field first
                    if "text" in monologue and monologue["text"]:
                        monologue_text = monologue["text"].strip()
                    
                    # Fallback to sentences structure  
                    elif "sentences" in monologue:
                        sentences = []
                        for sentence in monologue["sentences"]:
                            if isinstance(sentence, dict) and "text" in sentence:
                                sentences.append(sentence["text"])
                            elif isinstance(sentence, str):
                                sentences.append(sentence)
                        
                        if sentences:
                            monologue_text = " ".join(sentences)
                    
                    # Add to transcript if we have text
                    if monologue_text:
                        transcript_parts.append(f"**{speaker_name}:** {monologue_text}")
                
                if transcript_parts:
                    call_info["transcript"] = "\n\n".join(transcript_parts)
                    logger.debug(f"Extracted transcript with {len(transcript_parts)} monologues")
        
        return call_info
    
    def _extract_participant_name(self, participant: Dict[str, Any]) -> str:
        """
        Extract participant name from various possible name fields.
        
        Args:
            participant: Participant data dictionary
            
        Returns:
            Full name string or empty string if no name found
        """
        # Try different name field combinations
        first_name = participant.get("firstName", "").strip()
        last_name = participant.get("lastName", "").strip()
        
        # Full name field
        if "name" in participant and participant["name"]:
            return participant["name"].strip()
        
        # First and last name combination
        if first_name and last_name:
            return f"{first_name} {last_name}"
        elif first_name:
            return first_name
        elif last_name:
            return last_name
        
        # Try other possible name fields
        if "fullName" in participant and participant["fullName"]:
            return participant["fullName"].strip()
        
        if "displayName" in participant and participant["displayName"]:
            return participant["displayName"].strip()
        
        return ""