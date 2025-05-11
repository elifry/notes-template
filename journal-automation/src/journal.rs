use crate::utils::{get_device_info, get_git_root, get_location, get_weather, open_in_editor};
use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use rand::seq::SliceRandom;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
};

pub fn get_todays_journal_path() -> Result<String> {
    let today = Local::now();
    let year = today.format("%Y").to_string();
    let month = today.format("%m").to_string();
    let month_name = today.format("%B").to_string();
    let day = today.format("%d").to_string();
    let weekday = today.format("%A").to_string();

    let git_root = get_git_root()?;
    Ok(format!(
        "{}/journal/{}/{}-{}/{}_{}.md",
        git_root,
        year,
        month,
        month_name.to_lowercase(),
        day,
        weekday
    ))
}

pub fn create_journal_entry(journal_path: &str) -> Result<()> {
    if !std::path::Path::new(journal_path).exists() {
        anyhow::bail!("Journal file not found: {}", journal_path);
    }

    println!("Adding header info for: {}", journal_path);

    let today = Local::now();
    let date_text = today.format("%A, %B %d, %Y").to_string();

    let mut file = OpenOptions::new()
        .append(true)
        .open(journal_path)
        .context("Failed to open journal file")?;

    writeln!(file, "# {}", date_text)?;
    writeln!(file)?; // Add an extra newline
    writeln!(file, "| device  | location     | weather    |")?;
    writeln!(file, "| ------- | ------------ | ---------- |")?;

    let device = get_device_info();
    let location = get_location()?;
    let weather = get_weather(&location)?;

    writeln!(file, "| {} | {} | {} |", device, location, weather)?;

    open_in_editor(journal_path)
}

pub fn open_journal_entry() -> Result<()> {
    let journal_path = get_todays_journal_path()?;

    if !std::path::Path::new(&journal_path).exists() {
        anyhow::bail!("Journal file not found: {}", journal_path);
    }

    open_in_editor(&journal_path)
}

pub fn open_journal_entry_by_date(date_str: &str) -> Result<()> {
    // Parse the date string (YYYY-MM-DD)
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .with_context(|| format!("Invalid date format: {}. Expected YYYY-MM-DD", date_str))?;

    let year = date.format("%Y").to_string();
    let month = date.format("%m").to_string();
    let month_name = date.format("%b").to_string().to_lowercase();
    let day = date.format("%d").to_string();
    let weekday = date.format("%A").to_string();

    let git_root = get_git_root()?;
    let journal_path = format!(
        "{}/journal/{}/{}-{}/{}_{}.md",
        git_root, year, month, month_name, day, weekday
    );

    if !std::path::Path::new(&journal_path).exists() {
        anyhow::bail!("Journal file not found: {}", journal_path);
    }

    open_in_editor(&journal_path)
}

pub fn create_year(year: u32) -> Result<()> {
    let git_root = get_git_root()?;
    let year_folder = format!("{}/journal/{}", git_root, year);

    // Create year folder and journey file
    fs::create_dir_all(&year_folder)?;
    File::create(format!("{}/{}_journey.md", year_folder, year))?;

    // Create each month
    for month in 1..=12 {
        let date = NaiveDate::from_ymd_opt(year as i32, month, 1)
            .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;

        let month_name = date.format("%B").to_string();
        let month_name_short = date.format("%b").to_string().to_lowercase();
        let month_folder = format!("{}/{:02}-{}", year_folder, month, month_name_short);

        // Create month folder
        fs::create_dir_all(&month_folder)?;

        // Create monthly files
        File::create(format!(
            "{}/{} {} Happenings.md",
            month_folder, month_name, year
        ))?;
        File::create(format!("{}/{} goals.md", month_folder, month_name))?;

        // Create daily files
        let days_in_month = date
            .with_day(1)
            .and_then(|d| d.with_month(month + 1))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.pred_opt())
            .map(|d| d.day())
            .unwrap_or(31);

        for day in 1..=days_in_month {
            let date = NaiveDate::from_ymd_opt(year as i32, month, day)
                .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;
            let weekday = date.format("%A").to_string();
            let day_file = format!("{}/{:02}_{}.md", month_folder, day, weekday);
            File::create(day_file)?;
        }
    }

    println!("Created journal structure for year {}", year);
    Ok(())
}

pub fn find_empty_day(year_filter: Option<u32>) -> Result<()> {
    let git_root = get_git_root()?;
    let journal_path = format!("{}/journal", git_root);
    let current_year = Local::now().year();
    let mut empty_files = Vec::new();

    // Walk through the directory
    for entry in walkdir::WalkDir::new(&journal_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            // Check if file is empty
            if fs::metadata(path)?.len() == 0 {
                // Check if path contains a year and month pattern
                let path_str = path.to_string_lossy();
                if path_str.contains(|c: char| c.is_ascii_digit()) {
                    if let Some(year_str) = path_str
                        .split('/')
                        .find(|s| s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()))
                    {
                        if let Ok(year) = year_str.parse::<i32>() {
                            // Check year filter if provided
                            if let Some(filter_year) = year_filter {
                                if year != filter_year as i32 {
                                    continue;
                                }
                            } else if year > current_year {
                                // If no filter, skip future years
                                continue;
                            }

                            // Must be DD_Weekday.md format and be in a month folder
                            let file_name = path
                                .file_name()
                                .and_then(|f| f.to_str())
                                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

                            if path_str.contains(|c: char| c == '-') && // Must be in a month folder (contains hyphen)
                               file_name.len() >= 10 && // At least 2 digits + underscore + weekday + .md
                               file_name[..2].chars().all(|c| c.is_ascii_digit()) && // First 2 chars are digits
                               file_name[2..3] == *"_" && // Followed by underscore
                               file_name[3..].ends_with(".md")
                            // Ends with .md
                            {
                                // Verify it's a valid day number (01-31)
                                if let Ok(day) = file_name[..2].parse::<u32>() {
                                    if day >= 1 && day <= 31 {
                                        empty_files.push(path.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if empty_files.is_empty() {
        if let Some(year) = year_filter {
            anyhow::bail!("No empty files found for year {}", year);
        } else {
            anyhow::bail!("No empty files found in journal directory");
        }
    }

    // Select a random file
    let random_file = empty_files
        .choose(&mut rand::thread_rng())
        .ok_or_else(|| anyhow::anyhow!("Failed to select random file"))?;

    // Extract date from path
    let date = extract_date_from_path(random_file)?;

    // Add header to the file
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(random_file)?;

    let current_date = Local::now().format("%m/%d/%Y").to_string();
    writeln!(
        file,
        "# {}\n\n> Transcribed on: {}\n",
        date.format("%A, %B %d, %Y"),
        current_date
    )?;

    println!(
        "Randomly selected empty journal entry: {}",
        date.format("%A, %B %d, %Y")
    );

    // Open the file in the editor
    open_in_editor(random_file.to_str().unwrap())?;

    Ok(())
}

fn extract_date_from_path(path: &PathBuf) -> Result<NaiveDate> {
    let path_str = path.to_string_lossy();
    let parts: Vec<&str> = path_str.split('/').collect();

    // Find year, month, and day from path
    let year = parts
        .iter()
        .find(|s| s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| anyhow::anyhow!("Could not find year in path"))?
        .parse::<i32>()?;

    let month_part = parts
        .iter()
        .find(|s| s.contains('-'))
        .ok_or_else(|| anyhow::anyhow!("Could not find month in path"))?;

    let month = month_part
        .split('-')
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not parse month"))?
        .parse::<u32>()?;

    let day_part = parts
        .last()
        .ok_or_else(|| anyhow::anyhow!("Could not find day in path"))?
        .split('_')
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not parse day"))?
        .parse::<u32>()?;

    NaiveDate::from_ymd_opt(year, month, day_part).ok_or_else(|| anyhow::anyhow!("Invalid date"))
}

#[derive(Debug)]
struct YearStats {
    total_days: u32,
    empty_days: u32,
}

#[derive(Debug)]
struct JournalFile {
    path: PathBuf,
    year: i32,
    month: u32,
    day: u32,
    weekday: String,
}

fn process_journal_files() -> Result<Vec<JournalFile>> {
    let git_root = get_git_root()?;
    let journal_path = format!("{}/journal", git_root);
    let current_year = Local::now().year();
    let mut files = Vec::new();

    // Walk through the directory
    for entry in walkdir::WalkDir::new(&journal_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            let path_str = path.to_string_lossy();

            // Only process files that match the daily journal format: YYYY/MM-monthname/DD_Weekday.md
            if let Some(year_str) = path_str
                .split('/')
                .find(|s| s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()))
            {
                if let Ok(year) = year_str.parse::<i32>() {
                    if year <= current_year {
                        // Check if this is a daily journal file (DD_Weekday.md)
                        let file_name = path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

                        // Must be DD_Weekday.md format and be in a month folder
                        if path_str.contains(|c: char| c == '-') && // Must be in a month folder (contains hyphen)
                           file_name.len() >= 10 && // At least 2 digits + underscore + weekday + .md
                           file_name[..2].chars().all(|c| c.is_ascii_digit()) && // First 2 chars are digits
                           file_name[2..3] == *"_" && // Followed by underscore
                           file_name[3..].ends_with(".md")
                        // Ends with .md
                        {
                            // Verify it's a valid day number (01-31)
                            if let Ok(day) = file_name[..2].parse::<u32>() {
                                if day >= 1 && day <= 31 {
                                    // Extract month from path
                                    let month_part =
                                        path_str.split('/').find(|s| s.contains('-')).ok_or_else(
                                            || anyhow::anyhow!("Could not find month in path"),
                                        )?;

                                    let month = month_part
                                        .split('-')
                                        .next()
                                        .ok_or_else(|| anyhow::anyhow!("Could not parse month"))?
                                        .parse::<u32>()?;

                                    // Get the weekday from the filename
                                    let weekday = file_name[3..file_name.len() - 3].to_string();

                                    files.push(JournalFile {
                                        path: path.to_path_buf(),
                                        year,
                                        month,
                                        day,
                                        weekday,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(files)
}

pub fn analyze_completion() -> Result<()> {
    let files = process_journal_files()?;
    let mut year_stats: HashMap<i32, YearStats> = HashMap::new();

    for file in files {
        let stats = year_stats.entry(file.year).or_insert(YearStats {
            total_days: 0,
            empty_days: 0,
        });

        stats.total_days += 1;
        if fs::metadata(&file.path)?.len() == 0 {
            stats.empty_days += 1;
        }
    }

    // Calculate completion rates and sort years
    let mut years: Vec<_> = year_stats.keys().collect();
    years.sort();

    println!("\nJournal Completion Analysis");
    println!("=========================");
    println!();

    // Find maximum width for completed days (for alignment)
    let max_completed_width = year_stats
        .values()
        .map(|stats| (stats.total_days - stats.empty_days).to_string().len())
        .max()
        .unwrap_or(1);

    // First pass: show detailed stats for each year
    for &year in &years {
        let stats = &year_stats[&year];
        let completed_days = stats.total_days - stats.empty_days;
        let completion_rate = if stats.total_days > 0 {
            ((stats.total_days - stats.empty_days) as f64 / stats.total_days as f64) * 100.0
        } else {
            0.0
        };

        // Create ASCII bar chart (20 characters wide)
        let bar_length = (completion_rate / 5.0).round() as usize; // 5% per character
        let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);

        // For current year, include remaining days
        if *year == Local::now().year() {
            let today = Local::now();
            let days_in_year = if NaiveDate::from_ymd_opt(*year, 12, 31).unwrap().ordinal() == 366 {
                366
            } else {
                365
            };
            let days_remaining = days_in_year - today.ordinal() as u32;
            println!(
                "{} | {:>width$}/{} | {:>6.1}% {} {} days remain",
                year,
                completed_days,
                stats.total_days,
                completion_rate,
                bar,
                days_remaining,
                width = max_completed_width
            );
        } else {
            // Add checkmark if year is complete
            let completion_marker = if completion_rate == 100.0 { " ✓" } else { "" };
            println!(
                "{} | {:>width$}/{} | {:>6.1}% {}{}",
                year,
                completed_days,
                stats.total_days,
                completion_rate,
                bar,
                completion_marker,
                width = max_completed_width
            );
        }
    }

    println!();
    Ok(())
}

pub fn validate_structure() -> Result<()> {
    let files = process_journal_files()?;
    let mut year_stats: HashMap<i32, HashMap<String, Vec<String>>> = HashMap::new();
    let mut seen_dates: HashMap<String, String> = HashMap::new();
    let mut fixed_capitalization = false;

    for file in files {
        // Check if this date exists and matches the weekday
        if let Some(date) = NaiveDate::from_ymd_opt(file.year, file.month, file.day) {
            let actual_weekday = date.format("%A").to_string();
            let date_key = format!("{}-{:02}-{:02}", file.year, file.month, file.day);
            let date_display = date.format("%B %d, %Y").to_string();
            let file_name = file.path.file_name().unwrap().to_string_lossy().to_string();

            // Check for duplicates first
            if let Some(existing_file) = seen_dates.get(&date_key) {
                let year_issues = year_stats.entry(file.year).or_insert(HashMap::new());
                let date_issues = year_issues.entry(date_display).or_insert(Vec::new());
                date_issues.push(format!("Duplicate: {} and {}", file_name, existing_file));
            } else {
                seen_dates.insert(date_key, file_name.clone());

                // Then check weekday mismatch
                if actual_weekday != file.weekday {
                    // Check if it's just a capitalization issue
                    if actual_weekday.to_lowercase() == file.weekday.to_lowercase() {
                        // Fix capitalization
                        let new_name = format!("{:02}_{}.md", file.day, actual_weekday);
                        let new_path = file.path.with_file_name(&new_name);
                        fs::rename(&file.path, &new_path)?;
                        println!("Fixed capitalization: {} -> {}", file_name, new_name);
                        fixed_capitalization = true;
                    } else {
                        let year_issues = year_stats.entry(file.year).or_insert(HashMap::new());
                        let date_issues = year_issues.entry(date_display).or_insert(Vec::new());
                        date_issues.push(format!(
                            "Wrong weekday: {} (should be {})",
                            file_name, actual_weekday
                        ));
                    }
                }
            }
        } else {
            let year_issues = year_stats.entry(file.year).or_insert(HashMap::new());
            let date_display = format!("{}-{:02}-{:02}", file.year, file.month, file.day);
            let date_issues = year_issues.entry(date_display).or_insert(Vec::new());
            date_issues.push(format!(
                "Invalid date: {}",
                file.path.file_name().unwrap().to_string_lossy()
            ));
        }
    }

    // Sort years for consistent output
    let mut years: Vec<_> = year_stats.keys().collect();
    years.sort();

    println!("\nJournal Structure Validation");
    println!("===================================");

    if fixed_capitalization {
        println!("\nFixed capitalization issues in filenames.");
    }

    if years.is_empty() {
        println!("\nNo structural issues found in journal entries. ✓");
        return Ok(());
    }

    for &year in years {
        let year_issues = &year_stats[&year];
        if !year_issues.is_empty() {
            println!("\n{}:", year);
            let mut dates: Vec<_> = year_issues.keys().collect();
            dates.sort();

            for date in dates {
                let issues = &year_issues[date];
                if !issues.is_empty() {
                    println!("  {}:", date);
                    for issue in issues {
                        println!("    {}", issue);
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct DateComponents {
    weekday: Option<String>,
    month: Option<String>,
    day: Option<u32>,
    year: Option<i32>,
}

fn extract_date_components(text: &str, file_date: (i32, u32, u32)) -> DateComponents {
    let text = text.to_lowercase();
    let (file_year, file_month, file_day) = file_date;

    // Extract weekday
    let weekday = [
        ("sunday", vec!["sun"]),
        ("monday", vec!["mon"]),
        ("tuesday", vec!["tue", "tues"]),
        ("wednesday", vec!["wed", "weds"]),
        ("thursday", vec!["thu", "thur", "thurs"]),
        ("friday", vec!["fri"]),
        ("saturday", vec!["sat"]),
    ]
    .iter()
    .find(|(full, short)| {
        let words: Vec<&str> = text.split_whitespace().collect();
        words.iter().any(|word| {
            word.to_lowercase() == *full || short.iter().any(|&s| word.to_lowercase() == s)
        })
    })
    .map(|(full, _)| full.to_string());

    // Extract month name first
    let month = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
        "jan",
        "feb",
        "mar",
        "apr",
        "jun",
        "jul",
        "aug",
        "sep",
        "sept",
        "oct",
        "nov",
        "dec",
    ]
    .iter()
    .find(|&&month| text.contains(month))
    .map(|&month| month.to_string());

    // Extract all numbers from the text
    let mut numbers: Vec<u32> = text
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse::<u32>().ok())
        .filter(|&n| n > 0)
        .collect();

    // If we only have one number, it must be the day
    if numbers.len() == 1 {
        return DateComponents {
            weekday,
            month,
            day: Some(numbers[0]),
            year: Some(file_year),
        };
    }

    // If we have two numbers, one is day and the other might be month (if month not found in text)
    // or year (if month was found in text)
    if numbers.len() == 2 {
        numbers.sort(); // Smaller number is likely day
        let day = Some(numbers[0]);

        // If we found a month name, the second number is likely year
        if month.is_some() {
            let year = if numbers[1] >= 1000 {
                Some(numbers[1] as i32)
            } else {
                Some(file_year)
            };
            return DateComponents {
                weekday,
                month,
                day,
                year,
            };
        } else {
            // No month name found, so second number might be month
            // But only if it's ≤ 12, otherwise assume it's year
            if numbers[1] <= 12 {
                return DateComponents {
                    weekday,
                    month,
                    day,
                    year: Some(file_year),
                };
            }
        }
    }

    // For three numbers, check if they match file date exactly
    if numbers.len() == 3 {
        numbers.sort();
        if numbers.contains(&file_day) && numbers.contains(&file_month) {
            let year = numbers
                .iter()
                .find(|&&n| n >= 1000)
                .map(|&n| n as i32)
                .unwrap_or(file_year);

            return DateComponents {
                weekday,
                month,
                day: Some(file_day), // Use file_day since we found it in numbers
                year: Some(year),
            };
        }
    }

    // For any other case or if the above didn't match
    let mut day = None;
    let mut year = None;

    // Find the year first (largest number ≥ 1000)
    if let Some(&largest) = numbers.iter().rev().find(|&&n| n >= 1000) {
        year = Some(largest as i32);
        numbers.retain(|&n| n != largest);
    }

    // Look for exact day match
    if numbers.contains(&file_day) {
        day = Some(file_day);
        numbers.retain(|&n| n != file_day);
    } else {
        // If no exact match, smallest remaining number ≤ 31 is day
        if let Some(&smallest) = numbers.iter().find(|&&n| n <= 31) {
            day = Some(smallest);
        }
    }

    // If no year found in numbers, use file year
    if year.is_none() {
        year = Some(file_year);
    }

    DateComponents {
        weekday,
        month,
        day,
        year,
    }
}

#[derive(Debug)]
struct ValidationResult {
    header_issues: Vec<String>,
    nav_issues: Vec<String>,
    link_issues: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        ValidationResult {
            header_issues: Vec::new(),
            nav_issues: Vec::new(),
            link_issues: Vec::new(),
        }
    }

    fn has_issues(&self) -> bool {
        !self.header_issues.is_empty()
            || !self.nav_issues.is_empty()
            || !self.link_issues.is_empty()
    }
}

fn validate_header(header: &str, file: &JournalFile) -> Vec<String> {
    let mut issues = Vec::new();

    // Guard: Skip headers without numbers
    if !header.chars().any(|c| c.is_ascii_digit()) {
        return issues;
    }

    let components = extract_date_components(header, (file.year, file.month, file.day));

    // Check weekday if present
    if let Some(weekday) = &components.weekday {
        let file_weekday = file.weekday.to_lowercase();
        if !file_weekday.contains(weekday) && !weekday.contains(&file_weekday) {
            issues.push(format!(
                "Weekday mismatch: Header has '{}' but file indicates '{}'",
                weekday, file.weekday
            ));
        }
    }

    // Check year if present and if it was explicitly in the header
    if let Some(year) = components.year {
        if year != file.year {
            // Only report year mismatch if it was explicitly in the header
            let year_str = year.to_string();
            if header.contains(&year_str) {
                issues.push(format!(
                    "Year mismatch: Header has {} but file indicates {}",
                    year, file.year
                ));
            }
        }
    }

    // Check month if present
    if let Some(month) = &components.month {
        let file_month = chrono::NaiveDate::from_ymd_opt(2000, file.month, 1)
            .map(|d| d.format("%B").to_string().to_lowercase())
            .unwrap_or_default();
        let file_month_short = chrono::NaiveDate::from_ymd_opt(2000, file.month, 1)
            .map(|d| d.format("%b").to_string().to_lowercase())
            .unwrap_or_default();

        if !month.contains(&file_month)
            && !file_month.contains(month)
            && !month.contains(&file_month_short)
            && !file_month_short.contains(month)
        {
            issues.push(format!(
                "Month mismatch: Header has '{}' but file indicates month {}",
                month, file.month
            ));
        }
    }

    // Check day if present
    if let Some(day) = components.day {
        if day != file.day {
            issues.push(format!(
                "Day mismatch: Header has {} but file indicates {}",
                day, file.day
            ));
        }
    }

    issues
}

fn validate_nav(_contents: &str, _file: &JournalFile) -> Vec<String> {
    // TODO: Implement navigation validation
    // This should check for:
    // - Previous/Next day links
    // - Month/Year navigation
    // - Consistency with actual file structure
    Vec::new()
}

fn validate_links(_contents: &str, _file: &JournalFile) -> Vec<String> {
    // TODO: Implement link validation
    // This should check for:
    // - Broken internal links to other journal entries
    // - Malformed markdown links
    // - Links to non-existent files
    Vec::new()
}

pub fn validate_contents() -> Result<()> {
    let files = process_journal_files()?;
    let mut year_stats: HashMap<i32, HashMap<String, Vec<ValidationResult>>> = HashMap::new();
    let mut has_issues = false;

    for file in files {
        // Guard: Skip empty files
        let contents = match fs::read_to_string(&file.path) {
            Ok(content) if content.is_empty() => continue,
            Ok(content) => content,
            Err(_) => continue,
        };

        // Extract header
        let header = match contents.lines().find(|line| line.starts_with("# ")) {
            Some(h) => h.trim_start_matches("# ").trim(),
            None => continue,
        };

        let mut validation = ValidationResult::new();

        // Perform all validations
        validation.header_issues = validate_header(header, &file);
        validation.nav_issues = validate_nav(&contents, &file);
        validation.link_issues = validate_links(&contents, &file);

        // Record issues if any found
        if validation.has_issues() {
            has_issues = true;
            let year_issues = year_stats.entry(file.year).or_insert(HashMap::new());
            let date_display = format!("{}-{:02}-{:02}", file.year, file.month, file.day);
            let date_issues = year_issues.entry(date_display).or_insert(Vec::new());
            date_issues.push(validation);
        }
    }

    println!("\nJournal Content Validation");
    println!("===================================");

    if !has_issues {
        println!("\nNo content issues found in journal entries. ✓");
        return Ok(());
    }

    // Sort years for consistent output
    let mut years: Vec<_> = year_stats.keys().collect();
    years.sort();

    for &year in years {
        let year_issues = &year_stats[&year];
        if !year_issues.is_empty() {
            println!("\n{}:", year);
            let mut dates: Vec<_> = year_issues.keys().collect();
            dates.sort();

            for date in dates {
                let validations = &year_issues[date];
                if !validations.is_empty() {
                    println!("  {}:", date);
                    for validation in validations {
                        // Print header issues
                        for issue in &validation.header_issues {
                            println!("    {}", issue);
                        }
                        // Print nav issues
                        for issue in &validation.nav_issues {
                            println!("    Navigation: {}", issue);
                        }
                        // Print link issues
                        for issue in &validation.link_issues {
                            println!("    Links: {}", issue);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn analyze_length() -> Result<()> {
    let files = process_journal_files()?;
    let mut year_stats: HashMap<i32, (u64, u64, u32)> = HashMap::new(); // (total_words, total_lines, entry_count)

    for file in files {
        // Skip empty files
        let contents = match fs::read_to_string(&file.path) {
            Ok(content) if content.is_empty() => continue,
            Ok(content) => content,
            Err(_) => continue,
        };

        let line_count = contents.lines().count() as u64;
        let word_count = contents.split_whitespace().count() as u64;

        let stats = year_stats.entry(file.year).or_insert((0, 0, 0));
        stats.0 += word_count;
        stats.1 += line_count;
        stats.2 += 1;
    }

    println!("\nJournal Length Analysis");
    println!("===================================");

    // Sort years for consistent output
    let mut years: Vec<_> = year_stats.keys().collect();
    years.sort();

    // Calculate averages and find maximums
    let mut avg_stats: Vec<(i32, f64, f64)> = Vec::new(); // (year, avg_words, avg_lines)
    let mut max_avg_words: f64 = 0.0;
    let mut max_avg_lines: f64 = 0.0;

    for &year in &years {
        let (total_words, total_lines, entry_count) = year_stats[&year];
        if entry_count > 0 {
            let avg_words = total_words as f64 / entry_count as f64;
            let avg_lines = total_lines as f64 / entry_count as f64;
            max_avg_words = f64::max(max_avg_words, avg_words);
            max_avg_lines = f64::max(max_avg_lines, avg_lines);
            avg_stats.push((*year, avg_words, avg_lines));

            println!("\n{}:", year);
            println!("  Entries: {}", entry_count);
            println!("  Average Words: {:.1}", avg_words);
            println!("  Average Lines: {:.1}", avg_lines);
            println!("  Total Words: {}", total_words);
            println!("  Total Lines: {}", total_lines);
        }
    }

    // Print word count chart
    println!("\nAverage Words Per Entry");
    println!("===================================");

    // Calculate scale for words (round up to nearest 100 for cleaner numbers)
    let word_scale = ((max_avg_words / 100.0).ceil() * 100.0) / 10.0;
    let word_percentages = (0..=10)
        .map(|i| word_scale * (10 - i) as f64)
        .collect::<Vec<_>>();

    // Print y-axis labels and bars for words
    for &value in &word_percentages {
        let mut line = format!("{:5.0} │", value);
        for &(_, avg_words, _) in &avg_stats {
            if avg_words >= value {
                line.push('█');
            } else {
                line.push(' ');
            }
            line.push(' ');
        }
        println!("{}", line);
    }

    // Print x-axis
    let axis_length = avg_stats.len() * 2;
    println!("      └{}", "─".repeat(axis_length));

    // Print year labels vertically
    for digit in 0..4 {
        let mut line = String::from("       ");
        for &(year, _, _) in &avg_stats {
            let year_str = year.to_string();
            if digit < year_str.len() {
                line.push(year_str.chars().nth(digit).unwrap());
            } else {
                line.push(' ');
            }
            line.push(' ');
        }
        println!("{}", line);
    }

    // Print line count chart
    println!("\nAverage Lines Per Entry");
    println!("===================================");

    // Calculate scale for lines (round up to nearest 10 for cleaner numbers)
    let line_scale = ((max_avg_lines / 10.0).ceil() * 10.0) / 10.0;
    let line_percentages = (0..=10)
        .map(|i| line_scale * (10 - i) as f64)
        .collect::<Vec<_>>();

    // Print y-axis labels and bars for lines
    for &value in &line_percentages {
        let mut line = format!("{:5.0} │", value);
        for &(_, _, avg_lines) in &avg_stats {
            if avg_lines >= value {
                line.push('█');
            } else {
                line.push(' ');
            }
            line.push(' ');
        }
        println!("{}", line);
    }

    // Print x-axis
    println!("      └{}", "─".repeat(axis_length));

    // Print year labels vertically
    for digit in 0..4 {
        let mut line = String::from("       ");
        for &(year, _, _) in &avg_stats {
            let year_str = year.to_string();
            if digit < year_str.len() {
                line.push(year_str.chars().nth(digit).unwrap());
            } else {
                line.push(' ');
            }
            line.push(' ');
        }
        println!("{}", line);
    }

    Ok(())
}

pub fn add_custom_header(header: &str) -> Result<()> {
    let journal_path = get_todays_journal_path()?;
    let path = std::path::Path::new(&journal_path);

    // Check if file is empty
    let is_empty = path.metadata()?.len() == 0;

    // If file is empty, create standard header first
    if is_empty {
        create_journal_entry(&journal_path)?;
    }

    // Open file for appending
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .context("Failed to open journal file")?;

    // Add a newline before the custom header
    writeln!(file)?;

    // Add the custom header
    writeln!(file, "## {}", header)?;

    // Open the file in the editor
    open_in_editor(&journal_path)
}
