"""Markdown formatter for team call reports."""

import re
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional

import structlog

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
        customer = call_data.get("customer_name", "Unknown-Customer")
        date = call_data.get("date", "")
        
        # Clean customer name for filename
        clean_customer = self._sanitize_filename(customer)
        
        # Format date for filename
        file_date = self._format_date_for_filename(date)
        
        # Create filename
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
            dated_output_dir = Path(f"{sanitized_name}")
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
        
        # Step 1: Remove problematic filename characters: < > : " / \ | ? *
        sanitized = re.sub(r'[<>:"/\\|?*]', '', filename)
        
        # Step 2: Replace whitespace with hyphens
        sanitized = re.sub(r'\s+', '-', sanitized)
        
        # Step 3: Collapse multiple consecutive hyphens into single hyphens
        sanitized = re.sub(r'-+', '-', sanitized)
        
        # Step 4: Clean up and format
        sanitized = sanitized.strip('-.').lower()
        
        # Step 5: Limit length
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