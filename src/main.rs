mod cli;
mod query;
mod tables;

use json_value_merge::Merge;
use serde_json::json;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::parse()?;

    let select_statement =  cli::select_statement_from(&args)?;

    let files = cli::files_from(&args)?;

    let data = query::fetch_data(&select_statement, &files)?;

    // Apply filters

    // Do joins

    // Do aggregations

    // Print results

    // If no table names were specified in the SQL query, we will merge all JSON data
    // from the provided files and print it to stdout.
    // If table names were specified, we will only print the JSON data for those tables.
    let tables = tables::tables_from_select(&select_statement)?;
    if tables.is_empty() {
        let mut merged_data = json!("{}");
        for value in data.values() {
            merged_data.merge(value);
        }

        if cli::pretty(&args) {
            serde_json::to_writer_pretty(std::io::stdout(), &merged_data)?;
        } else {
            serde_json::to_writer(std::io::stdout(), &merged_data)?;
        }
    } else {
        for table_name in &tables {
            if let Some(value) = data.get(table_name) {
                if cli::pretty(&args) {
                    serde_json::to_writer_pretty(std::io::stdout(), value)?;
                } else {
                    serde_json::to_writer(std::io::stdout(), value)?;
                }
            } else {
                return Err(format!("Table '{}' not found in the provided files.", table_name).into());
            }
        }
    }

    Ok(())
}
