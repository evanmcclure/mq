mod cli;

use serde_json::{json, Value};
use sqlparser::ast::{Select, TableFactor::Table};
use std::{collections::{HashMap, HashSet}, error::Error, fs::File, io::BufReader, path::Path};
use json_value_merge::Merge;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::parse()?;

    let select_statement =  cli::parse_sql_arg(&args)?;

    let files = cli::parse_file_args(&args)?;

    let table_names_to_file_paths = get_table_names_from_file_paths(&files)?;

    let table_names = get_table_names_from_select(&select_statement)?;

    // Verify that the table names in the SQL query exist in the provided files
    for table_name in &table_names {
        if !table_names_to_file_paths.contains_key(table_name) {
            return Err(format!("Table '{}' not found in the provided files.", table_name).into());
        }
    }

    // Deserialize the JSON files into serde_json::Value and
    // collect them into a map where the keys are the table names
    // and the values are the JSON data.
    let mut data: HashMap<String, Value> = HashMap::new();
    for (table_name, path) in table_names_to_file_paths {
        let file =  File::open(path)?;
        let reader = BufReader::new(file);
        let value: Value = serde_json::from_reader(reader)?;
        data.insert(table_name, value);
    }

    // If no table names were specified in the SQL query, we will merge all JSON data
    // from the provided files and print it to stdout.
    // If table names were specified, we will only print the JSON data for those tables.
    if table_names.is_empty() {
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
    }

    Ok(())
}

/// Function to get the table names from the provided files.
/// 
/// This function collects the basenames of the files and maps them to their file paths.
/// 
/// It returns a HashMap where the keys are the table names (in uppercase) and the values are the file paths.
/// 
/// If a file does not have a valid basename, it returns an error.
fn get_table_names_from_file_paths(files: &Vec<String>) -> Result<HashMap<std::string::String, &Path>, Box<dyn std::error::Error>> {
    // Collect the basenames of the files mapped to their file paths
    // This will be used to validate the table names in the SQL query
    let mut table_names_to_file_paths = HashMap::new();

    for file in files {
        let path = Path::new(file);

        let file_name = match path.file_stem() {
            Some(file_name) => file_name.to_string_lossy().to_string(),
            None => {
                return Err(format!("Could not extract a valid table name from '{}'.", file).into());
            }
        };

        let table_name = sql_table_name(&file_name);

        table_names_to_file_paths.insert(table_name, path);
    }

    Ok(table_names_to_file_paths)
}

/// Function to get the table names from the SQL SELECT statement.
/// 
/// This function extracts the table names from the FROM clause of the SELECT statement.
/// 
/// It returns a HashSet of table names.
fn get_table_names_from_select(select: &Select) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut table_names = HashSet::new();
    if !select.from.is_empty() {
        let tables: Vec<_> = select.from.iter().collect();
        for table in tables {
            let relation = &table.relation;

            let table_name = match relation {
                Table{name, ..} => name.to_string(),
                _ => {
                    return Err("The provided SQL statement does not specify a table.".into());
                },
            };

            let table_name = sql_table_name(&table_name);

            table_names.insert(table_name);
        }
    }

    Ok(table_names)
}

fn sql_table_name(file_name: &str) -> String {
    let table_name = file_name.to_uppercase();
     // Replace hyphens with underscores for to make it a valid SQL table name
    table_name.replace("-","_")
}