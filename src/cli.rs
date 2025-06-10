use clap::Parser as _;
use sqlparser::{dialect::GenericDialect, parser::Parser, ast::Select};
use std::{error::Error, path::Path};

/// A simple CLI tool to process JSON files using SQL queries.
#[derive(Debug, clap::Parser)]
#[command(version, about)]
pub struct Cli {
    #[clap(required = true, short, long)]
    /// The input file to process. May be specified multiple times.
    file: Option<Vec<String>>,

    #[arg(value_name = "SQL", required = true)]
    /// Optional SQL query to execute
    sql: Option<String>,

    #[arg(short, long, default_value = "false")]
    pretty: bool
}

pub fn parse() -> Result<Cli, Box<dyn Error>> {
    // Parse the command line arguments
    let args = Cli::parse();
    Ok(args)
}

/// Function to parse the SQL argument from the command line.
/// 
/// This function checks if the SQL query is provided, parses it, and returns a Select statement.
/// 
/// If the SQL query is not provided or is invalid, it returns an error.
/// 
/// If the SQL query is empty, it returns an error.
/// 
/// If the SQL query is not a valid SELECT statement, it returns an error.
pub fn select_statement_from(args: &Cli) -> Result<Select, Box<dyn Error + 'static>> {
    let sql = match &args.sql {
        Some(sql) => sql,
        None => {
            return Err("No SQL query provided.".into());
        }
    };

    if sql.trim().is_empty() {
        return Err("Empty SQL query provided.".into());
    }
    let statements = Parser::parse_sql(&GenericDialect {}, &sql);

    let statements = match statements {
        Ok(statements) => statements,
        Err(e) => return Err(format!("Failed to parse SQL: {}", e).into()),
    };

    if statements.len() != 1 {
        return Err("Expected exactly one SQL statement.".into());
    }

    let statement = &statements[0];

        // Check if the statement is a query
    let query = match statement {
        sqlparser::ast::Statement::Query(query) => query,
        _ => {
            return Err("Unsupported SQL statement type.".into());
        }
    };

    let select = match query.body.as_select() {
        Some(select) => select,
        None => {
            return Err("The provided SQL statement is not a SELECT query.".into());
        }
    };

    let select = select.to_owned();

    Ok(select)
}

/// Function to parse the file arguments from the command line.
/// 
/// This function checks if the files exist and are readable, and returns a vector of file paths.
/// 
/// If no files are provided, it returns an error.
/// 
/// If any file does not exist or is not readable, it returns an error.
pub fn files_from(args: &Cli) -> Result<Vec<String>, Box<dyn Error>> {
    // Gather the file paths from the command line arguments
    let files = match &args.file {
        Some(files) => files,
        None => {
            return Err("No input files provided.".into());
        }
    };

    // Check if any files were provided
    if files.is_empty() {
        return Err("No input files provided.".into());
    }

    // Verify that the files exist and are readable
    for file in files {
        let path = Path::new(&file);
        if !path.try_exists().unwrap_or(false) {
            return Err(format!("File '{}' does not exist or is not readable.", file).into());
        }
        if !path.is_file() {
            return Err(format!("'{}' is not a valid file.", file).into());
        }
    }

    let files = files.clone();

    Ok(files)
}

pub fn pretty(args: &Cli) -> bool {
    args.pretty
}