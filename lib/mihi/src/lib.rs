use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

mod migrate;

/// Returns the configuration path for the application, and it even creates it
/// if it doesn't exist already.
pub fn get_config_path() -> Result<PathBuf, String> {
    let dir = match &std::env::var("XDG_CONFIG_HOME") {
        Ok(path) => PathBuf::from(path),
        Err(_) => match &std::env::var("HOME") {
            Ok(path) => Path::new(path).join(".config"),
            Err(_) => {
                return Err(String::from(
                    "cannot find a suitable path for the configuration",
                ))
            }
        },
    }
    .join("mihi");

    match fs::create_dir_all(&dir) {
        Ok(_) => {}
        Err(e) => return Err(e.to_string()),
    };

    Ok(dir)
}

/// Add the given language into the configuration of this application.
pub fn add_language(language: String) -> Result<(), String> {
    if language.as_str() != "latin" {
        return Err(String::from("only 'latin' is allowed for a language"));
    }

    let path = get_config_path()?;
    let cfg = path.join("languages.txt");

    if cfg.exists() {
        return Ok(());
    }

    let mut file = match File::create(cfg) {
        Ok(f) => f,
        Err(e) => return Err(format!("could not create file: {}", e)),
    };
    match file.write_all(language.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not save language '{}': {}", language, e)),
    }
}

/// Ensure that in the config path there is a fully initialized database.
pub fn init_database() -> Result<(), String> {
    let path = get_config_path()?.join("database.sqlite3");
    let conn = match Connection::open(path) {
        Ok(handle) => handle,
        Err(e) => {
            return Err(format!(
                "could not initialize the database: {}",
                e.to_string()
            ))
        }
    };

    match migrate::init(conn) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("bad database schema file: {}", e.to_string())),
    }
}

// #[derive(Debug)]
// struct Word {
//     id: i32,
//     enunciated: String,
//     particle: String,
//     language_id: u64,
//     declension_id: u64,
//     conjugation_id: u64,
//     kind: String,
//     category: String,
//     regular: bool,
//     locative: bool,
//     gender: u32,
//     suffix: String,
//     translation: String,
//     // TODO: datetime
//     // TODO: jsonb
// }

pub fn select_enunciated(filter: Option<String>) -> Result<Vec<String>, String> {
    let conn = get_connection()?;

    let mut stmt;
    let mut it = match filter {
        Some(filter) => {
            stmt = conn
                .prepare(
                    "SELECT enunciated FROM words WHERE enunciated LIKE ('%' || ?1 || '%') ORDER BY enunciated",
                )
                .unwrap();
            stmt.query([filter.as_str()]).unwrap()
        }
        None => {
            stmt = conn
                .prepare("SELECT enunciated FROM words ORDER BY enunciated")
                .unwrap();
            stmt.query([]).unwrap()
        }
    };

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(row.get::<usize, String>(0).unwrap());
    }
    Ok(res)
}

pub fn delete_word(enunciated: &String) -> Result<(), String> {
    let conn = get_connection()?;

    match conn.execute(
        "DELETE FROM words WHERE enunciated = ?1",
        params![enunciated.as_str()],
    ) {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("could not remove '{}': {}", enunciated, e)),
    }
}

fn get_connection() -> Result<rusqlite::Connection, String> {
    let path = get_config_path()?.join("database.sqlite3");
    match Connection::open(path) {
        Ok(handle) => Ok(handle),
        Err(_) => {
            return Err(
                "could not fetch the database. Ensure that you have called 'init' first"
                    .to_string(),
            )
        }
    }
}
