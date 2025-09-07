![CS-Transcript-CLI Banner](banner.jpg)

# Customer Transcripts CLI - Team Calls Extractor

A standalone tool for extracting customer calls from Gong and saving them as markdown transcripts.

## Features

- **One-click setup**: Installs everything automatically, including Python if needed
- **Multi-browser authentication**: Automatically detects Gong cookies from Chrome, Firefox, Safari, Brave, Edge, Opera, or Chromium
- **Flexible date ranges**: Extract calls by days back or custom date ranges  
- **Rich markdown output**: Clean, formatted transcripts with attendee information
- **High-performance**: Concurrent API calls with intelligent rate limiting
- **Progress tracking**: Real-time progress bars for extraction status

## Installation

### Option 1: One-Click Setup (Recommended for Non-Technical Users)

**If you have git:**
```bash
git clone <repository-url>
cd cs-transcript-cli
./setup
```

**If you don't have git:**
1. Go to the repository page on GitHub
2. Click the green "Code" button → "Download ZIP"
3. Extract the ZIP file to your Desktop
4. Open Terminal and navigate to the folder:
   ```bash
   cd ~/Desktop/cs-transcript-cli
   ./setup
   ```

The setup script will automatically:
- Install Homebrew if you don't have it
- Install Git if you don't have it  
- Install Python 3 if you don't have it
- Create a virtual environment
- Install all dependencies
- Set up the CLI tool

### Option 2: Manual Installation (For Technical Users)
```bash
git clone <repository-url>
cd cs-transcript-cli
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

After running `./setup`, use the `./cli` command to extract calls:

### Basic Usage

**Extract calls by customer (most common):**
```bash
# Last 7 days for customer
./cli --customer "7-11" 

# Last 30 days for customer
./cli --customer "Postman" --days 30

# Customer with partial name matching
./cli --customer "7-" 
```

**Extract calls from folder (team calls):**
```bash
# Use default folder - last 7 days
./cli

# Specific folder - last 30 days  
./cli --folder-id "195005774106634129" --days 30
```

### Date Range Options
```bash
# Customer-based with date ranges
./cli --customer "7-11" --from-date 2024-01-01 --to-date 2024-01-31

# Folder-based with date ranges
./cli --folder-id "195005774106634129" --from-date 2024-01-01 --to-date 2024-01-31

# From specific date to now
./cli --customer "Postman" --from-date 2024-01-01

# All calls up to specific date  
./cli --customer "7-11" --to-date 2024-01-31
```

### Other Options
```bash
# Enable debug logging
./cli --debug --customer "7-11"

# Custom output directory  
./cli --output-dir ./my-calls --customer "Postman"

# Get help
./cli --help
```

### Customer Search Features

The `--customer` parameter uses intelligent autocomplete matching:

- **Partial matching**: `--customer "7-"` finds "7 - Eleven (7 - 11) Inc.", "700Apps", "777 Partners", etc.
- **Fuzzy matching**: `--customer "seven"` finds "7 - Eleven (7 - 11) Inc."
- **Company resolution**: Automatically resolves customer names to company entities in Gong
- **Multiple matches**: If multiple companies match, includes calls from all matching entities

## Workflows & Use Cases

This tool enables two powerful workflows for converting conversation data into actionable business insights. Both work best when combined with AI agents for post-processing analysis.

### Workflow 1: Team Performance & Coaching Analysis

**Objective:** Generate weekly performance summaries and identify coaching opportunities across your team.

**Setup Process:**
1. **Create a Team Stream in Gong:** Set up a dedicated folder/stream containing your team members' calls
2. **Weekly Extraction:** Run every Friday to capture the full week's activities

**Extraction Command:**
```bash
# Extract last week's team calls
./cli --folder-id "your-team-stream-id" --days 7

# Or use specific date range for consistency
./cli --folder-id "your-team-stream-id" --from-date 2024-01-15 --to-date 2024-01-19
```

**AI Post-Processing Framework:**
Use Claude, Cursor, or another AI agent to analyze the extracted transcripts for:

- **Performance Metrics:** Talk/listen ratios, question frequency, meeting control
- **Deal Progression:** Advancement signals, next steps clarity, timeline adherence  
- **Coaching Opportunities:** Objection handling, discovery depth, presentation skills
- **Customer Wins:** Value realization moments, competitive advantages, success stories
- **Process Adherence:** Methodology following, documentation quality

**Recommended AI Analysis Prompt Structure:**
```
Analyze these [X] call transcripts from our sales team this week.

For each call, extract:
1. Key outcomes and progression signals
2. Coaching opportunities (specific behaviors to improve)
3. Customer sentiment and engagement level
4. Competitive mentions or concerns
5. Process adherence (discovery, demo, next steps)

Then create a summary report with:
- Executive summary of week's impact
- Individual performance highlights/areas for improvement  
- Top 5 coaching priorities across the team
- Customer wins and success stories to share
- Action items for next week

Include specific quotes as evidence for all observations.
```

**Success Metrics:**
- Faster identification of at-risk deals
- More targeted coaching conversations
- Improved win rates and deal velocity
- Better customer experience consistency

### Workflow 2: Customer Evolution & Relationship Analysis

**Objective:** Build comprehensive customer health assessments and identify renewal risks or expansion opportunities.

**Setup Process:**
1. **Pre-QBR Analysis:** Extract 30-90 days before quarterly business reviews
2. **Renewal Preparation:** Run 60-90 days before renewal dates
3. **Escalation Investigation:** Extract when customer issues arise

**Extraction Command:**
```bash
# Extract 60 days of customer interactions
./cli --customer "Postman" --days 60

# Or analyze specific time periods
./cli --customer "7-11" --from-date 2024-01-01 --to-date 2024-03-31
```

**AI Post-Processing Framework:**
Use AI to analyze customer transcripts for:

- **Stakeholder Evolution:** Who's engaged, who's dropped off, new contacts
- **Sentiment Tracking:** Satisfaction changes over time, frustration signals
- **Technical Blockers:** Implementation issues, feature gaps, integration problems
- **Commercial Discussions:** Pricing concerns, budget changes, competitive pressure
- **Relationship Health:** Executive sponsorship, strategic alignment, usage adoption
- **Follow-up Gaps:** Commitments not kept, questions not answered, promises unfulfilled

**Recommended AI Analysis Prompt Structure:**
```
Analyze these call transcripts for [CUSTOMER] over the past [X] days.

Create a comprehensive customer evolution report:

1. **Timeline of Key Events:** Chronological list of significant moments, decisions, and changes

2. **Stakeholder Analysis:** 
   - Current active contacts and their roles
   - Stakeholders who have disengaged 
   - New contacts that have joined conversations
   - Decision-maker identification and influence

3. **Sentiment & Relationship Health:**
   - Overall satisfaction trajectory 
   - Areas of frustration or concern
   - Engagement level changes
   - Trust and partnership indicators

4. **Technical & Commercial Assessment:**
   - Outstanding technical blockers or issues
   - Implementation challenges and progress
   - Budget or pricing discussions
   - Feature requests and product feedback

5. **Risk Assessment:**
   - Renewal risk factors (specific evidence)
   - Competitive threats mentioned
   - Red flags requiring immediate attention
   - Missed commitments or follow-ups

6. **Opportunity Analysis:**
   - Expansion possibilities discussed
   - Additional use cases or departments
   - Success stories and value realization

7. **Recommended Actions:**
   - Immediate priorities for account team
   - Stakeholder re-engagement needed
   - Technical issues to resolve
   - Strategic discussions to schedule

Include specific quotes and call dates as evidence for all observations.
```

**Success Metrics:**
- Earlier identification of renewal risks
- More strategic account planning
- Improved customer satisfaction scores
- Higher expansion revenue per account

## AI Post-Processing Best Practices

### Tool Selection
- **Claude (Anthropic):** Excellent for analysis, summarization, and structured outputs
- **Cursor:** Great for iterating through multiple transcript files programmatically
- **ChatGPT:** Good for conversational analysis and creative insights
- **Custom Agents:** Build specialized agents for your specific analysis needs

### Prompt Engineering Tips
1. **Be Specific:** Define exactly what insights you want extracted
2. **Require Evidence:** Always ask for specific quotes and timestamps
3. **Structure Output:** Request consistent JSON or markdown formats
4. **Set Context:** Provide background on your sales process, product, and goals
5. **Iterate:** Refine prompts based on output quality

### Quality Assurance
- **Spot Check:** Manually verify AI insights against original transcripts
- **Evidence Links:** Ensure all claims reference specific conversation moments  
- **Confidence Scoring:** Ask AI to rate confidence in each observation
- **Multiple Perspectives:** Use different AI models for cross-validation on critical insights

### Integration & Action
- **CRM Updates:** Use insights to update customer records and opportunity stages
- **Slack Summaries:** Share weekly team insights in sales channels
- **Coaching Sessions:** Use specific examples in 1:1 performance discussions
- **Account Planning:** Incorporate customer evolution data into strategic reviews

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

The tool creates organized directories with:
- **Customer extractions**: `ct_[customer-name]/` (e.g., `ct_7-11/`, `ct_postman/`)
- **Team call extractions**: `team-calls-[date]/` (e.g., `team-calls-2024-01-15/`)
- Individual markdown files for each call
- `SUMMARY.md` with an overview of all extracted calls

Customer folders are prefixed with `ct_` for easy organization and git exclusion.

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