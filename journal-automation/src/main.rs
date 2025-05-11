mod cli;
mod journal;
mod schedule;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::StartJournal { class } => {
            let journal_path = journal::get_todays_journal_path(&class)?;
            journal::create_journal_entry(&journal_path, &class)?;
        }
        Commands::OpenJournal { class } => {
            journal::open_journal_entry(&class)?;
        }
        Commands::OpenDay { date, class } => {
            journal::open_journal_entry_by_date(&date, &class)?;
        }
        Commands::CreateYear { year, class } => {
            journal::create_year(year, &class)?;
        }
        Commands::EmptyDay { year } => {
            journal::find_empty_day(year)?;
        }
        Commands::AddCustomHeader { header } => {
            journal::add_custom_header(&header)?;
        }
        Commands::AnalyzeCompletion => {
            journal::analyze_completion()?;
        }
        Commands::AnalyzeLength => {
            journal::analyze_length()?;
        }
        Commands::ValidateStructure => {
            journal::validate_structure()?;
        }
        Commands::ValidateContents => {
            journal::validate_contents()?;
        }
    }

    Ok(())
}
