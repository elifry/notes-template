# Journal Automation

I want to automate certain things about my journal. For now, these will be in the form of rust tools that I can run as needed. Eventually, I see these being run regularly and without my involvement.

I aim to make small utilities that do a specific thing. I can use them to format my journals to all match what I expect, fix mistakes, convert older journals to newer methods, and collect data and give me my first analytics.

## `ja` CLI Tool Usage

The tool is available as a Rust CLI application with the following commands:

```bash
# Start today's journal entry (creates header with date, device, location, and weather)
ja start-journal

# Open today's journal entry in your preferred editor
ja open-journal

# Open a specific journal entry by date
ja open-day 2024-03-15

# Create an entire year's worth of journal files
ja create-year 2025 [class]  # Optional class name (e.g., CS101), defaults to "journal"

# Find and open a random empty journal entry (optionally filtered by year)
ja empty-day [year]

# Add a custom header to today's journal entry
ja add-custom-header "Header Text"

# Analyze journal completion rates by year (with completion markers and days remaining)
ja analyze-completion

# Analyze average word and line counts by year
ja analyze-length

# Validate journal structure against expected dates
ja validate-structure

# Validate journal contents against file dates
ja validate-contents
```

## Completed utilities

- Start today's journal: a vscode run action (now implemented as `ja start-journal`)
- Open today's journal: a vscode run action (now implemented as `ja open-journal`)
- Open specific journal: opens entry for given date (now implemented as `ja open-day`)
- Create entire year structure: creates all folders and files for a given year (now implemented as `ja create-year`)
- Find empty journal entries: finds and opens a random empty journal entry with proper date header, optionally filtered by year (now implemented as `ja empty-day`)
- Add custom headers: adds a custom H2 header to today's journal entry (now implemented as `ja add-custom-header`)
- Analyze completion rates: shows completion statistics for each year with visual indicators (now implemented as `ja analyze-completion`)
- Analyze journal length: shows average word and line counts per year (now implemented as `ja analyze-length`)
- Validate structure: checks for date mismatches and duplicates (now implemented as `ja validate-structure`)
- Validate contents: checks that journal headers match their file dates (now implemented as `ja validate-contents`)

## Later utilities

- Where did I park (North Campus or not)
- Export journal template (take the most up to date iteration and create a new repo with the templates and structure in place without anything else, so that someone else could clone and start using it)
- Exercise data include (Nike run club, Health app?)
- Sleep data include (Health app?)
- Clean templates (I add things for future days that are one-offs, which then must be removed from the template once that day has passed and it is either done on that day or must be moved to a later day)
- Video shuffler, have a lot more of them and shuffle through them, and never show the same too close together
- Make a plan for me (take my current load of one off things, ask for time estimates, and help me fit them into my schedule for me)
- Years visualizer
- Longest streak(s)
- Paper journal convert assistant (help me create the journal data page, get the date range from that, create dummy files and links, it checks which ones I still have to enter, adds the number of pages to convert that I am comfortable with)
- Move daily notes link to next month (daily-notes.json `"folder": "journal/2022/07-jul",`)
- Number of coffees (and other) in header matches log entries
- Determine if there is a conference call on Tuesdays automatically [by reading my emails](https://www.codeforests.com/2020/06/04/python-to-read-email-from-outlook/) (or by looking at the calendar) - this could then trigger a reschedule event if necessary
- Mass categorizer using hashtags, example #stubby for short days with barely anything written
- Intelligent word count that does not count anything from templates or repeating tasks

## Smaller utilities created as a result

- List all journal days
- Log file stuff

## Development

The project is structured as a Rust CLI tool with the following components:

- `src/cli.rs`: Command-line interface definitions
- `src/journal.rs`: Core journal functionality
- `src/utils.rs`: Utility functions (device info, location, weather, editor)

## Editor Requirements

The tool is designed to work with either Cursor or VS Code as the default editor. To ensure proper functionality:

1. Install either Cursor (preferred) or VS Code
2. Add the editor to your system PATH:
   - For Windows:
     - Add to both Command Prompt (cmd) and PowerShell PATH
     - Restart your terminal after installation
   - For macOS/Linux:
     - The installer should handle this automatically
     - If needed, add to your shell's PATH manually

### Troubleshooting Editor Issues

If you encounter issues with the editor not being found:

1. Verify the editor is in your PATH:
   - Windows: Run `where cursor` or `where code` in Command Prompt
   - macOS/Linux: Run `which cursor` or `which code` in terminal
2. Try restarting your terminal/IDE
3. If using PowerShell on Windows, ensure the editor is in PowerShell's PATH specifically
4. The tool will automatically try multiple methods to launch the editor, including:
   - Direct command
   - Command Prompt (Windows)
   - PowerShell (Windows)
