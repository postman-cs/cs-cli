# Customer Transcripts CLI - Team Calls Extractor

A standalone tool for extracting customer calls from Gong and saving them as markdown transcripts.

## Features

- **Multi-browser authentication**: Automatically detects Gong cookies from Chrome, Firefox, Safari, Brave, Edge, Opera, or Chromium
- **Flexible date ranges**: Extract calls by days back or custom date ranges  
- **Rich markdown output**: Clean, formatted transcripts with attendee information
- **High-performance**: Concurrent API calls with intelligent rate limiting
- **Progress tracking**: Real-time progress bars for extraction status

## Installation

### Quick Start (Recommended)
The easiest way to use the tool is with the included wrapper scripts that automatically manage the virtual environment:

```bash
git clone <repository-url>
cd customer-transcripts-cli

# Run directly - automatically sets up venv and dependencies
./customer-transcripts --customer "7-11" --days 7

# Or use the team-calls alias
./team-calls --customer "Postman" --days 30
```

The wrapper scripts will:
- Create a virtual environment if one doesn't exist
- Install all dependencies automatically  
- Activate the environment and run the CLI
- Accept all the same parameters as the main CLI

### Manual Installation
```bash
git clone <repository-url>
cd customer-transcripts-cli
python3 -m venv venv
source venv/bin/activate
pip install -e .
```

### Dependencies
The tool requires these external packages:
- `structlog` - Structured logging
- `click` - Command line interface 
- `rich` - Rich terminal output
- `browser-cookie3` - Browser cookie extraction
- `pydantic` - Data validation
- `curl-cffi` - HTTP client with browser fingerprinting

## Usage

### Basic Usage

**Extract calls from folder (default method):**
```bash
# Use default folder - last 7 days
./customer-transcripts

# Specific folder - last 30 days  
./customer-transcripts --folder-id "195005774106634129" --days 30
```

**Extract calls by customer:**
```bash
# Last 7 days for customer
./customer-transcripts --customer "7-11" 

# Last 30 days for customer
./customer-transcripts --customer "Postman" --days 30

# Customer with partial name matching
./customer-transcripts --customer "7-" 
```

### Date Range Options
```bash
# Folder-based with date ranges
./customer-transcripts --folder-id "195005774106634129" --from-date 2024-01-01 --to-date 2024-01-31

# Customer-based with date ranges
./customer-transcripts --customer "7-11" --from-date 2024-01-01 --to-date 2024-01-31

# From specific date to now
./customer-transcripts --customer "Postman" --from-date 2024-01-01

# All calls up to specific date  
./customer-transcripts --customer "7-11" --to-date 2024-01-31
```

### Other Options
```bash
# Enable debug logging
./customer-transcripts --debug --customer "7-11"

# Custom output directory  
./customer-transcripts --output-dir ./my-calls --customer "Postman"

# Get help
./customer-transcripts --help

# Using the team-calls alias
./team-calls --customer "7-11" --days 30
```

### Customer Search Features

The `--customer` parameter uses intelligent autocomplete matching:

- **Partial matching**: `--customer "7-"` finds "7 - Eleven (7 - 11) Inc.", "700Apps", "777 Partners", etc.
- **Fuzzy matching**: `--customer "seven"` finds "7 - Eleven (7 - 11) Inc."
- **Company resolution**: Automatically resolves customer names to company entities in Gong
- **Multiple matches**: If multiple companies match, includes calls from all matching entities

## Authentication

The tool automatically extracts authentication cookies from your browser. Make sure you're logged into Gong in at least one supported browser:

**Supported Browsers:**
- Google Chrome
- Mozilla Firefox  
- Safari (macOS)
- Brave Browser
- Microsoft Edge
- Opera
- Chromium

The tool will try each browser and use the first one with valid Gong cookies.

## Output

The tool creates a timestamped directory with:
- Individual markdown files for each call
- `SUMMARY.md` with an overview of all extracted calls

### Markdown Format
Each call transcript includes:
- Call title and customer information
- Date and attendee list
- Full transcript with speaker identification
- Call metadata (ID, URL if available)

## Configuration

### Environment Variables
- `GONG_DEBUG=true` - Enable debug logging
- `GONG_HTTP_CONCURRENCY=50` - Set HTTP concurrency level

### Performance Tuning
The tool uses intelligent defaults but can be tuned via environment variables for better performance on different systems.

## Troubleshooting

### Authentication Issues
- Ensure you're logged into Gong in a supported browser
- Try refreshing your Gong session in the browser
- Check browser permissions for cookie access

### Rate Limiting
- The tool includes automatic retry logic with exponential backoff
- Reduce concurrency if experiencing rate limits: `GONG_HTTP_CONCURRENCY=20`

### HTTP/3 Issues
If you experience connection issues, HTTP/3 can be disabled by setting the debug flag or modifying config defaults.

## Architecture

The tool is built with a clean architecture:
- **Authentication**: Multi-browser cookie extraction with CSRF token management
- **HTTP Client**: High-performance client pool with intelligent retry logic  
- **API Layer**: Gong API clients for library and transcript extraction
- **Formatters**: Markdown generation with rich formatting
- **CLI**: User-friendly command-line interface

## Contributing

This is a standalone extraction from a larger Gong pipeline project, optimized for team calls extraction specifically.

## License

MIT License