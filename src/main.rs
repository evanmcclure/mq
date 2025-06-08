mod cli;

use sqlparser::ast::{Select, TableFactor::Table};
use std::{collections::{HashMap, HashSet}, error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::parse()?;

    let select_statement =  cli::parse_sql_arg(&args)?;

    let files = cli::parse_file_args(&args)?;

    let table_names_to_file_paths = get_table_names_from_file_paths(&files)?;
    println!("Files to process: {:?}", table_names_to_file_paths);

    let table_names = get_table_names_from_select(&select_statement)?;
    println!("Table names found in the query: {:?}", table_names);

    // Validate that the table names in the SQL query exist in the provided files
    for table_name in &table_names {
        if !table_names_to_file_paths.contains_key(table_name) {
            return Err(format!("Table '{}' not found in the provided files.", table_name).into());
        }
    }

    // let serialized = serde_json::to_string_pretty(&query);
    // let serialized = match serialized {
    //     Ok(json) => json,
    //     Err(e) => return Err(format!("Failed to serialize SQL statements: {}", e).into()),
    // };
    // println!("{}", serialized);

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
                let table_name = match path.file_stem() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                return Err(format!("Could not extract a valid table name from '{}'.", file).into());
            }   
        };

        let table_name = table_name.to_uppercase();

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

            let table_name = table_name.to_uppercase();

            table_names.insert(table_name);
        }
    }

    Ok(table_names)
}