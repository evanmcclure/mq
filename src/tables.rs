use sqlparser::ast::{Select, TableFactor::Table};
use std::{collections::{HashMap, HashSet}, error::Error, path::Path};

pub fn verify_files_and_tables_match(table_names_to_file_paths: &HashMap<String, &Path>, table_names: &HashSet<String>) -> Result<(), Box<dyn Error + 'static>> {
    // Verify that the table names in the SQL query exist in the provided files
    for table_name in table_names {
        if !table_names_to_file_paths.contains_key(table_name) {
            return Err(format!("Table '{}' not found in the provided files.", table_name).into());
        }
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
pub fn tables_from_files(files: &Vec<String>) -> Result<HashMap<std::string::String, &Path>, Box<dyn std::error::Error>> {
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
pub fn tables_from_select(select: &Select) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
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
