use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead, BufReader, Error};
use std::path::{Path, PathBuf};

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

    match std::fs::create_dir_all(&dir) {
        Ok(_) => {}
        Err(e) => return Err(e.to_string()),
    };

    Ok(dir)
}

/// The case order to be followed by the current session. This is stored in the
/// configuration.
#[derive(Default, Debug)]
pub enum CaseOrder {
    #[default]
    European,
    English,
}

impl CaseOrder {
    /// Returns the current case order as a bunch of integers which represent
    /// each case on a given order.
    pub fn to_usizes(&self) -> [usize; 7] {
        match self {
            CaseOrder::European => [0, 1, 2, 3, 4, 5, 6],
            CaseOrder::English => [0, 3, 4, 2, 5, 1, 6],
        }
    }
}

/// Representation for languages supported by this application.
#[derive(Clone, Debug, Default)]
pub enum Language {
    #[default]
    Unknown = 0,
    Latin,
}

impl TryFrom<isize> for Language {
    type Error = &'static str;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Latin),
            _ => Err("unknonwn language!"),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Latin => write!(f, "latin"),
        }
    }
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
        Err(e) => return Err(format!("could not create file: {e}")),
    };
    match file.write_all(language.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not save language '{language}': {e}")),
    }
}

/// Configuration object for this application. Obtain this via the
/// `configuration` function.
#[derive(Debug)]
pub struct Configuration {
    pub language: Language,
    pub case_order: CaseOrder,
}

/// Reads the global configuration and returns a proper object for it. It will
/// assume some defaults if there is something that goes wrong when reading it.
pub fn configuration() -> Configuration {
    let order = read_line_from(1).unwrap_or(String::from("european"));
    let case_order = match order.as_str() {
        "english" => CaseOrder::English,
        _ => CaseOrder::European,
    };

    Configuration {
        language: Language::Latin,
        case_order,
    }
}

// Read a specific line from the configuration and return a String.
fn read_line_from(line: usize) -> Result<String, Error> {
    let path = get_config_path().map_err(std::io::Error::other)?;
    let cfg = path.join("languages.txt");

    let file = File::open(cfg)?;
    let reader = BufReader::new(file);

    let line = reader
        .lines()
        .nth(line)
        .transpose()?
        .ok_or_else(|| io::Error::other("line not found"))?;

    Ok(line)
}
