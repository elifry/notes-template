use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::process::Command;

pub fn get_device_info() -> String {
    let output = Command::new("ifconfig").arg("en0").output();

    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("fc:e2:6c:18:be:70") {
                "✨ luna".to_string()
            } else {
                "other device".to_string()
            }
        }
        Err(_) => "unknown device".to_string(),
    }
}

pub fn get_location() -> Result<String> {
    let client = Client::new();
    let ip = client.get("https://ipinfo.io/ip").send()?.text()?;

    let response: Value = client
        .get(format!("https://ipinfo.io/{}/geo", ip))
        .send()?
        .json()?;

    Ok(format!(
        "{}, {}",
        response["region"].as_str().unwrap_or("Unknown"),
        response["country"].as_str().unwrap_or("Unknown")
    ))
}

pub fn get_weather(location: &str) -> Result<String> {
    let client = Client::new();
    let response: Value = client
        .get(format!("https://wttr.in/{}?format=j1&u", location))
        .send()?
        .json()?;

    let high_f = response["weather"][0]["maxtempF"].as_str().unwrap_or("N/A");
    let low_f = response["weather"][0]["mintempF"].as_str().unwrap_or("N/A");

    let condition = response["current_condition"][0]["weatherDesc"][0]["value"]
        .as_str()
        .unwrap_or("Unknown");

    let emoji = match condition {
        s if s.contains("Sunny") => "☀️",
        s if s.contains("Partly cloudy") => "⛅",
        s if s.contains("Cloudy") || s.contains("Overcast") => "☁️",
        s if s.contains("Rain") => "🌧️",
        s if s.contains("Thunder") => "⛈️",
        s if s.contains("Snow") => "❄️",
        _ => "🌈",
    };

    Ok(format!("{}-{} F {}", low_f, high_f, emoji))
}

pub fn open_in_editor(file_path: &str) -> Result<()> {
    // Try different methods to open Cursor
    let cursor_commands = [
        ("cursor", vec![file_path]),
        ("cmd", vec!["/c", "cursor", file_path]),
        ("powershell", vec!["-Command", "cursor", file_path]),
    ];

    // Track which methods failed
    let mut failed_methods = Vec::new();
    for (cmd, args) in cursor_commands.iter() {
        let result = Command::new(cmd).args(args).spawn();
        if result.is_ok() {
            return Ok(());
        }
        failed_methods.push((cmd, args.clone()));
    }

    // If we're on Windows and cmd works but powershell doesn't, try to help fix the PATH
    #[cfg(target_os = "windows")]
    {
        if failed_methods.iter().any(|(cmd, _)| *cmd == "powershell") {
            // Check if cursor is in cmd PATH
            if let Ok(output) = Command::new("cmd").args(["/c", "where", "cursor"]).output() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    if !path.trim().is_empty() {
                        // Found cursor in cmd PATH, suggest adding to PowerShell
                        let cursor_path = path.lines().next().unwrap_or("").trim();
                        if !cursor_path.is_empty() {
                            println!("\n[INFO] Cursor found in Command Prompt PATH but not in PowerShell PATH.");
                            println!("To fix this, run the following command in PowerShell as Administrator:");
                            println!("$env:Path += \";{}\"", cursor_path);
                            println!("To make this permanent, add the above line to your PowerShell profile.");
                            println!("You can open your profile with: notepad $PROFILE\n");
                        }
                    }
                }
            }
        }
    }

    // Fallback to VS Code
    let code_commands = [
        ("code", vec![file_path]),
        ("cmd", vec!["/c", "code", file_path]),
        ("powershell", vec!["-Command", "code", file_path]),
    ];

    for (cmd, args) in code_commands.iter() {
        let result = Command::new(cmd).args(args).spawn();
        if result.is_ok() {
            return Ok(());
        }
    }

    anyhow::bail!("Failed to open file in any editor. Please ensure Cursor or VS Code is installed and in your PATH.")
}

pub fn validate_year(s: &str) -> Result<u32, String> {
    let year: u32 = s.parse().map_err(|_| "Year must be a number")?;
    if year >= 2000 && year <= 2099 {
        Ok(year)
    } else {
        Err("Year must be between 2000 and 2099".to_string())
    }
}

pub fn get_journal_path_for_date(date: chrono::NaiveDate) -> Result<String> {
    let year = date.format("%Y").to_string();
    let month = date.format("%m").to_string();
    let month_name = date.format("%b").to_string().to_lowercase();
    let day = date.format("%d").to_string();
    let weekday = date.format("%A").to_string();

    let git_root = get_git_root()?;
    Ok(format!(
        "{}/journal/{}/{}-{}/{}_{}.md",
        git_root, year, month, month_name, day, weekday
    ))
}

pub fn get_git_root() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to execute git command")?;

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
