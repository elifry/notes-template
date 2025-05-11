use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassSchedule {
    pub class_name: String,
    pub start_date: String, // YYYY-MM-DD
    pub end_date: String,   // YYYY-MM-DD
    pub schedule: Vec<ClassDay>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassDay {
    pub weekday: Weekday,
    pub start_time: String, // HH:MM in 24-hour format
    pub end_time: String,   // HH:MM in 24-hour format
    pub location: Option<String>,
    pub instructor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Weekday::Monday => write!(f, "Monday"),
            Weekday::Tuesday => write!(f, "Tuesday"),
            Weekday::Wednesday => write!(f, "Wednesday"),
            Weekday::Thursday => write!(f, "Thursday"),
            Weekday::Friday => write!(f, "Friday"),
            Weekday::Saturday => write!(f, "Saturday"),
            Weekday::Sunday => write!(f, "Sunday"),
        }
    }
}

impl ClassSchedule {
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read schedule file: {}", path))?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse schedule file: {}", path))
    }

    pub fn get_class_dates(&self) -> Result<Vec<NaiveDate>> {
        let start_date = NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d")
            .with_context(|| format!("Invalid start date format: {}", self.start_date))?;
        let end_date = NaiveDate::parse_from_str(&self.end_date, "%Y-%m-%d")
            .with_context(|| format!("Invalid end date format: {}", self.end_date))?;

        // Create a set of weekdays that have classes
        let class_weekdays: HashSet<Weekday> = self
            .schedule
            .iter()
            .map(|day| day.weekday.clone())
            .collect();

        let mut class_dates = Vec::new();
        let mut current_date = start_date;

        while current_date <= end_date {
            let weekday = match current_date.weekday() {
                chrono::Weekday::Mon => Weekday::Monday,
                chrono::Weekday::Tue => Weekday::Tuesday,
                chrono::Weekday::Wed => Weekday::Wednesday,
                chrono::Weekday::Thu => Weekday::Thursday,
                chrono::Weekday::Fri => Weekday::Friday,
                chrono::Weekday::Sat => Weekday::Saturday,
                chrono::Weekday::Sun => Weekday::Sunday,
            };

            if class_weekdays.contains(&weekday) {
                class_dates.push(current_date);
            }

            current_date = current_date
                .succ_opt()
                .ok_or_else(|| anyhow::anyhow!("Failed to get next date"))?;
        }

        Ok(class_dates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_parsing() {
        let json = r#"{
            "class_name": "CS101",
            "start_date": "2024-01-15",
            "end_date": "2024-05-15",
            "schedule": [
                {
                    "weekday": "monday",
                    "start_time": "10:00",
                    "end_time": "11:30",
                    "location": "Room 101",
                    "instructor": "Dr. Smith"
                },
                {
                    "weekday": "wednesday",
                    "start_time": "10:00",
                    "end_time": "11:30",
                    "location": "Room 101",
                    "instructor": "Dr. Smith"
                }
            ]
        }"#;

        let schedule: ClassSchedule = serde_json::from_str(json).unwrap();
        assert_eq!(schedule.class_name, "CS101");
        assert_eq!(schedule.schedule.len(), 2);
    }

    #[test]
    fn test_get_class_dates() {
        let json = r#"{
            "class_name": "CS101",
            "start_date": "2024-01-15",
            "end_date": "2024-01-22",
            "schedule": [
                {
                    "weekday": "monday",
                    "start_time": "10:00",
                    "end_time": "11:30"
                },
                {
                    "weekday": "wednesday",
                    "start_time": "10:00",
                    "end_time": "11:30"
                }
            ]
        }"#;

        let schedule: ClassSchedule = serde_json::from_str(json).unwrap();
        let dates = schedule.get_class_dates().unwrap();

        // Should have 2 dates: Jan 15 (Monday) and Jan 17 (Wednesday)
        assert_eq!(dates.len(), 2);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
    }
}
