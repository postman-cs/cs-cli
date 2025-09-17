![CS-Transcript-CLI Banner](banner.jpg)

# CS Deep Research CLI

**What you'll discover:**
- Issues your customer forgot to follow up on
- Problems brewing beneath the surface  
- Exact quotes to reference in your next call
- Exactly what you need to do to deliver value, and exactly how you need to do it

## Installation

### **[Download CS-CLI Installer (.pkg)](https://github.com/postman-cs/cs-cli/releases/latest/download/cs-cli-macos.pkg)** 

Just download the latest PKG file and double-click. That's it!

### Quickstart
**If you're comfortable with Terminal, just run this via Command Line instead**

```bash
curl -s https://raw.githubusercontent.com/postman-cs/cs-cli/main/install.sh | bash
```
```bash
source ~/.zshrc && cs-cli
```

### Step 1: Get Your Data

![Demo!](demo.gif)

**Important:** Before using the tool, make sure you're logged into Gοng in your browser. The tool needs Gοng to download your customer conversations.

Just click the app or run `cs-cli` in your terminal and answer 3 simple questions: 
- WHO: Which Customer?
- WHEN: How far back do you want to go?
- WHAT: Calls? Emails?

#### Advanced: Command-Line Mode
Arguments can be in any order. If you don't specify days, it defaults to 90:
```bash
cs-cli Postman 90 calls emails        # Last 3 months of Postman
cs-cli Wells Fargo calls              # Last 90 days (default) of Wells Fargo calls
cs-cli 7 - 11 365                     # Last year of 7-Eleven
cs-cli emails Stripe 30                # Last month of Stripe emails
```

### Step 2: Find Your Results

Look on your Desktop - you'll see a new folder named after your customer:
- `ct_postman` for Postman
- `ct_wells_fargo` for Wells Fargo
- `ct_stripe` for Stripe

### Step 3: Analyze with AI

1. Open your AI Agent (eg. Cursor)
2. Click "File" → "Open Folder"
3. Select the customer folder from your Desktop
4. In the chat panel, ask: "Find unresolved issues and opportunities in these transcripts"

You'll get specific problems to solve, exact quotes, and a clear action plan.

## Common Questions

**"I've never used Terminal before"**  
Perfect! This tool was built for you. You'll be fine.

**"What if it asks for my password?"**  
That's normal! The installer needs to install software on your Mac, which requires your permission. When you type your password, you won't see any characters appear - just type your Mac login password and press Enter.

**"What if I mess up?"**  
You can't break anything. If something doesn't work, just close Terminal, open it again, and try once more.

**"What does interactive mode look like?"**  
```
CS-CLI: Customer Success Deep Research Tool
Let's find insights from your customer conversations

What customer are you looking for?: Wells Fargo
How far back should I look?: 180

What would you like to analyze?
1. Calls only
2. Emails only  
3. Both calls and emails (recommended)

Type a number and press Enter [3]: 3

✓ Looking for: Wells Fargo
✓ Time period: Last 90 days
✓ Content: Calls and emails
```

**"How do I know it's working?"**  
You'll see progress messages like "Downloading transcripts..." When it's done, check your Desktop - you'll find a new folder with your customer's name (like `ct_postman`).

**"What if it says 'No customers found'?"**  
Make sure you're logged into Gοng first. The tool can only see data you have access to.

**"Where do the files go?"**  
Always on your Desktop! Each customer gets their own folder: `ct_postman`, `ct_stripe`, etc. Easy to find, easy to share.

**"How do I update the tool?"**
Download the latest version from the [releases page](https://github.com/postman-cs/cs-cli/releases/latest) and install it.

## Your Next Move

1. Pick a customer you're worried about
2. Run the tool to extract their last 90 days
3. Use AI to find issues you didn't know existed
4. Show up to your next call as their hero

---

*Built by your technical colleagues who believe in your success. Questions? We're here to help.*
