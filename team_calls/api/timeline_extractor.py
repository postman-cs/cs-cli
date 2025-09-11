"""Timeline extraction with advanced BDR/SPAM/Automation filtering."""

import asyncio
import re
from collections import defaultdict
from datetime import datetime
from typing import Dict, List, Optional, Tuple

import structlog

from ..infra.auth import GongAuthenticator
from ..infra.config import GongConfig
from ..infra.http import HTTPClientPool
from ..models import (
    Call,
    CallDirection,
    CallParticipant,
    Email,
    EmailDirection,
    EmailRecipient,
    ExtractionRange,
)

logger = structlog.get_logger()

# Pre-compiled regex patterns for better performance
RE_THREAD_PREFIX = re.compile(r"\b(re|fwd):\s*", re.IGNORECASE)
RE_URL = re.compile(r"https?://\S+")
RE_GREETING = re.compile(r"\b(hi|hello|hey|dear)\s+[a-z]+[,.-]?\s*", re.IGNORECASE)
RE_COMPANY_AT = re.compile(r"\bat\s+[a-z]+\s+with\b", re.IGNORECASE)
RE_ACCOUNT_MANAGER = re.compile(r"\b[a-z]+\'s\s+account\s+manager\b", re.IGNORECASE)
RE_USERS_NUMBER = re.compile(r"\b\d+\.?\d*\s+users?\b", re.IGNORECASE)
RE_WITH_USERS = re.compile(r"\bwith\s+\d+\.?\d*\s+users\b", re.IGNORECASE)
RE_BIG_NUMBERS = re.compile(r"\b\d{2,}\b")
RE_NAME_IS = re.compile(r"\bmy name is\s+[a-z]+\b", re.IGNORECASE)
RE_I_AM = re.compile(r"\bi am\s+[a-z]+\'s\b", re.IGNORECASE)
RE_DATE_SLASH = re.compile(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b")
RE_DATE_MONTH = re.compile(
    r"\b(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2}(st|nd|rd|th)?\b",
    re.IGNORECASE,
)
RE_WHITESPACE = re.compile(r"\s+")

# CRITICAL: Pre-compiled patterns for BDR/SPAM/Automation filtering
TEMPLATE_MARKERS = frozenset([
    "following up",
    "circling back", 
    "resurfacing",
    "touching base",
    "checking in",
    "reaching out",
    "quick follow-up",
    "just checking",
    "wanted to circle back",
    "hope this finds you well",
    # Additional sales/BDR patterns
    "quick chat",
    "15 minutes",
    "quick question",
    "calendar",
    "schedule time",
    "appropriate person",
    "intro call",
    "connect you with",
    "demo",
    "case study",
    "pilot",
    "free trial",
    "webinar",
    "whitepaper",
    "any interest",
    "worth a chat",
    "following up on my previous",
    "bumping this",
    "any thoughts",
    "quick sync",
    "hop on a call",
    "brief call",
    "quick call",
    "short call",
    "find time",
    "available this week",
    "available next week",
    "few minutes to chat",
    # Common reminder patterns
    "per my last note",
    "per my last email",
    "as mentioned previously",
    "quick reminder",
    # Auto-reply and out-of-office patterns
    "automatic reply",
    "out of office",
    "out-of-office",
    "ooo",
    "will be out",
    "out of the office",
    "returning on",
    "limited access to email",
    "urgent matters",
])


class TimelineExtractor:
    """Extract account timeline with sophisticated email filtering."""

    def __init__(
        self,
        http_client: HTTPClientPool,
        auth: GongAuthenticator,
        config: Optional[GongConfig] = None,
        chunk_days: int = 30,
    ) -> None:
        self.http = http_client
        self.auth = auth
        self.config = config
        self.chunk_days = chunk_days
        
        # Filtering statistics (reset per extraction batch)
        self.filtered_stats = {
            "similarity_filtered": 0,
            "bdr_filtered": 0,
            "total_filtered": 0,
            "calls_filtered": 0,
            "calls_duration_filtered": 0,
            "noise_filtered": 0,
        }
        
        logger.info("Timeline extractor initialized with advanced filtering")

    async def extract_account_timeline(
        self, account_id: str, start_date: datetime, end_date: Optional[datetime] = None
    ) -> Tuple[List[Call], List[Email]]:
        """Extract all communications for an account within date range."""
        if end_date is None:
            end_date = datetime.now()

        logger.info(
            "Extracting timeline",
            account_id=account_id,
            start=start_date.isoformat(),
            end=end_date.isoformat(),
        )

        # Create date range chunks
        date_range = ExtractionRange(start_date=start_date, end_date=end_date)
        chunks = date_range.to_chunks(self.chunk_days)

        logger.debug("Timeline chunked", chunks=len(chunks))

        # Fetch all chunks concurrently
        tasks = [self._fetch_chunk(account_id, chunk) for chunk in chunks]
        chunk_results = await asyncio.gather(*tasks, return_exceptions=True)

        # Aggregate results
        all_calls = []
        all_emails = []

        for result in chunk_results:
            if isinstance(result, Exception):
                logger.error("Chunk failed", error=str(result))
                continue

            calls, emails = result
            all_calls.extend(calls)
            all_emails.extend(emails)

        # Sort by date
        all_calls.sort(key=lambda c: c.scheduled_start or datetime.min)
        all_emails.sort(key=lambda e: e.sent_at or datetime.min)

        logger.info(
            "Timeline extracted",
            account_id=account_id,
            calls=len(all_calls),
            emails=len(all_emails),
        )

        return all_calls, all_emails

    async def _fetch_chunk(
        self, account_id: str, chunk: ExtractionRange
    ) -> Tuple[List[Call], List[Email]]:
        """Fetch a single timeline chunk with filtering."""
        base_url = self.auth.get_base_url()
        
        # Get API parameters from chunk
        params = chunk.to_api_params(account_id)
        
        # Add required workspace parameters
        workspace_id = self.auth.get_workspace_id_sync() or "5562739194953732039"  # Dynamic or default workspace ID
        team_id = "5359555372180789967"  # Default team ID
        
        if self.config and hasattr(self.config, 'extraction'):
            workspace_id = getattr(self.config.extraction, 'workspace_id', workspace_id)
            team_id = getattr(self.config.extraction, 'team_id', team_id)
        
        params.update({
            "workspace-id": workspace_id,
            "team-id": team_id
        })

        endpoint = f"{base_url}/ajax/account/day-activities"
        headers = await self.auth.get_read_headers()

        try:
            response = await self.http.get(endpoint, params=params, headers=headers)

            if response.status_code != 200:
                if response.status_code in [401, 403]:
                    await self.auth.handle_auth_error(response.status_code, is_post_request=False)
                logger.error(
                    "Chunk fetch failed", account_id=account_id, status=response.status_code
                )
                return [], []

            data = response.json()

            # Extract activities from date-based results
            activities = []
            date_pattern = re.compile(r"^\d{4}-\d{2}-\d{2}$")

            for key, value in data.items():
                if date_pattern.match(key) and isinstance(value, list):
                    for activity in value:
                        if isinstance(activity, dict) and activity.get("type") in ["EMAIL", "CALL"]:
                            # Add date to activity if not present
                            if "date" not in activity:
                                activity["date"] = key
                            activities.append(activity)

            # Parse activities with advanced filtering
            calls = []
            emails = []
            email_activities = []

            # Parse calls and collect email activities
            for activity in activities:
                activity_type = activity.get("type")

                if activity_type in ["CALL", "call"]:
                    call = self._parse_call(activity, account_id)
                    if call and self._should_include_call(call):
                        calls.append(call)
                elif activity_type in ["EMAIL", "email"]:
                    email_activities.append((activity, account_id))

            # Process emails with advanced filtering
            if email_activities:
                raw_emails = self._process_emails_with_filtering(email_activities)
                emails = [e for e in raw_emails if not e.is_automated]

            return calls, emails

        except Exception as e:
            logger.error(
                "Chunk extraction error", account_id=account_id, chunk=params, error=str(e)
            )
            return [], []

    def _should_include_call(self, call: Call) -> bool:
        """Call filtering logic - match behavior of direct customer search."""
        # Include all calls with basic validity checks to match option 1 behavior
        return (
            call is not None and 
            hasattr(call, "id") and call.id is not None and
            hasattr(call, "title") and call.title is not None
        )


    def _process_emails_with_filtering(self, email_activities: List[Tuple[dict, str]]) -> List[Email]:
        """
        Process emails with advanced BDR/SPAM/Automation filtering.
        This is the CORE filtering logic that removes noise.
        """
        if not email_activities:
            return []

        # Group activities by sender for context-aware processing
        sender_groups = defaultdict(list)
        for activity, account_id in email_activities:
            sender_email = self._extract_sender_email(activity)
            if sender_email:
                sender_groups[sender_email].append((activity, account_id))

        # Process each sender group with full similarity context
        all_emails = []
        for sender_email, activities in sender_groups.items():
            sender_emails = self._process_sender_emails(activities, sender_email)
            all_emails.extend(sender_emails)

        # Apply additional filtering by synopsis to remove duplicates and mass emails
        if all_emails:
            # Convert to dict format for filtering
            email_dicts = [
                {
                    "id": email.id,
                    "subject": email.subject,
                    "snippet": email.snippet,
                    "sent_at": email.sent_at,
                    "sender": {"email": email.sender.email, "title": email.sender.title or ""},
                }
                for email in all_emails
            ]

            # Apply advanced synopsis filtering
            filtered_dicts, filter_stats = self._filter_emails_by_synopsis(email_dicts)
            filtered_ids = {e["id"] for e in filtered_dicts}
            filtered_emails = [e for e in all_emails if e.id in filtered_ids]

            logger.info(
                "Email filtering applied",
                original=len(all_emails),
                filtered=len(filtered_emails),
                removed=len(all_emails) - len(filtered_emails),
                filter_stats=filter_stats
            )

            return filtered_emails

        return all_emails

    def _extract_sender_email(self, activity: dict) -> Optional[str]:
        """Extract sender email from activity data."""
        extended_data = activity.get("extendedData", {})
        from_data = extended_data.get("from", extended_data.get("byPerson", {}))
        return from_data.get("email")

    def _process_sender_emails(
        self, activities: List[Tuple[dict, str]], sender_email: str
    ) -> List[Email]:
        """Process all emails from a single sender with automation detection."""
        # Parse all emails from this sender
        sender_email_data = []
        for activity, account_id in activities:
            email_data = self._parse_email_basic(activity, account_id)
            if email_data:
                sender_email_data.append(email_data)

        # Apply automation detection with similarity context
        processed_emails = []
        for i, email_data in enumerate(sender_email_data):
            subject = email_data["subject"]
            snippet = email_data["snippet"]
            
            # CRITICAL: Use unified automation detection
            sender_title = email_data.get("sender_title", "")
            is_automated, is_template = self._is_automated_content(subject, snippet, sender_email, sender_title)

            # Check similarity against other emails from same sender
            if not is_automated and len(sender_email_data) > 1:
                for j, other_email_data in enumerate(sender_email_data):
                    if i != j:  # Avoid self-comparison
                        subject_sim = self._similarity_score(subject, other_email_data["subject"])
                        snippet_sim = self._similarity_score(snippet, other_email_data["snippet"])
                        max_similarity = max(subject_sim, snippet_sim)

                        if max_similarity >= 0.95:  # High similarity threshold
                            is_automated = True
                            is_template = True
                            break

            # Create Email object
            email = Email(
                emailId=email_data["activity_id"],
                accountId=email_data["account_id"],
                threadId=None,
                subject=subject,
                direction=email_data["direction"],
                sentAt=email_data["sent_at"],
                sender=email_data["sender"],
                recipients=email_data["recipients"],
                body_text=None,
                snippet=snippet,
                is_automated=is_automated,
                is_template=is_template,
            )
            processed_emails.append(email)

        return processed_emails

    def _is_automated_content(
        self, subject: str, snippet: str, sender_email: str, sender_title: str = ""
    ) -> Tuple[bool, bool]:
        """
        CRITICAL: Unified automation and template detection.
        Returns: (is_automated, is_template)
        """
        if not subject and not snippet:
            return False, False

        # Check for specific filtered senders and roles (highest confidence)
        sender_email_lower = sender_email.lower()
        sender_title_lower = sender_title.lower()
        
        # Filter out sales@postman.com
        if sender_email_lower == "sales@postman.com":
            logger.debug("Filtering out email from sales@postman.com", sender_email=sender_email)
            return True, True
        
        # Filter out anyone with "Account Development" in their title
        if "account development" in sender_title_lower:
            logger.debug("Filtering out email from Account Development role", 
                       sender_email=sender_email, sender_title=sender_title)
            return True, True

        # Check for known automated senders (highest confidence)
        automated_domains = {"academy@postman.com", "help@postman.com", "noreply@", "no-reply@"}
        if any(domain in sender_email_lower for domain in automated_domains):
            return True, True

        # Check for auto-reply patterns (high-confidence automation)
        subject_lower = subject.lower() if subject else ""
        auto_reply_patterns = [
            "automatic reply:",
            "out-of-office",
            "out of office",
            "ooo ",
            "paternity leave",
            "maternity leave"
        ]
        
        if any(pattern in subject_lower for pattern in auto_reply_patterns):
            return True, True

        # Check for template language markers
        content_lower = f"{subject} {snippet}".lower()
        has_template_markers = any(marker in content_lower for marker in TEMPLATE_MARKERS)

        return has_template_markers, has_template_markers

    def _similarity_score(self, text1: str, text2: str) -> float:
        """Calculate text similarity using efficient Jaccard similarity."""
        if not text1 or not text2:
            return 0.0

        # Use normalized text for better comparison
        words1 = set(text1.lower().split())
        words2 = set(text2.lower().split())

        if not words1 or not words2:
            return 0.0

        intersection = len(words1 & words2)
        union = len(words1 | words2)

        return intersection / union if union > 0 else 0.0

    def _normalize_synopsis(self, text: str) -> str:
        """Enhanced normalization to detect template variations with personalization."""
        if not text:
            return ""

        t = text.lower()

        # Use precompiled regex patterns for better performance
        t = RE_THREAD_PREFIX.sub("", t)
        t = RE_URL.sub(" URL ", t)
        t = RE_GREETING.sub(" GREETING ", t)
        t = RE_COMPANY_AT.sub(" at COMPANY with ", t)
        t = RE_ACCOUNT_MANAGER.sub(" COMPANY account manager ", t)
        t = RE_USERS_NUMBER.sub(" NUM users ", t)
        t = RE_WITH_USERS.sub(" with NUM users ", t)
        t = RE_BIG_NUMBERS.sub(" NUM ", t)
        t = RE_NAME_IS.sub(" my name is NAME ", t)
        t = RE_I_AM.sub(" i am COMPANY ", t)
        t = RE_DATE_SLASH.sub(" DATE ", t)
        t = RE_DATE_MONTH.sub(" DATE ", t)
        t = RE_WHITESPACE.sub(" ", t).strip()

        return t

    def _filter_emails_by_synopsis(self, emails: List[dict]) -> Tuple[List[dict], Dict[str, int]]:
        """
        CRITICAL: Filter emails based on synopsis to remove duplicates and mass emails.
        Returns: (filtered_emails, filter_stats)
        """
        if not emails:
            return [], {"total": 0}

        filtered_emails = []
        similarity_filtered = 0
        template_filtered = 0

        # Group emails by sender
        sender_groups = defaultdict(list)
        for email in emails:
            sender_email = email.get("sender", {}).get("email", "").lower()
            sender_groups[sender_email].append(email)

        # Process each sender group
        for sender_email, sender_emails in sender_groups.items():
            # Check for high-volume template senders
            template_count = sum(
                1
                for email in sender_emails
                if self._is_automated_content(
                    email.get("subject", ""), 
                    email.get("snippet", ""), 
                    sender_email,
                    email.get("sender", {}).get("title", "")
                )[1]  # Use is_template result
            )

            template_rate = template_count / len(sender_emails) if sender_emails else 0
            is_high_template_sender = len(sender_emails) >= 5 and template_rate >= 0.7

            if is_high_template_sender:
                # Keep only the best representative for template senders
                representative = self._select_blast_representative(sender_emails)
                filtered_emails.append(representative)
                template_filtered += len(sender_emails) - 1
                continue

            # For normal senders, group by content similarity
            email_groups = self._group_emails_by_content_similarity(sender_emails, threshold=0.85)

            for group in email_groups:
                if len(group) > 1 and self._is_blast_pattern(group):
                    # Keep one representative for similar content blasts
                    representative = self._select_blast_representative(group)
                    filtered_emails.append(representative)
                    similarity_filtered += len(group) - 1
                else:
                    # Keep all emails for conversations and unique content
                    filtered_emails.extend(group)

        # Update tracking stats
        total_filtered = similarity_filtered + template_filtered
        self.filtered_stats["similarity_filtered"] += similarity_filtered
        self.filtered_stats["bdr_filtered"] += template_filtered
        self.filtered_stats["total_filtered"] += total_filtered

        return filtered_emails, {
            "similarity": similarity_filtered,
            "template_mass": template_filtered,
            "total": total_filtered,
        }

    def _group_emails_by_content_similarity(
        self, emails: List[dict], threshold: float = 0.85
    ) -> List[List[dict]]:
        """Group emails by content similarity to detect blast patterns."""
        if not emails:
            return []

        groups = []
        processed_indices = set()

        for i, email in enumerate(emails):
            if i in processed_indices:
                continue

            # Start a new group with this email
            group = [email]
            processed_indices.add(i)

            email_content = (
                (email.get("snippet", "") or "") + " " + (email.get("subject", "") or "")
            )
            normalized_content = self._normalize_synopsis(email_content)

            # Find similar emails
            for j, other_email in enumerate(emails):
                if j in processed_indices or i == j:
                    continue

                other_content = (
                    (other_email.get("snippet", "") or "")
                    + " "
                    + (other_email.get("subject", "") or "")
                )
                normalized_other = self._normalize_synopsis(other_content)

                # Use normalized content for better template detection
                similarity = self._similarity_score(normalized_content, normalized_other)
                if similarity >= threshold:
                    group.append(other_email)
                    processed_indices.add(j)

            groups.append(group)

        return groups

    def _is_blast_pattern(self, email_group: List[dict]) -> bool:
        """Detect if email group represents a blast pattern."""
        if len(email_group) <= 1:
            return False

        # Parse timestamps
        timestamps = []
        for email in email_group:
            sent_at = email.get("sent_at")
            if isinstance(sent_at, datetime):
                timestamps.append(sent_at)

        if len(timestamps) >= 2:
            # Blast pattern: emails sent within 24 hours
            time_span = max(timestamps) - min(timestamps)
            return time_span.total_seconds() <= 86400  # 24 hours

        # Default: multiple similar emails likely indicate blast pattern
        return len(email_group) >= 2

    def _select_blast_representative(self, blast_group: List[dict]) -> dict:
        """Select the best representative email from a blast group."""
        if len(blast_group) == 1:
            return blast_group[0]

        # Priority: Prefer emails with longer content (more context)
        best_email = max(
            blast_group,
            key=lambda e: len((e.get("snippet", "") or "") + (e.get("subject", "") or "")),
        )

        return best_email

    def _parse_call(self, activity: dict, account_id: str) -> Optional[Call]:
        """Parse call activity into Call model."""
        try:
            extended_data = activity.get("extendedData", {})
            
            # Debug: Log available fields to identify correct call ID
            logger.debug("Timeline call activity fields", 
                        activity_keys=list(activity.keys()),
                        extended_data_keys=list(extended_data.keys()),
                        activity_id=activity.get("id"))
            
            # Convert epoch time to datetime with fallbacks
            epoch_time = activity.get("epochTime")
            if epoch_time:
                scheduled_start = datetime.fromtimestamp(epoch_time)
            else:
                # Fallback to effectiveDateTime or date field
                date_str = activity.get("effectiveDateTime") or activity.get("date")
                if date_str:
                    try:
                        scheduled_start = datetime.strptime(date_str, "%Y-%m-%d")
                    except (ValueError, TypeError):
                        scheduled_start = None
                        logger.warning("Could not parse date", date_str=date_str)
                else:
                    scheduled_start = None
            
            logger.debug("Timeline date extraction", 
                        epoch_time=epoch_time,
                        effective_date=activity.get("effectiveDateTime"),
                        scheduled_start=scheduled_start)

            # Parse participants
            participants = []
            participant_emails = set()

            # Add detailed participant from extendedData.byPerson if available
            by_person = extended_data.get("byPerson", {})
            if by_person.get("email"):
                email = by_person.get("email")
                participants.append(
                    CallParticipant(
                        name=by_person.get("name", email.split("@")[0]),
                        email=email,
                        title=by_person.get("title"),
                        company=by_person.get("companyName"),
                        is_internal=False,
                    )
                )
                participant_emails.add(email)

            # Add remaining participants from email list
            for email in activity.get("participantsEmailList", []):
                if email not in participant_emails:
                    participants.append(
                        CallParticipant(name=email.split("@")[0], email=email, is_internal=False)
                    )
                    participant_emails.add(email)

            # Determine direction
            direction = CallDirection.UNKNOWN
            dir_str = activity.get("direction", "").upper()
            if dir_str in CallDirection.__members__:
                direction = CallDirection[dir_str]

            # Try multiple fields for call ID like customer search does
            call_id = (activity.get("id") or 
                      activity.get("callId") or 
                      activity.get("call_id") or
                      extended_data.get("callId") or
                      extended_data.get("id"))
            
            logger.debug("Timeline call ID extraction", 
                        activity_id=activity.get("id"),
                        extracted_call_id=call_id,
                        activity_type=activity.get("type"))
            
            call = Call(
                callId=call_id,
                accountId=account_id,
                title=extended_data.get("title", extended_data.get("contentTitle", "Call")),
                direction=direction,
                duration=int(extended_data.get("duration", 0)) if extended_data.get("duration") else 0,
                scheduledStart=scheduled_start,
                actualStart=None,
                recordingUrl=extended_data.get("recordingUrl"),
                transcriptUrl=None,
                callBrief=extended_data.get("callBrief") or extended_data.get("contentTitle"),
                participants=participants,
                hostId=None,
                hostName=None,
                status=activity.get("status"),
                call_type=extended_data.get("callType"),
            )

            return call

        except Exception as e:
            logger.error("Call parsing error", error=str(e), activity_id=activity.get("id"))
            return None

    def _parse_email_basic(self, activity: dict, account_id: str) -> Optional[dict]:
        """Parse basic email data from activity."""
        try:
            extended_data = activity.get("extendedData", {})

            # Convert epoch time to datetime
            epoch_time = activity.get("epochTime")
            sent_at = datetime.fromtimestamp(epoch_time) if epoch_time else None

            # Parse sender
            from_data = extended_data.get("from", extended_data.get("byPerson", {}))
            sender = EmailRecipient(
                email=from_data.get("email", "unknown@example.com"),
                name=from_data.get("name", "Unknown"),
                type="from",
                is_internal="postman.com" in from_data.get("email", "").lower(),
                title=from_data.get("title"),
                company=from_data.get("companyName"),
            )

            # Parse recipients
            recipients = []
            to_list = extended_data.get("to", [])

            if to_list:
                for recipient_data in to_list:
                    recipient = EmailRecipient(
                        email=recipient_data.get("email", ""),
                        name=recipient_data.get("name", recipient_data.get("email", "").split("@")[0]),
                        type="to",
                        is_internal="postman.com" in recipient_data.get("email", "").lower(),
                        title=recipient_data.get("title"),
                        company=recipient_data.get("companyName"),
                    )
                    recipients.append(recipient)
            else:
                # Fallback to participantsEmailList
                participants_emails = activity.get("participantsEmailList", [])
                for email_addr in participants_emails:
                    if email_addr != sender.email:
                        recipient = EmailRecipient(
                            email=email_addr,
                            name=email_addr.split("@")[0],
                            type="to",
                            is_internal="postman.com" in email_addr.lower(),
                        )
                        recipients.append(recipient)

            # Determine direction
            direction = EmailDirection.INBOUND
            if sender.is_internal:
                direction = EmailDirection.OUTBOUND
            elif any(r.is_internal for r in recipients):
                direction = EmailDirection.INBOUND
            else:
                direction = EmailDirection.INTERNAL

            # Extract subject and snippet
            subject = extended_data.get("subject") or extended_data.get("contentTitle", "No Subject")
            snippet = extended_data.get("synopsis") or extended_data.get("categoryPassiveVoice", "")
            
            # Get the correct account ID from the activity
            email_account_id = activity.get("accountId") or account_id
            email_id = activity.get("id")

            return {
                "activity_id": email_id,
                "account_id": email_account_id,  # Use activity's accountId first
                "subject": subject,
                "snippet": snippet,
                "sender": sender,
                "sender_email": sender.email,
                "sender_title": sender.title or "",
                "recipients": recipients,
                "direction": direction,
                "sent_at": sent_at,
                "extended_data": extended_data,
            }

        except Exception as e:
            logger.error("Email basic parsing error", error=str(e), activity_id=activity.get("id"))
            return None
