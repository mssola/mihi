use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

mod migrate;

#[derive(Clone, Copy, Debug, Default)]
pub enum Category {
    #[default]
    Unknown = 0,
    Noun,
    Adjective,
    Verb,
    Pronoun,
    Adverb,
    Preposition,
    Conjunction,
    Interjection,
    Determiner,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Noun => write!(f, "noun"),
            Self::Adjective => write!(f, "adjective"),
            Self::Verb => write!(f, "verb"),
            Self::Pronoun => write!(f, "pronoun"),
            Self::Adverb => write!(f, "adverb"),
            Self::Preposition => write!(f, "preposition"),
            Self::Conjunction => write!(f, "conjunction"),
            Self::Interjection => write!(f, "interjection"),
            Self::Determiner => write!(f, "determiner"),
        }
    }
}

impl TryFrom<usize> for Category {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Noun),
            2 => Ok(Self::Adjective),
            3 => Ok(Self::Verb),
            4 => Ok(Self::Pronoun),
            5 => Ok(Self::Adverb),
            6 => Ok(Self::Preposition),
            7 => Ok(Self::Conjunction),
            8 => Ok(Self::Interjection),
            9 => Ok(Self::Determiner),
            _ => Err("unknonwn category!"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Gender {
    Masculine = 0,
    Feminine,
    MasculineOrFeminine,
    Neuter,
    #[default]
    None,
}

impl TryFrom<usize> for Gender {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Masculine),
            1 => Ok(Self::Feminine),
            2 => Ok(Self::MasculineOrFeminine),
            3 => Ok(Self::Neuter),
            4 => Ok(Self::None),
            _ => Err("unknonwn gender!"),
        }
    }
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Masculine => write!(f, "masculine"),
            Self::Feminine => write!(f, "feminine"),
            Self::MasculineOrFeminine => write!(f, "masculine or feminine"),
            Self::Neuter => write!(f, "neuter"),
            Self::None => write!(f, "none"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum Language {
    #[default]
    Unknown = 0,
    Latin,
}

impl TryFrom<usize> for Language {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
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
        Err(e) => return Err(format!("could not create file: {e}")),
    };
    match file.write_all(language.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not save language '{language}': {e}")),
    }
}

/// Ensure that in the config path there is a fully initialized database.
pub fn init_database() -> Result<(), String> {
    let path = get_config_path()?.join("database.sqlite3");
    let conn = match Connection::open(path) {
        Ok(handle) => handle,
        Err(e) => return Err(format!("could not initialize the database: {e}")),
    };

    match migrate::init(conn) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("bad database schema file: {e}")),
    }
}

#[derive(Clone, Debug, Default)]
pub struct Word {
    pub id: i32,
    pub enunciated: String,
    pub particle: String,
    pub language: Language,
    pub declension_id: Option<usize>,
    pub conjugation_id: Option<usize>,
    pub kind: String,
    pub category: Category,
    pub regular: bool,
    pub locative: bool,
    pub gender: Gender,
    pub suffix: Option<String>,
    pub translation: Value,
    pub flags: Value,
    pub succeeded: usize,
    pub steps: usize,
}

impl Word {
    pub fn from(
        particle: String,
        category: Category,
        declension_id: Option<usize>,
        conjugation_id: Option<usize>,
        gender: Gender,
        kind: String,
    ) -> Word {
        Word {
            id: 0,
            enunciated: "".to_string(),
            particle,
            category,
            declension_id,
            conjugation_id,
            kind,
            regular: true,
            locative: false,
            gender,
            suffix: None,
            language: Language::Latin,
            translation: serde_json::from_str("{}").unwrap(),
            flags: serde_json::from_str("{}").unwrap(),
            succeeded: 0,
            steps: 0,
        }
    }

    pub fn inflection_id(&self) -> usize {
        if matches!(self.category, Category::Verb) {
            return self.conjugation_id.unwrap();
        }
        self.declension_id.unwrap()
    }
}

const DECLENSIONS_WITH_KINDS: &[&[&str]] = &[
    &["a"],
    &["us", "um", "ius", "er/ir"],
    &[
        "is",
        "istem",
        "pureistem",
        "one",
        "onenonistem",
        "two",
        "three",
        "visvis",
        "sussuis",
        "bosbovis",
        "iuppiteriovis",
    ],
    &["fus", "domusdomus"],
    &["ies", "es"],
    &["indeclinable"],
];

const ADJECTIVE_KINDS: &[&[&str]] = &[
    &["us", "er/ir"],
    &[],
    &[
        "one",
        "onenonistem",
        "two",
        "three",
        "unusnauta",
        "unusnautaer/ir",
        "duo",
        "tres",
        "mille",
    ],
];

/// Creates the given word into the database.
pub fn create_word(word: Word) -> Result<(), String> {
    match word.category {
        Category::Noun => match word.declension_id {
            Some(id @ 1..7) => {
                if !DECLENSIONS_WITH_KINDS[id - 1].contains(&word.kind.as_str()) {
                    return Err(format!("bad kind for declension '{id}'"));
                }
            }
            Some(val) => return Err(format!("the declension ID '{val}' is not valid for nouns")),
            None => {
                return Err(String::from(
                    "you have to provide the declension ID for this noun",
                ))
            }
        },
        Category::Adjective => match word.declension_id {
            Some(id @ (1 | 3)) => {
                if !ADJECTIVE_KINDS[id - 1].contains(&word.kind.as_str()) {
                    return Err(format!("bad kind for declension '{id}'"));
                }
            }
            Some(val) => {
                return Err(format!(
                    "the declension ID '{val}' is not valid for adjectives"
                ))
            }
            None => {
                return Err(String::from(
                    "you have to provide the declension ID for this adjective",
                ))
            }
        },
        // TODO
        _ => {
            return Err(format!(
                "you cannot create a word from the '{}' category",
                word.category
            ))
        }
    }

    let conn = get_connection()?;
    match conn.execute(
        "INSERT INTO words (enunciated, particle, language_id, declension_id, conjugation_id, kind, category, regular, locative, gender, suffix, flags, translation, updated_at, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'), datetime('now'))",
        params![word.enunciated, word.particle, word.language as usize,
    word.declension_id, word.conjugation_id, word.kind, word.category as usize,
    word.regular, word.locative, word.gender as usize, word.suffix,
    serde_json::to_string(&word.flags).unwrap(), serde_json::to_string(&word.translation).unwrap()]) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not create '{}': {}", word.enunciated, e)),
    }
}

pub fn update_word(word: Word) -> Result<(), String> {
    if word.id == 0 {
        return Err("invalid word to update; seems it has not been created before".to_string());
    }

    let conn = get_connection()?;

    match conn.execute(
        "UPDATE words \
         SET enunciated = ?2, particle = ?3, declension_id = ?4, conjugation_id = ?5, \
             kind = ?6, category = ?7, regular = ?8, locative = ?9, gender = ?10, \
             suffix = ?11, flags = ?12, translation = ?13, updated_at = datetime('now') \
         WHERE id = ?1",
        params![
            word.id,
            word.enunciated,
            word.particle,
            word.declension_id,
            word.conjugation_id,
            word.kind,
            word.category as usize,
            word.regular,
            word.locative,
            word.gender as usize,
            word.suffix,
            serde_json::to_string(&word.flags).unwrap(),
            serde_json::to_string(&word.translation).unwrap()
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", word.enunciated, e)),
    }
}

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

pub fn find_by(enunciated: &str) -> Result<Word, String> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, enunciated, particle, language_id, declension_id, conjugation_id, \
                    kind, category, regular, locative, gender, suffix, translation, \
                    succeeded, steps, flags \
             FROM words \
             WHERE enunciated = ?1",
        )
        .unwrap();
    let mut it = stmt.query([enunciated]).unwrap();

    match it.next() {
        Err(_) => Err("no words were found with this enunciate".to_string()),
        Ok(rows) => match rows {
            Some(row) => Ok(Word {
                id: row.get(0).unwrap(),
                enunciated: row.get(1).unwrap(),
                particle: row.get(2).unwrap(),
                language: row.get::<usize, usize>(3).unwrap().try_into()?,
                declension_id: row.get(4).unwrap(),
                conjugation_id: row.get(5).unwrap(),
                kind: row.get(6).unwrap(),
                category: row.get::<usize, usize>(7).unwrap().try_into()?,
                regular: row.get(8).unwrap(),
                locative: row.get(9).unwrap(),
                gender: row.get::<usize, usize>(10).unwrap().try_into()?,
                suffix: row.get(11).unwrap(),
                translation: serde_json::from_str(&row.get::<usize, String>(12).unwrap()).unwrap(),
                succeeded: row.get(13).unwrap(),
                steps: row.get(14).unwrap(),
                flags: serde_json::from_str(&row.get::<usize, String>(15).unwrap()).unwrap(),
            }),
            None => Err("no words were found with this enunciate".to_string()),
        },
    }
}

pub fn select_random_words(category: Category, number: usize) -> Result<Vec<Word>, String> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, enunciated, particle, language_id, declension_id, conjugation_id, \
                    kind, category, regular, locative, gender, suffix, translation, \
                    succeeded, steps \
             FROM words \
             WHERE category = ?1 AND translation != '{}' \
             ORDER BY succeeded ASC, updated_at DESC
             LIMIT ?2",
        )
        .unwrap();
    let mut it = stmt.query([category as usize, number]).unwrap();

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(Word {
            id: row.get(0).unwrap(),
            enunciated: row.get(1).unwrap(),
            particle: row.get(2).unwrap(),
            language: row.get::<usize, usize>(3).unwrap().try_into()?,
            declension_id: row.get(4).unwrap(),
            conjugation_id: row.get(5).unwrap(),
            kind: row.get(6).unwrap(),
            category: row.get::<usize, usize>(7).unwrap().try_into()?,
            regular: row.get(8).unwrap(),
            locative: row.get(9).unwrap(),
            gender: row.get::<usize, usize>(10).unwrap().try_into()?,
            suffix: row.get(11).unwrap(),
            translation: serde_json::from_str(&row.get::<usize, String>(12).unwrap()).unwrap(),
            succeeded: row.get(13).unwrap(),
            steps: row.get(14).unwrap(),
            flags: serde_json::from_str("{}").unwrap(),
        });
    }
    Ok(res)
}

pub fn update_success(word: &Word, success: usize, steps: usize) -> Result<(), String> {
    let conn = get_connection()?;

    match conn.execute(
        "UPDATE words \
         SET succeeded = ?1, steps = ?2, updated_at = datetime('now') \
         WHERE id = ?3",
        params![success, steps, word.id],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", word.enunciated, e)),
    }
}

pub fn delete_word(enunciated: &String) -> Result<(), String> {
    let conn = get_connection()?;

    match conn.execute(
        "DELETE FROM words WHERE enunciated = ?1",
        params![enunciated.as_str()],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not remove '{enunciated}': {e}")),
    }
}

fn get_connection() -> Result<rusqlite::Connection, String> {
    let path = get_config_path()?.join("database.sqlite3");
    match Connection::open(path) {
        Ok(handle) => Ok(handle),
        Err(_) => Err(
            "could not fetch the database. Ensure that you have called 'init' first".to_string(),
        ),
    }
}
