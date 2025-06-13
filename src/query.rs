use crate::tables;

use serde_json::Value;
use sqlparser::ast::Select;
use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

pub fn fetch_data(select_statement: &Select, files: &Vec<String>) -> Result<HashMap<String, Value>, Box<dyn Error>> {
    let tables_to_files = tables::tables_from_files(files)?;

    let tables = tables::tables_from_select(select_statement)?;

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

    Ok(data)
}