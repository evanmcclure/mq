mod cli;
mod tables;

use serde_json::{json, Value};
use std::{collections::HashMap, error::Error, fs::File, io::BufReader};
use json_value_merge::Merge;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::parse()?;

    let select_statement =  cli::select_statement_from(&args)?;

    let files = cli::files_from(&args)?;

    let tables_to_files = tables::tables_from_files(&files)?;

    let tables = tables::tables_from_select(&select_statement)?;

    tables::verify_files_and_tables_match(&tables_to_files, &tables)?;

    // Deserialize the JSON files into serde_json::Value and
    // collect them into a map where the keys are the table names
    // and the values are the JSON data.
    let mut data: HashMap<String, Value> = HashMap::new();
    for (table_name, path) in tables_to_files {
        let file =  File::open(path)?;
        let reader = BufReader::new(file);
        let value: Value = serde_json::from_reader(reader)?;
        data.insert(table_name, value);
    }

    // If no table names were specified in the SQL query, we will merge all JSON data
    // from the provided files and print it to stdout.
    // If table names were specified, we will only print the JSON data for those tables.
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
        // If table names were specified, we will print the JSON data for those tables
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
