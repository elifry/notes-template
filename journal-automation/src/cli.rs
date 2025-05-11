use crate::utils::validate_year;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start today's journal entry
    StartJournal {
        /// Class name (e.g., CS101)
        #[arg(long, default_value = "journal")]
        class: String,
    },
    /// Open today's journal entry
    OpenJournal {
        /// Class name (e.g., CS101)
        #[arg(long, default_value = "journal")]
        class: String,
    },
    /// Open a specific journal entry by date (YYYY-MM-DD)
    OpenDay {
        /// Date in YYYY-MM-DD format
        date: String,
        /// Class name (e.g., CS101)
        #[arg(long, default_value = "journal")]
        class: String,
    },
    /// Create journal structure for an entire year
    CreateYear {
        /// Year to create (2000-2099)
        #[arg(value_parser = validate_year)]
        year: u32,
        /// Class name (e.g., CS101)
        #[arg(default_value = "journal")]
        class: String,
    },
    /// Find and open a random empty journal entry
    EmptyDay {
        /// Optional year to limit search to (2000-2099)
        #[arg(value_parser = validate_year)]
        year: Option<u32>,
    },
    /// Add a custom header to today's journal entry
    AddCustomHeader {
        /// The header text to add
        header: String,
    },
    /// Analyze journal completion rates
    AnalyzeCompletion,
    /// Analyze journal length statistics
    AnalyzeLength,
    /// Validate journal structure against expected dates
    ValidateStructure,
    /// Validate journal contents
    ValidateContents,
}
