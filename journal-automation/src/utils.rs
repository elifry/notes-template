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
    // Try to open with Cursor first, then VS Code
    let cursor_result = Command::new("cursor").arg(file_path).spawn();

    if cursor_result.is_err() {
        Command::new("code")
            .arg(file_path)
            .spawn()
            .context("Failed to open file in VS Code")?;
    }

    Ok(())
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
