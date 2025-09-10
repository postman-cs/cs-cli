"""Markdown formatter for team call reports."""

import re
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional

import structlog
from ..models import Email

logger = structlog.get_logger()


class CallMarkdownFormatter:
    """Formats call data into markdown reports."""
    
    def __init__(self, output_dir: Optional[Path] = None):
        self.output_dir = output_dir or Path("team-calls-output")
        # Don't create directory here - only when actually saving files
    
    def format_call_to_markdown(self, call_data: Dict[str, Any]) -> str:
        """
        Format a single call into markdown content.
        
        Args:
            call_data: Call details dictionary
            
        Returns:
            Formatted markdown string
        """
        # Extract call information
        title = call_data.get("title", "Untitled Call")
        customer = call_data.get("customer_name", "Unknown Customer")
        date = call_data.get("date", "")
        attendees = call_data.get("attendees", [])
        transcript = call_data.get("transcript", "No transcript available")
        call_id = call_data.get("id", "")
        call_url = call_data.get("call_url", "")
        
        # Format date for display
        formatted_date = self._format_date(date)
        
        # Build markdown content
        markdown_content = f"""# {title}

**Customer:** {customer}
**Date:** {formatted_date}
**Call ID:** `{call_id}`"""
        
        # Add call URL if available
        if call_url:
            markdown_content += f"\n**Call Link:** {call_url}"
        
        markdown_content += "\n\n## Attendees\n\n"
        
        # Add attendees section
        if attendees:
            for attendee in attendees:
                name = attendee.get("name", "Unknown")
                title = attendee.get("title", "")
                company = attendee.get("company", "")
                email = attendee.get("email", "")
                
                markdown_content += f"- **{name}**"
                if title:
                    markdown_content += f" - {title}"
                if company:
                    markdown_content += f" ({company})"
                if email:
                    markdown_content += f" - {email}"
                markdown_content += "\n"
        else:
            markdown_content += "No attendee information available.\n"
        
        # Add transcript section
        markdown_content += f"""

## Transcript

{self._clean_transcript(transcript)}

---
*Generated on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}*
"""
        
        return markdown_content
    
    def save_call_markdown(self, call_data: Dict[str, Any]) -> Path:
        """
        Save a call as a markdown file with proper naming.
        
        Args:
            call_data: Call details dictionary
            
        Returns:
            Path to the saved markdown file
        """
        # Extract info for filename
        generated_title = call_data.get("generatedTitle", "")
        customer = call_data.get("customer_name", "Unknown-Customer")
        date = call_data.get("date", "")
        
        # Format date for filename
        file_date = self._format_date_for_filename(date)
        
        # Create filename - prefer generatedTitle, fallback to customer
        if generated_title and generated_title.strip():
            clean_title = self._sanitize_filename(generated_title)
            filename = f"{clean_title}-{file_date}.md"
        else:
            clean_customer = self._sanitize_filename(customer)
            filename = f"{clean_customer}-{file_date}.md"
        filepath = self.output_dir / filename
        
        # Generate markdown content
        markdown_content = self.format_call_to_markdown(call_data)
        
        # Ensure output directory exists
        self.output_dir.mkdir(exist_ok=True)
        
        # Write file
        try:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(markdown_content)
            
            logger.info(f"Saved call markdown to {filepath}")
            return filepath
            
        except Exception as e:
            logger.error(f"Failed to save markdown for call {call_data.get('id', 'unknown')}: {e}")
            raise
    
    def save_multiple_calls(self, calls_data: List[Dict[str, Any]], custom_dir_name: Optional[str] = None) -> List[Path]:
        """
        Save multiple calls as markdown files.
        
        Args:
            calls_data: List of call details dictionaries
            custom_dir_name: Optional custom directory name (e.g., customer name)
            
        Returns:
            List of paths to saved markdown files
        """
        saved_files = []
        
        # Create output directory - use custom name if provided, otherwise use dated format
        if custom_dir_name:
            # Sanitize customer name for directory use
            sanitized_name = self._sanitize_filename(custom_dir_name)
            dated_output_dir = Path(f"ct_{sanitized_name}")
        else:
            today = datetime.now().strftime('%Y-%m-%d')
            dated_output_dir = Path(f"team-calls-{today}")
        
        dated_output_dir.mkdir(exist_ok=True)
        
        # Temporarily use dated directory
        original_output_dir = self.output_dir
        self.output_dir = dated_output_dir
        
        try:
            for call_data in calls_data:
                try:
                    filepath = self.save_call_markdown(call_data)
                    saved_files.append(filepath)
                except Exception as e:
                    logger.error(f"Failed to save call {call_data.get('id', 'unknown')}: {e}")
                    continue
                    
            logger.info(f"Saved {len(saved_files)} call markdown files to {dated_output_dir}")
            
        finally:
            # Restore original output directory
            self.output_dir = original_output_dir
        
        return saved_files
    
    def _format_date(self, date_str: str) -> str:
        """Format date string for display."""
        if not date_str:
            return "Unknown Date"
        
        try:
            # Try to parse various date formats
            for fmt in [
                "%Y-%m-%dT%H:%M:%S.%fZ",  # ISO with microseconds
                "%Y-%m-%dT%H:%M:%SZ",     # ISO without microseconds
                "%Y-%m-%d %H:%M:%S",      # Standard format
                "%Y-%m-%d",               # Date only
            ]:
                try:
                    dt = datetime.strptime(date_str, fmt)
                    return dt.strftime("%B %d, %Y at %I:%M %p")
                except ValueError:
                    continue
            
            # If no format matches, return as-is
            return date_str
            
        except Exception:
            return date_str or "Unknown Date"
    
    def _format_date_for_filename(self, date_str: str) -> str:
        """Format date string for use in filename."""
        if not date_str:
            return "unknown-date"
        
        try:
            # Try to parse various date formats
            for fmt in [
                "%Y-%m-%dT%H:%M:%S.%fZ",  # ISO with microseconds
                "%Y-%m-%dT%H:%M:%SZ",     # ISO without microseconds
                "%Y-%m-%d %H:%M:%S",      # Standard format
                "%Y-%m-%d",               # Date only
            ]:
                try:
                    dt = datetime.strptime(date_str, fmt)
                    return dt.strftime("%Y-%m-%d")
                except ValueError:
                    continue
            
            # If no format matches, sanitize the original string
            return self._sanitize_filename(date_str)
            
        except Exception:
            return "unknown-date"
    
    def _sanitize_filename(self, filename: str) -> str:
        """Sanitize a string for use as a filename."""
        if not filename:
            return "unnamed"
        
        # Step 1: Keep only letters, numbers, spaces, and basic punctuation
        sanitized = re.sub(r'[^a-zA-Z0-9\s\-._()]', '', filename)
        
        # Step 2: Replace whitespace and parentheses with hyphens
        sanitized = re.sub(r'[\s()]+', '-', sanitized)
        
        # Step 3: Remove dots and underscores (keep only letters, numbers, hyphens)
        sanitized = re.sub(r'[._]+', '-', sanitized)
        
        # Step 4: Collapse multiple consecutive hyphens into single hyphens
        sanitized = re.sub(r'-+', '-', sanitized)
        
        # Step 5: Clean up and format
        sanitized = sanitized.strip('-.').lower()
        
        # Step 6: Limit length
        if len(sanitized) > 50:
            sanitized = sanitized[:50].rstrip('-.')
        
        return sanitized or "unnamed"
    
    def _clean_transcript(self, transcript: str) -> str:
        """Clean and format transcript text for markdown."""
        if not transcript:
            return "No transcript available."
        
        # Basic cleaning
        cleaned = transcript.strip()
        
        # Check if transcript is already properly formatted (contains **Speaker:** patterns)
        if '**' in cleaned and ':**' in cleaned:
            # Already formatted by the API client, just return with minimal cleaning
            # Remove excessive blank lines (more than 2 consecutive)
            import re
            cleaned = re.sub(r'\n\n\n+', '\n\n', cleaned)
            return cleaned
        
        # Legacy formatting for unformatted transcripts
        # Format speaker changes more clearly
        # Look for patterns like "Speaker Name:" at start of lines
        lines = cleaned.split('\n')
        formatted_lines = []
        
        for line in lines:
            line = line.strip()
            if not line:
                continue  # Skip empty lines, let natural spacing handle it
            
            # Check if line starts with a speaker name pattern
            if ':' in line and len(line.split(':', 1)[0]) < 50:
                # This looks like a speaker line
                parts = line.split(':', 1)
                speaker = parts[0].strip()
                text = parts[1].strip() if len(parts) > 1 else ""
                formatted_lines.append(f"**{speaker}:** {text}")
            else:
                formatted_lines.append(line)
        
        return '\n\n'.join(formatted_lines)
    
    def format_email_to_markdown(self, email: Email) -> str:
        """
        Format a single email into markdown content.
        
        Args:
            email: Email object to format
            
        Returns:
            Formatted markdown string
        """
        # Extract email information
        subject = email.subject or "No Subject"
        sender_name = email.sender.name or "Unknown Sender"
        sender_email = email.sender.email or ""
        sent_at = email.sent_at.strftime("%B %d, %Y at %I:%M %p") if email.sent_at else "Unknown Date"
        
        # Build sender info
        sender_info = f"{sender_name}"
        if sender_email:
            sender_info += f" ({sender_email})"
        if email.sender.title:
            sender_info += f" - {email.sender.title}"
        if email.sender.company:
            sender_info += f" @ {email.sender.company}"
        
        # Build markdown content
        markdown_content = f"""## {subject}

**From:** {sender_info}
**Date:** {sent_at}
**Direction:** {email.direction.value.title()}
**Email ID:** `{email.id}`"""
        
        # Add recipients section
        if email.recipients:
            markdown_content += "\n**To:** "
            recipient_names = []
            for recipient in email.recipients:
                name = recipient.name or recipient.email.split("@")[0] if recipient.email else "Unknown"
                if recipient.email:
                    recipient_names.append(f"{name} ({recipient.email})")
                else:
                    recipient_names.append(name)
            markdown_content += ", ".join(recipient_names)
        
        # Add automation/template info if applicable
        if email.is_automated or email.is_template:
            markdown_content += f"\n**Type:** "
            if email.is_template:
                markdown_content += "Template/Automated"
            else:
                markdown_content += "Automated"
        
        # Add body content
        markdown_content += "\n\n### Content\n\n"
        
        if email.body_text and email.body_text.strip():
            # Clean and format the body text
            cleaned_body = self._clean_email_body(email.body_text)
            markdown_content += cleaned_body
        elif email.snippet and email.snippet.strip():
            # Fallback to snippet if no body
            markdown_content += f"*[Preview only - full content not available]*\n\n{email.snippet}"
        else:
            markdown_content += "*No content available*"
        
        markdown_content += "\n\n---\n"
        
        return markdown_content
    
    def format_emails_batch_to_markdown(self, emails: List[Email], batch_num: int, customer_name: str) -> str:
        """
        Format a batch of emails into a single markdown document.
        
        Args:
            emails: List of emails to format
            batch_num: Batch number for the title
            customer_name: Customer name for the title
            
        Returns:
            Formatted markdown string for the entire batch
        """
        if not emails:
            return "# No Emails\n\nNo emails found in this batch."
        
        # Sort emails by date (newest first)
        sorted_emails = sorted(emails, key=lambda e: e.sent_at or datetime.min, reverse=True)
        
        # Get date range for title
        if sorted_emails:
            oldest_date = min(email.sent_at for email in sorted_emails if email.sent_at)
            newest_date = max(email.sent_at for email in sorted_emails if email.sent_at)
            date_range = f"{oldest_date.strftime('%m/%d')} - {newest_date.strftime('%m/%d/%Y')}"
        else:
            date_range = "Unknown Date Range"
        
        # Build header
        markdown_content = f"""# {customer_name} - Emails Batch {batch_num}

**Date Range:** {date_range}  
**Total Emails:** {len(emails)}  
**Generated:** {datetime.now().strftime('%B %d, %Y at %I:%M %p')}  
**Advanced BDR/SPAM filtering applied** - Templates, duplicates, and automation removed

---

"""
        
        # Add each email
        for i, email in enumerate(sorted_emails, 1):
            markdown_content += f"### Email {i}/{len(emails)}\n\n"
            markdown_content += self.format_email_to_markdown(email)
            markdown_content += "\n"
        
        # Add footer
        markdown_content += f"""

---
*Batch {batch_num} of emails for {customer_name} - Generated by cs-transcript-cli*
"""
        
        return markdown_content
    
    def save_emails_as_markdown(self, emails: List[Email], customer_name: str, custom_dir_name: Optional[str] = None) -> List[Path]:
        """
        Save emails as markdown files in batches of 20 with specific naming pattern.
        
        Args:
            emails: List of emails to save
            customer_name: Customer name for directory and file naming
            custom_dir_name: Optional custom directory name
            
        Returns:
            List of paths to saved markdown files
        """
        if not emails:
            logger.info("No emails to save")
            return []
        
        # Create output directory
        if custom_dir_name:
            sanitized_name = self._sanitize_filename(custom_dir_name)
            output_dir = Path(f"ct_{sanitized_name}")
        else:
            sanitized_name = self._sanitize_filename(customer_name)
            output_dir = Path(f"ct_{sanitized_name}")
        
        output_dir.mkdir(exist_ok=True)
        
        # Sort emails by date for consistent batching
        sorted_emails = sorted(emails, key=lambda e: e.sent_at or datetime.min)
        
        saved_files = []
        batch_size = 20
        
        # Process emails in batches of 20
        for batch_start in range(0, len(sorted_emails), batch_size):
            batch_end = min(batch_start + batch_size, len(sorted_emails))
            batch_emails = sorted_emails[batch_start:batch_end]
            batch_num = (batch_start // batch_size) + 1
            
            # Calculate date range for filename
            if batch_emails:
                # Get date range from actual emails in this batch
                batch_dates = [email.sent_at for email in batch_emails if email.sent_at]
                if batch_dates:
                    oldest_date = min(batch_dates)
                    newest_date = max(batch_dates)
                    opening_range = oldest_date.strftime("%m-%d")
                    closing_range = newest_date.strftime("%m-%d")
                else:
                    opening_range = "unknown"
                    closing_range = "unknown"
            else:
                opening_range = "unknown"
                closing_range = "unknown"
            
            # Create filename with specified pattern: [customer]-emls-[opening range mm-dd]-[closing range mm-dd]
            clean_customer = self._sanitize_filename(customer_name)
            filename = f"{clean_customer}-emls-{opening_range}-{closing_range}.md"
            
            # Handle duplicate filenames by adding batch number
            filepath = output_dir / filename
            if filepath.exists():
                filename = f"{clean_customer}-emls-{opening_range}-{closing_range}-batch{batch_num}.md"
                filepath = output_dir / filename
            
            # Generate markdown content for this batch
            markdown_content = self.format_emails_batch_to_markdown(batch_emails, batch_num, customer_name)
            
            # Save the batch file
            try:
                with open(filepath, 'w', encoding='utf-8') as f:
                    f.write(markdown_content)
                
                saved_files.append(filepath)
                logger.info(f"Saved email batch {batch_num} ({len(batch_emails)} emails) to {filepath}")
                
            except Exception as e:
                logger.error(f"Failed to save email batch {batch_num}: {e}")
                continue
        
        logger.info(f"Saved {len(emails)} emails across {len(saved_files)} batch files to {output_dir}")
        return saved_files
    
    def _clean_email_body(self, body_text: str) -> str:
        """Clean and format email body text for markdown."""
        if not body_text or not body_text.strip():
            return "*No content available*"
        
        # Basic cleaning
        cleaned = body_text.strip()
        
        # Remove excessive whitespace but preserve paragraph breaks
        import re
        cleaned = re.sub(r'\n\s*\n\s*\n+', '\n\n', cleaned)
        cleaned = re.sub(r'[ \t]+', ' ', cleaned)
        
        # Handle common email artifacts
        cleaned = re.sub(r'^[\s>]+', '', cleaned, flags=re.MULTILINE)  # Remove quote prefixes
        cleaned = re.sub(r'(^|\n)On .* wrote:\s*$', r'\1\n---\n\n**Previous conversation:**\n', cleaned, flags=re.MULTILINE)
        
        # Ensure reasonable length (truncate very long emails)
        max_length = 5000  # 5000 characters max
        if len(cleaned) > max_length:
            cleaned = cleaned[:max_length] + "\n\n*[Email content truncated for readability]*"
        
        return cleaned


class CallSummaryReporter:
    """Generates summary reports of extracted calls."""
    
    def __init__(self):
        pass
    
    def generate_summary_report(self, 
                              calls_data: List[Dict[str, Any]], 
                              output_path: Optional[Path] = None) -> str:
        """
        Generate a summary report of all extracted calls.
        
        Args:
            calls_data: List of call details
            output_path: Optional path to save summary report
            
        Returns:
            Summary report markdown content
        """
        today = datetime.now().strftime('%Y-%m-%d')
        
        summary_content = f"""# Team Calls Summary - {today}

Generated on {datetime.now().strftime('%B %d, %Y at %I:%M %p')}

## Overview

- **Total Calls:** {len(calls_data)}
- **Date Range:** Last 7 days
- **Extraction Date:** {today}

## Calls by Customer

"""
        
        # Group calls by customer
        customer_calls = {}
        for call in calls_data:
            customer = call.get("customer_name") or "Unknown Customer"
            if customer not in customer_calls:
                customer_calls[customer] = []
            customer_calls[customer].append(call)
        
        # Add customer sections (handle None values in sorting)
        for customer, calls in sorted(customer_calls.items(), key=lambda x: x[0] or ""):
            summary_content += f"### {customer} ({len(calls)} calls)\n\n"
            
            for call in calls:
                title = call.get("title", "Untitled Call")
                date = call.get("date", "Unknown Date")
                formatted_date = self._format_date_summary(date)
                call_id = call.get("id", "")
                
                summary_content += f"- **{title}** - {formatted_date} (ID: `{call_id}`)\n"
            
            summary_content += "\n"
        
        # Save if path provided
        if output_path:
            try:
                with open(output_path, 'w', encoding='utf-8') as f:
                    f.write(summary_content)
                logger.info(f"Summary report saved to {output_path}")
            except Exception as e:
                logger.error(f"Failed to save summary report: {e}")
        
        return summary_content
    
    def _format_date_summary(self, date_str: str) -> str:
        """Format date for summary display."""
        if not date_str:
            return "Unknown Date"
        
        try:
            for fmt in [
                "%Y-%m-%dT%H:%M:%S.%fZ",
                "%Y-%m-%dT%H:%M:%SZ",
                "%Y-%m-%d %H:%M:%S",
                "%Y-%m-%d",
            ]:
                try:
                    dt = datetime.strptime(date_str, fmt)
                    return dt.strftime("%m/%d/%Y")
                except ValueError:
                    continue
            return date_str
        except Exception:
            return "Unknown Date"