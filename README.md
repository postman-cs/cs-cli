![CS-Transcript-CLI Banner](banner.jpg)

# CS Deep Research CLI

Turn 6 months of customer calls into actionable insights in 5 minutes. 

Just type `cs-cli` and answer 3 simple questions.

## Why This Matters to You

Critical customer issues hide in a sea of data; by the time they surface, most of the time you're too late. This tool changes that.

**What you'll discover:**
- Issues your customer forgot to follow up on
- Problems brewing beneath the surface  
- Exact quotes to reference in your next call
- Opportunities to shift from vendor to trusted advisor

## Get Started in 3 Minutes

<div align="center">
  <img src="demo.gif" alt="Demo Gif">
</div>

### Step 1: Set It Up - Only need to do it Once

First, open Terminal (it's just a text window):
- Press `Command + Space` 
- Type "Terminal"
- Press Enter

You'll see a window with text like this:
```
YourName@YourMac ~ %
```

Copy this entire line and paste it after the `%` symbol:
```bash
curl -s https://raw.githubusercontent.com/jaredboynton/cs-cli/main/install.sh | bash
```

Press Enter. That's it! The installer will handle everything else.

**If the installer says to run `source ~/.zshrc`:** Just copy and paste that command and press Enter. This makes the `cs-cli` command available immediately.

**Important:** Before using the tool, make sure you're logged into Gοng in Firefox (your regular browser). The tool needs use Gοng to download your customer conversations.

### Step 2: Find Customer Insights

Just type:
```bash
cs-cli
```

That's it! The tool will ask you everything it needs:
- What customer are you looking for?
- How far back should I look?
- What would you like to analyze?

**No quotes needed. No complex commands. Just answer the questions.**

#### Advanced: Command-Line Mode
Arguments can be in any order. If you don't specify days, it defaults to 90:
```bash
cs-cli Postman 90 calls emails        # Last 3 months of Postman
cs-cli Wells Fargo calls              # Last 90 days (default) of Wells Fargo calls
cs-cli 7 - 11 365                     # Last year of 7-Eleven
cs-cli emails Stripe 30                # Last month of Stripe emails
```

### Step 3: Find Your Results

Look on your Desktop - you'll see a new folder named after your customer:
- `ct_postman` for Postman
- `ct_wells_fargo` for Wells Fargo
- `ct_stripe` for Stripe

### Step 4: Analyze with AI

1. Open your AI Agent (eg. Cursor)
2. Click "File" → "Open Folder"
3. Select the customer folder from your Desktop
4. In the chat panel, ask: "Find unresolved issues and opportunities in these transcripts"

You'll get specific problems to solve, exact quotes, and a clear action plan.

## Common Questions

**"I've never used Terminal before"**  
Perfect! This tool was built for you. Terminal is just a text-based way to talk to your computer. Copy and paste the commands exactly as shown - you'll be fine.

**"What if it asks for my password?"**  
That's normal! The installer needs to install software on your Mac, which requires your permission. When you type your password, you won't see any characters appear - just type your Mac login password and press Enter.

**"What if I mess up?"**  
You can't break anything. If something doesn't work, just close Terminal, open it again, and try once more.

**"What does interactive mode look like?"**  
```
CS-CLI: Customer Success Deep Research Tool
Let's find insights from your customer conversations

What customer are you looking for?: Wells Fargo
How far back should I look?: 90

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
Make sure you're logged into Gοng in Firefox first. The tool can only see data you have access to.

**"Where do the files go?"**  
Always on your Desktop! Each customer gets their own folder: `ct_postman`, `ct_stripe`, etc. Easy to find, easy to share.

## Your Next Move

1. Pick a customer you're worried about
2. Run the tool to extract their last 90 days
3. Use AI to find issues you didn't know existed
4. Show up to your next call as their hero

---

*Built by your technical colleagues who believe in your success. Questions? We're here to help.*
