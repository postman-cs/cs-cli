"""Data models for cs-transcript-cli with email support."""

from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Optional, List

from pydantic import BaseModel, ConfigDict, Field


class CommunicationType(str, Enum):
    """Communication types in Gong."""
    CALL = "call"
    EMAIL = "email"
    MEETING = "meeting"
    CHAT = "chat"
    SMS = "sms"


class CallDirection(str, Enum):
    """Call direction types."""
    INBOUND = "INBOUND"
    OUTBOUND = "OUTBOUND"
    INTERNAL = "INTERNAL"
    UNKNOWN = "UNKNOWN"


class EmailDirection(str, Enum):
    """Email direction types."""
    INBOUND = "inbound"
    OUTBOUND = "outbound"
    INTERNAL = "internal"


class CallParticipant(BaseModel):
    """Call participant details."""
    id: Optional[str] = None
    name: str
    email: Optional[str] = None
    phone: Optional[str] = None
    title: Optional[str] = None
    company: Optional[str] = None
    is_internal: bool = False
    speaking_time: Optional[float] = None
    talk_ratio: Optional[float] = None


class Call(BaseModel):
    """Call communication model."""
    id: str = Field(alias="callId")
    account_id: str = Field(alias="accountId")
    title: str
    direction: CallDirection
    duration: int  # seconds
    scheduled_start: datetime = Field(alias="scheduledStart")
    actual_start: Optional[datetime] = Field(None, alias="actualStart")
    recording_url: Optional[str] = Field(None, alias="recordingUrl")
    transcript_url: Optional[str] = Field(None, alias="transcriptUrl")
    call_brief: Optional[str] = Field(None, alias="callBrief")

    # Recording status indicators (from timeline activity)
    status: Optional[str] = None  # e.g., "COMPLETED" for recorded calls
    call_type: Optional[str] = None  # e.g., "WEB_CONFERENCE" for recorded calls

    # Participants
    participants: List[CallParticipant] = Field(default_factory=list)
    host_id: Optional[str] = Field(None, alias="hostId")
    host_name: Optional[str] = Field(None, alias="hostName")

    # Analytics
    sentiment: Optional[float] = None
    talk_ratio: Optional[float] = None
    longest_monologue: Optional[int] = None
    interactivity: Optional[float] = None
    questions_asked: Optional[int] = None

    # Content
    transcript: Optional[str] = None
    summary: Optional[str] = None
    topics: List[str] = Field(default_factory=list)
    action_items: List[str] = Field(default_factory=list)

    model_config = ConfigDict(populate_by_name=True)


class EmailRecipient(BaseModel):
    """Email recipient details."""
    email: str
    name: Optional[str] = None
    type: str = "to"  # to, cc, bcc, from
    is_internal: bool = False
    title: Optional[str] = None
    company: Optional[str] = None


class Email(BaseModel):
    """Email communication model."""
    id: str = Field(alias="emailId")
    account_id: str = Field(alias="accountId")
    thread_id: Optional[str] = Field(None, alias="threadId")
    subject: str
    direction: EmailDirection
    sent_at: datetime = Field(alias="sentAt")

    # Participants
    sender: EmailRecipient
    recipients: List[EmailRecipient] = Field(default_factory=list)

    # Content
    body_text: Optional[str] = None
    snippet: Optional[str] = None

    # Metadata
    is_automated: bool = False
    is_template: bool = False
    template_id: Optional[str] = None
    campaign_id: Optional[str] = None
    sequence_id: Optional[str] = None

    # Analytics
    open_count: int = 0
    click_count: int = 0
    reply_count: int = 0
    sentiment: Optional[float] = None

    # Vector embeddings for search
    embedding: Optional[List[float]] = None

    model_config = ConfigDict(populate_by_name=True)


class ExtractionRange(BaseModel):
    """Date range for extraction."""
    start_date: datetime
    end_date: datetime

    def to_chunks(self, days: int = 30) -> List["ExtractionRange"]:
        """Split range into chunks of specified days."""
        chunks = []
        current = self.start_date

        while current < self.end_date:
            chunk_end = min(current + timedelta(days=days), self.end_date)
            chunks.append(ExtractionRange(start_date=current, end_date=chunk_end))
            current = chunk_end

        return chunks

    def to_api_params(self, account_id: str) -> dict[str, str]:
        """Convert to API query parameters for day-activities endpoint."""
        return {
            "id": account_id,
            "type": "ACCOUNT",
            "day-from": self.start_date.strftime("%Y-%m-%d"),
            "day-to": self.end_date.strftime("%Y-%m-%d"),
        }


class Communication(BaseModel):
    """Generic communication wrapper."""
    type: CommunicationType
    id: str
    account_id: str
    occurred_at: datetime
    participants_count: int

    # Type-specific data
    call: Optional[Call] = None
    email: Optional[Email] = None

    # Common fields
    title: str
    summary: Optional[str] = None
    sentiment: Optional[float] = None
    is_internal: bool = False

    # Caching metadata
    fetched_at: datetime = Field(default_factory=datetime.now)
    enhanced: bool = False  # Whether full content has been fetched

    model_config = ConfigDict(populate_by_name=True)
