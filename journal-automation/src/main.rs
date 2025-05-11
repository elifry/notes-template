mod amazon;
mod cli;
mod journal;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::StartJournal => {
            let journal_path = journal::get_todays_journal_path()?;
            journal::create_journal_entry(&journal_path)?;
        }
        Commands::OpenJournal => {
            journal::open_journal_entry()?;
        }
        Commands::OpenDay { date } => {
            journal::open_journal_entry_by_date(&date)?;
        }
        Commands::CreateYear { year } => {
            journal::create_year(year)?;
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
        Commands::AnalyzeAmazonData => {
            let base_path = std::path::Path::new("../amazon-data");
            amazon::analyze_amazon_data(base_path.to_str().unwrap(), false)?;
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
