use serde_json::Value;
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead, BufReader, Error};
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

impl TryFrom<isize> for Category {
    type Error = &'static str;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
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

impl Gender {
    /// Returns a string containing the abbreviation for this gender.
    pub fn abbrev(&self) -> &str {
        match self {
            Self::Masculine => "m.",
            Self::Feminine => "f.",
            Self::MasculineOrFeminine => "m./f.",
            Self::Neuter => "n.",
            Self::None => "(genderless)",
        }
    }
}

impl TryFrom<isize> for Gender {
    type Error = &'static str;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
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

/// Ensure that in the config path there is a fully initialized database.
pub fn init_database() -> Result<(), String> {
    let name = &std::env::var("MIHI_DATABASE").unwrap_or("database.sqlite3".to_string());
    let path = get_config_path()?.join(name);
    let conn = match Connection::open(path) {
        Ok(handle) => handle,
        Err(e) => return Err(format!("could not initialize the database: {e}")),
    };

    match migrate::init(conn) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("bad database schema file: {e}")),
    }
}

/// Defines in which way two words are related.
#[derive(Clone, Debug)]
pub enum RelationKind {
    /// The destination word is the comparative of the source (e.g. 'magnus,
    /// magna, magnum' -> has irregular comparative -> 'māior, māius').
    Comparative = 1,

    /// The destination word is the superlative of the source (e.g. 'magnus,
    /// magna, magnum' -> has irregular superlative -> 'maximus, maxima,
    /// maximum').
    Superlative,

    /// The destination word is the adverb of the other (e.g. 'magnus, magna,
    /// magnum' -> has an adverb -> 'magnē').
    Adverb,

    /// Two given words are the alternative of the other because of their root
    /// or because of some sort of historical contraction (e.g. 'nihil' <->
    /// 'nīl', or the root on 'versō' <-> 'vōrsō').
    Alternative,

    /// One is the gendered alternative of the other (e.g. 'victor' <->
    /// 'victrix').
    Gendered,
}

impl TryFrom<isize> for RelationKind {
    type Error = String;

    fn try_from(v: isize) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(RelationKind::Comparative),
            2 => Ok(RelationKind::Superlative),
            3 => Ok(RelationKind::Adverb),
            4 => Ok(RelationKind::Alternative),
            5 => Ok(RelationKind::Gendered),
            _ => Err(format!("unknown relation kind value '{}'", v)),
        }
    }
}

/// Join by enunciate the given words.
pub fn joint_related_words(related: &[Word]) -> String {
    related
        .iter()
        .map(|w| w.enunciated.clone())
        .collect::<Vec<String>>()
        .join("; ")
}

/// Returns a string with the enunciate of the comparative form of the given
/// `word`. This function assumes that it really does, or at least it's
/// contained in the `related` vector.
pub fn comparative(word: &Word, related: &[Word]) -> String {
    if !related.is_empty() {
        return joint_related_words(related);
    }
    if word.is_flag_set("compsup_prefix") {
        return format!("magis {}", word.singular_nominative());
    }

    let part = word.real_particle();
    format!("{part}ior, {part}ius")
}

/// Returns a string with the enunciate of the superlative form of the given
/// `word`. This function assumes that it really does, or at least it's
/// contained in the `related` vector.
pub fn superlative(word: &Word, related: &[Word]) -> String {
    if !related.is_empty() {
        return joint_related_words(related);
    }
    if word.is_flag_set("compsup_prefix") {
        return format!("maximē {}", word.singular_nominative());
    }

    let part = &word.particle;
    if word.is_flag_set("irregularsup") {
        return format!("{part}limus, {part}lima, {part}limum");
    } else if word.is_flag_set("contracted_root") {
        return format!("{part}rimus, {part}rima, {part}rimum");
    }
    format!("{part}issimus, {part}issima, {part}issimum")
}

/// Returns a string with the enunciate of the adverbial form of the given
/// `word`. This function assumes that it really does, or at least it's
/// contained in the `related` vector.
pub fn adverb(word: &Word, related: &[Word]) -> String {
    if !related.is_empty() {
        return joint_related_words(related);
    }

    let part = word.real_particle();
    match word.declension_id {
        Some(1 | 2) => format!("{part}ē"),
        Some(3) => format!("{part}iter"),
        _ => "<unknown>".to_string(),
    }
}

#[derive(Clone, Debug)]
pub struct Word {
    pub id: i32,
    pub enunciated: String,
    pub particle: String,
    pub language: Language,
    pub declension_id: Option<isize>,
    pub conjugation_id: Option<isize>,
    pub kind: String,
    pub category: Category,
    pub regular: bool,
    pub locative: bool,
    pub gender: Gender,
    pub suffix: Option<String>,
    pub translation: Value,
    pub flags: Value,
    pub succeeded: isize,
    pub steps: isize,
    pub weight: isize,
}

impl Word {
    pub fn from(
        particle: String,
        category: Category,
        declension_id: Option<isize>,
        conjugation_id: Option<isize>,
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
            weight: 5,
        }
    }

    pub fn inflection_id(&self) -> Option<isize> {
        if matches!(self.category, Category::Verb) {
            return Some(self.conjugation_id.unwrap());
        }
        self.declension_id
    }

    /// Returns whether the given flag is set to true on this word.
    pub fn is_flag_set(&self, flag: &str) -> bool {
        match self.flags.get(flag) {
            Some(value) => value.as_bool().unwrap_or_default(),
            None => false,
        }
    }

    /// Returns the nominative version of the enunciate.
    pub fn singular_nominative(&self) -> String {
        self.enunciated
            .split(',')
            .nth(0)
            .unwrap_or("")
            .trim()
            .to_string()
    }

    pub fn real_particle(&self) -> String {
        if self.is_flag_set("contracted_root") {
            return format!(
                "{}{}",
                &self.particle[0..(self.particle.len() - 2)],
                self.particle.chars().last().unwrap_or(' '),
            );
        }
        self.particle.clone()
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

/// List of boolean flags supported for words.
pub const BOOLEAN_FLAGS: &[&str] = &[
    "deponent",
    "onlysingular",
    "onlyplural",
    "contracted_root",
    "nonpositive",
    "compsup_prefix",
    "indeclinable",
    "irregularsup",
    "nopassive",
    "nosupine",
    "noperfect",
    "nogerundive",
    "impersonal",
    "impersonalpassive",
    "noimperative",
    "noinfinitive",
    "shortimperative",
    "onlythirdpassive",
    "enclitic",
    "notcomparable",
    "onlyperfect",
    "semideponent",
    "contracted_vocative",
];

/// Returns true if the given flag is supported by this application.
pub fn is_valid_word_flag(flag: &str) -> bool {
    BOOLEAN_FLAGS.contains(&flag)
}

/// Creates the given word into the database.
pub fn create_word(word: Word) -> Result<(), String> {
    match word.category {
        Category::Noun => match word.declension_id {
            Some(id @ 1..7) => {
                if !DECLENSIONS_WITH_KINDS[id as usize - 1].contains(&word.kind.as_str()) {
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
                if !ADJECTIVE_KINDS[id as usize - 1].contains(&word.kind.as_str()) {
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
        Category::Verb => match word.conjugation_id {
            Some(1..6) => {
                if word.kind.as_str() != "verb" {
                    return Err("bad kind for verb".to_string());
                }
            }
            Some(val) => return Err(format!("the conjugation ID '{val}' is not valid")),
            None => {
                return Err(String::from(
                    "you have to provide the conjugation ID for this verb",
                ))
            }
        },
        Category::Adverb
        | Category::Preposition
        | Category::Conjunction
        | Category::Interjection
        | Category::Determiner => {
            if word.declension_id.is_some() || word.conjugation_id.is_some() {
                return Err(format!("no inflection allowed for '{}'", word.category));
            }
        }
        Category::Unknown | Category::Pronoun => {
            return Err(format!(
                "you cannot create a word from the '{}' category",
                word.category
            ))
        }
    }

    let conn = get_connection()?;
    match conn.execute(
        "INSERT INTO words (enunciated, particle, language_id, declension_id, \
                            conjugation_id, kind, category, regular, locative, \
                            gender, suffix, flags, translation, weight, succeeded, \
                            updated_at, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, \
                 datetime('now'), datetime('now'))",
        params![
            word.enunciated.trim(),
            word.particle.trim(),
            word.language as isize,
            word.declension_id,
            word.conjugation_id,
            word.kind.trim(),
            word.category as isize,
            word.regular,
            word.locative,
            word.gender as isize,
            word.suffix,
            serde_json::to_string(&word.flags).unwrap(),
            serde_json::to_string(&word.translation).unwrap(),
            word.weight,
            0
        ],
    ) {
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
             suffix = ?11, flags = ?12, translation = ?13, weight = ?14, \
             updated_at = datetime('now') \
         WHERE id = ?1",
        params![
            word.id,
            word.enunciated,
            word.particle,
            word.declension_id,
            word.conjugation_id,
            word.kind,
            word.category as isize,
            word.regular,
            word.locative,
            word.gender as isize,
            word.suffix,
            serde_json::to_string(&word.flags).unwrap(),
            serde_json::to_string(&word.translation).unwrap(),
            word.weight
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", word.enunciated, e)),
    }
}

/// Update the `updated_at` timestamp for any word matching the given
/// `enunciated` string. In theory the given enunciated should identify only a
/// single word, but nothing forbids the caller from updating every word which
/// somehow matches the given string.
pub fn update_timestamp(enunciated: &str) -> Result<(), String> {
    let conn = get_connection()?;

    match conn.execute(
        "UPDATE words \
         SET updated_at = datetime('now') \
         WHERE enunciated = ?1",
        params![enunciated],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", enunciated, e)),
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

/// Returns all words that are related to the given `word` in one way or
/// another. The result is given as an array where each element is indexed by
/// RelationKind, and has a vector of words following that relationship.
pub fn select_related_words(word: &Word) -> Result<[Vec<Word>; 5], String> {
    let mut res = [vec![], vec![], vec![], vec![], vec![]];

    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
                "SELECT w.id, w.enunciated, w.particle, w.language_id, w.declension_id, w.conjugation_id, \
                    w.kind as wkind, w.category, w.regular, w.locative, w.gender, w.suffix, w.translation, \
                    w.succeeded, w.steps, w.flags, w.weight, r.kind as rkind \
                 FROM words w \
                 JOIN word_relations r ON w.id = r.destination_id
                 WHERE r.source_id = ?1",
        )
        .unwrap();
    let mut it = stmt.query([word.id]).unwrap();

    while let Some(row) = it.next().unwrap() {
        let relation: RelationKind = row.get::<usize, isize>(17).unwrap().try_into()?;

        res[relation as usize - 1].push(Word {
            id: row.get(0).unwrap(),
            enunciated: row.get(1).unwrap(),
            particle: row.get(2).unwrap(),
            language: row.get::<usize, isize>(3).unwrap().try_into()?,
            declension_id: row.get(4).unwrap(),
            conjugation_id: row.get(5).unwrap(),
            kind: row.get(6).unwrap(),
            category: row.get::<usize, isize>(7).unwrap().try_into()?,
            regular: row.get(8).unwrap(),
            locative: row.get(9).unwrap(),
            gender: row.get::<usize, isize>(10).unwrap().try_into()?,
            suffix: row.get(11).unwrap(),
            translation: serde_json::from_str(&row.get::<usize, String>(12).unwrap()).unwrap(),
            succeeded: row.get(13).unwrap(),
            steps: row.get(14).unwrap(),
            flags: serde_json::from_str(&row.get::<usize, String>(15).unwrap()).unwrap(),
            weight: row.get(16).unwrap(),
        });
    }

    Ok(res)
}

pub fn find_by(enunciated: &str) -> Result<Word, String> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, enunciated, particle, language_id, declension_id, conjugation_id, \
                    kind, category, regular, locative, gender, suffix, translation, \
                    succeeded, steps, flags, weight \
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
                language: row.get::<usize, isize>(3).unwrap().try_into()?,
                declension_id: row.get(4).unwrap(),
                conjugation_id: row.get(5).unwrap(),
                kind: row.get(6).unwrap(),
                category: row.get::<usize, isize>(7).unwrap().try_into()?,
                regular: row.get(8).unwrap(),
                locative: row.get(9).unwrap(),
                gender: row.get::<usize, isize>(10).unwrap().try_into()?,
                suffix: row.get(11).unwrap(),
                translation: serde_json::from_str(&row.get::<usize, String>(12).unwrap()).unwrap(),
                succeeded: row.get(13).unwrap(),
                steps: row.get(14).unwrap(),
                flags: serde_json::from_str(&row.get::<usize, String>(15).unwrap()).unwrap(),
                weight: row.get(16).unwrap(),
            }),
            None => Err("no words were found with this enunciate".to_string()),
        },
    }
}

// Builds up a chain of OR clauses that check whether either of the given
// `flags` are set for a row. If no flags are given, then an empty string is
// returned. Otherwise the string is prepended by an "AND" clause, meaning that
// it expects the caller to have other clauses before this one.
fn flags_clause(flags: &Vec<String>) -> String {
    if flags.is_empty() {
        return "".to_string();
    }

    let mut clauses: Vec<String> = vec![];
    for flag in flags {
        clauses.push(format!("json_extract(flags, '$.{flag}') = 1"));
    }

    "AND (".to_owned() + &clauses.join(" OR ") + ")"
}

// Select a maximum of `number` words which match a given word `category` and
// have set one of the given boolean `flags`.
pub fn select_relevant_words(
    category: Category,
    flags: &Vec<String>,
    number: isize,
) -> Result<Vec<Word>, String> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            format!(
                "SELECT id, enunciated, particle, language_id, declension_id, conjugation_id, \
                    kind, category, regular, locative, gender, suffix, translation, \
                    succeeded, steps, flags, weight \
                 FROM words \
                 WHERE category = ?1 AND translation != '{{}}' {} \
                 ORDER BY weight DESC, succeeded ASC, updated_at DESC
                 LIMIT ?2",
                flags_clause(flags)
            )
            .as_str(),
        )
        .unwrap();
    let mut it = stmt.query([category as isize, number]).unwrap();

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(Word {
            id: row.get(0).unwrap(),
            enunciated: row.get(1).unwrap(),
            particle: row.get(2).unwrap(),
            language: row.get::<usize, isize>(3).unwrap().try_into()?,
            declension_id: row.get(4).unwrap(),
            conjugation_id: row.get(5).unwrap(),
            kind: row.get(6).unwrap(),
            category: row.get::<usize, isize>(7).unwrap().try_into()?,
            regular: row.get(8).unwrap(),
            locative: row.get(9).unwrap(),
            gender: row.get::<usize, isize>(10).unwrap().try_into()?,
            suffix: row.get(11).unwrap(),
            translation: serde_json::from_str(&row.get::<usize, String>(12).unwrap()).unwrap(),
            succeeded: row.get(13).unwrap(),
            steps: row.get(14).unwrap(),
            flags: serde_json::from_str(&row.get::<usize, String>(15).unwrap()).unwrap(),
            weight: row.get(16).unwrap(),
        });
    }
    Ok(res)
}

/// Select a set of words except for the ones passed in the `excluded`
/// vector. You have to pass the categories to be selected via the `categories`
/// parameter, which cannot be empty. It also accepts a set of boolean `flags`
/// as with functions like `select_relevant_words`.
pub fn select_words_except(
    excluded: &[Word],
    categories: &[Category],
    flags: &Vec<String>,
) -> Result<Vec<Word>, String> {
    assert!(!categories.is_empty());

    let ids = excluded.iter().map(|w| w.id).collect::<Vec<i32>>();
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let cats = categories
        .iter()
        .map(|c| format!("{}", *c as isize))
        .collect::<Vec<_>>()
        .join(", ");

    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            format!(
                "SELECT id, enunciated, particle, language_id, declension_id, conjugation_id, \
                    kind, category, regular, locative, gender, suffix, translation, \
                    succeeded, steps, flags, weight \
                 FROM words \
                 WHERE id NOT IN ({}) AND category IN ({}) AND translation != '{{}}' {} \
                 ORDER BY weight DESC, succeeded ASC, updated_at DESC
                 LIMIT 5",
                placeholders,
                cats,
                flags_clause(flags)
            )
            .as_str(),
        )
        .unwrap();

    let mut it = stmt.query(rusqlite::params_from_iter(ids)).unwrap();
    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(Word {
            id: row.get(0).unwrap(),
            enunciated: row.get(1).unwrap(),
            particle: row.get(2).unwrap(),
            language: row.get::<usize, isize>(3).unwrap().try_into()?,
            declension_id: row.get(4).unwrap(),
            conjugation_id: row.get(5).unwrap(),
            kind: row.get(6).unwrap(),
            category: row.get::<usize, isize>(7).unwrap().try_into()?,
            regular: row.get(8).unwrap(),
            locative: row.get(9).unwrap(),
            gender: row.get::<usize, isize>(10).unwrap().try_into()?,
            suffix: row.get(11).unwrap(),
            translation: serde_json::from_str(&row.get::<usize, String>(12).unwrap()).unwrap(),
            succeeded: row.get(13).unwrap(),
            steps: row.get(14).unwrap(),
            flags: serde_json::from_str(&row.get::<usize, String>(15).unwrap()).unwrap(),
            weight: row.get(16).unwrap(),
        });
    }

    Ok(res)
}

pub fn update_success(word: &Word, success: isize, steps: isize) -> Result<(), String> {
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
    let name = &std::env::var("MIHI_DATABASE").unwrap_or("database.sqlite3".to_string());
    let path = get_config_path()?.join(name);

    match Connection::open(path) {
        Ok(handle) => Ok(handle),
        Err(_) => Err(
            "could not fetch the database. Ensure that you have called 'init' first".to_string(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ExerciseKind {
    #[default]
    Pensum = 0,
    Translation,
    Transformation,
    Numerical,
}

impl std::fmt::Display for ExerciseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Pensum => write!(f, "Pensum"),
            Self::Translation => write!(f, "Translation"),
            Self::Transformation => write!(f, "Transformation"),
            Self::Numerical => write!(f, "Numerical"),
        }
    }
}

impl TryFrom<isize> for ExerciseKind {
    type Error = &'static str;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Pensum),
            1 => Ok(Self::Translation),
            2 => Ok(Self::Transformation),
            3 => Ok(Self::Numerical),
            _ => Err("unknonwn exercise kind"),
        }
    }
}

impl TryFrom<&str> for ExerciseKind {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pensum" => Ok(Self::Pensum),
            "translation" => Ok(Self::Translation),
            "transformation" => Ok(Self::Transformation),
            "numerical" => Ok(Self::Numerical),
            _ => Err("unknonwn exercise kind. Available: pensum, translation, transformation and numerical"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Exercise {
    pub id: i32,
    pub title: String,
    pub enunciate: String,
    pub solution: String,
    pub lessons: String,
    pub kind: ExerciseKind,
}

/// Creates the given exercise into the database.
pub fn create_exercise(exercise: Exercise) -> Result<(), String> {
    let conn = get_connection()?;
    match conn.execute(
        "INSERT INTO exercises (title, enunciate, solution, lessons, kind, \
                                updated_at, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
        params![
            exercise.title,
            exercise.enunciate,
            exercise.solution,
            exercise.lessons,
            exercise.kind as isize,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not create '{}': {}", exercise.title, e)),
    }
}

pub fn select_by_title(filter: Option<String>) -> Result<Vec<String>, String> {
    let conn = get_connection()?;

    let mut stmt;
    let mut it = match filter {
        Some(filter) => {
            stmt = conn
                .prepare(
                    "SELECT title FROM exercises WHERE title LIKE ('%' || ?1 || '%') ORDER BY title",
                )
                .unwrap();
            stmt.query([filter.as_str()]).unwrap()
        }
        None => {
            stmt = conn
                .prepare("SELECT title FROM exercises ORDER BY title")
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

pub fn find_exercise_by_title(title: &str) -> Result<Exercise, String> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, enunciate, solution, lessons, kind  \
             FROM exercises \
             WHERE title = ?1",
        )
        .unwrap();
    let mut it = stmt.query([title]).unwrap();

    match it.next() {
        Err(_) => Err("no exercises were found with this title".to_string()),
        Ok(rows) => match rows {
            Some(row) => Ok(Exercise {
                id: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                enunciate: row.get(2).unwrap(),
                solution: row.get(3).unwrap(),
                lessons: row.get(4).unwrap(),
                kind: row.get::<usize, isize>(5).unwrap().try_into()?,
            }),
            None => Err("no exercises were found with this title".to_string()),
        },
    }
}

/// Updates the given exercise.
pub fn update_exercise(exercise: Exercise) -> Result<(), String> {
    if exercise.id == 0 {
        return Err("invalid exercise to update; seems it has not been created before".to_string());
    }

    let conn = get_connection()?;

    match conn.execute(
        "UPDATE exercises \
         SET title = ?2, enunciate = ?3, solution = ?4, lessons = ?5, kind = ?6, \
             updated_at = datetime('now') \
         WHERE id = ?1",
        params![
            exercise.id,
            exercise.title,
            exercise.enunciate,
            exercise.solution,
            exercise.lessons,
            exercise.kind as isize,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", exercise.title, e)),
    }
}

/// Updates the 'updated_at' column for an exercise.
pub fn touch_exercise(exercise: Exercise) -> Result<(), String> {
    if exercise.id == 0 {
        return Err("invalid exercise to update; seems it has not been created before".to_string());
    }

    let conn = get_connection()?;

    match conn.execute(
        "UPDATE exercises \
         SET updated_at = datetime('now') \
         WHERE id = ?1",
        params![exercise.id],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", exercise.title, e)),
    }
}

/// Delete an exercise from the database.
pub fn delete_exercise(title: &str) -> Result<(), String> {
    let conn = get_connection()?;

    match conn.execute("DELETE FROM exercises WHERE title = ?1", params![title]) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not remove '{title}': {e}")),
    }
}

// Get a list of exercises sorted by relevance. A maximum of `limit` exercises
// will be returned, and you can also specify to filter the returned exercises
// by `kind`.
pub fn select_relevant_exercises(
    kind: Option<ExerciseKind>,
    limit: isize,
) -> Result<Vec<Exercise>, String> {
    let conn = get_connection()?;

    let mut stmt;
    let mut it = match kind {
        Some(kind) => {
            stmt = conn
                .prepare(
                    "SELECT id, title, enunciate, solution, lessons, kind  \
                     FROM exercises \
                     WHERE kind = ?1 \
                     ORDER BY updated_at DESC \
                     LIMIT ?2",
                )
                .unwrap();
            stmt.query([kind as isize, limit]).unwrap()
        }
        None => {
            stmt = conn
                .prepare(
                    "SELECT id, title, enunciate, solution, lessons, kind  \
                     FROM exercises \
                     ORDER BY updated_at DESC \
                     LIMIT ?1",
                )
                .unwrap();
            stmt.query([limit]).unwrap()
        }
    };

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(Exercise {
            id: row.get(0).unwrap(),
            title: row.get(1).unwrap(),
            enunciate: row.get(2).unwrap(),
            solution: row.get(3).unwrap(),
            lessons: row.get(4).unwrap(),
            kind: row.get::<usize, isize>(5).unwrap().try_into()?,
        });
    }
    Ok(res)
}

#[derive(Debug, Default)]
pub struct DeclensionInfo {
    pub inflected: Vec<String>,
}

#[derive(Debug, Default)]
pub struct DeclensionTable {
    pub nominative: [DeclensionInfo; 2],
    pub vocative: [DeclensionInfo; 2],
    pub accusative: [DeclensionInfo; 2],
    pub genitive: [DeclensionInfo; 2],
    pub dative: [DeclensionInfo; 2],
    pub ablative: [DeclensionInfo; 2],
    pub locative: [DeclensionInfo; 2],
}

impl DeclensionTable {
    pub fn consume_blob(
        &mut self,
        case: usize,
        blob: &Value,
        word: &Word,
        gender: usize,
        add: bool,
    ) {
        if let Some(singular) = blob.get("singular") {
            let values = singular.as_array().unwrap();
            for v in values {
                let s = v.as_str().unwrap();
                if add {
                    self.add(word, case, 0, gender, s);
                } else {
                    self.set(word, case, 0, gender, s);
                }
            }
        }

        if let Some(plural) = blob.get("plural") {
            let values = plural.as_array().unwrap();
            for v in values {
                let s = v.as_str().unwrap();
                if add {
                    self.add(word, case, 1, gender, s);
                } else {
                    self.set(word, case, 1, gender, s);
                }
            }
        }
    }

    pub fn set(&mut self, word: &Word, case: usize, number: usize, gender: usize, term: &str) {
        match case {
            0 => {
                self.nominative[number].inflected = inflect_from(word, case, number, gender, term);
            }
            1 => {
                self.vocative[number].inflected = inflect_from(word, case, number, gender, term);
            }
            2 => {
                self.accusative[number].inflected = inflect_from(word, case, number, gender, term);
            }
            3 => {
                self.genitive[number].inflected = inflect_from(word, case, number, gender, term);
            }
            4 => {
                self.dative[number].inflected = inflect_from(word, case, number, gender, term);
            }
            5 => {
                self.ablative[number].inflected = inflect_from(word, case, number, gender, term);
            }
            6 => {
                self.locative[number].inflected = inflect_from(word, case, number, gender, term);
            }
            _ => {}
        }
    }

    pub fn add(&mut self, word: &Word, case: usize, number: usize, gender: usize, term: &str) {
        match case {
            0 => {
                self.nominative[number]
                    .inflected
                    .append(&mut inflect_from(word, case, number, gender, term));
            }
            1 => {
                self.vocative[number]
                    .inflected
                    .append(&mut inflect_from(word, case, number, gender, term));
            }
            2 => {
                self.accusative[number]
                    .inflected
                    .append(&mut inflect_from(word, case, number, gender, term));
            }
            3 => {
                self.genitive[number]
                    .inflected
                    .append(&mut inflect_from(word, case, number, gender, term));
            }
            4 => {
                self.dative[number]
                    .inflected
                    .append(&mut inflect_from(word, case, number, gender, term));
            }
            5 => {
                self.ablative[number]
                    .inflected
                    .append(&mut inflect_from(word, case, number, gender, term));
            }
            6 => {
                self.locative[number]
                    .inflected
                    .append(&mut inflect_from(word, case, number, gender, term));
            }
            _ => {}
        }
    }
}

fn contract_root(word: &Word, case: usize, number: usize, gender: usize) -> bool {
    // First off, check out that this is a word explicitely marked as to be
    // contracted by either the flag or the kind.
    if !word.is_flag_set("contracted_root") {
        return false;
    }
    if word.kind != "er/ir" && word.kind != "unusnautaer/ir" {
        return false;
    }

    // All plurals have to be contracted.
    if number == 1 {
        return true;
    }

    // Nominative/vocative singular are only contracted for feminine nouns. The
    // accusative is only not contracted on neuter words.
    match case {
        0 | 1 => gender == Gender::Feminine as usize,
        2 => gender != Gender::Neuter as usize,
        _ => true,
    }
}

fn should_use_first_root(word: &Word, case: usize, number: usize, gender: usize) -> bool {
    // All plurals always follow `word.particle`.
    if number == 1 {
        return false;
    }

    match case {
        0 | 1 => {
            word.kind == "is"
                || word.kind == "istem"
                || word.kind == "pureistem"
                || word.kind == "one"
                || word.kind == "onenonistem"
        }
        2 => {
            // Only neuter words should consider this on the accusative.
            if gender != 3 {
                return false;
            }
            word.kind == "is"
                || word.kind == "istem"
                || word.kind == "pureistem"
                || word.kind == "one"
                || word.kind == "onenonistem"
        }
        _ => false,
    }
}

fn inflect_from(word: &Word, case: usize, number: usize, gender: usize, term: &str) -> Vec<String> {
    let mut inflections = vec![];

    if !word.regular {
        inflections.push(term.to_owned());
    } else if contract_root(word, case, number, gender) {
        inflections.push(word.particle[0..word.particle.len() - 2].to_string() + "r" + term);
    } else if should_use_first_root(word, case, number, gender) {
        let parts: Vec<&str> = word.enunciated.split(',').collect();
        inflections.push(parts.first().unwrap().to_string() + term);
    } else if word.kind == "ius" && number == 0 {
        // Words of this kind are a bit troublesome on the singular, let's
        // handle them now.
        if case == 1 && word.is_flag_set("contracted_vocative") {
            inflections.push(word.particle[0..word.particle.len() - 1].to_string() + term);
        } else {
            if case == 3 {
                inflections.push(word.particle[0..word.particle.len() - 1].to_string() + term);
            }
            inflections.push(word.particle.to_string() + term);
        }
    } else {
        inflections.push(word.particle.clone() + term);
    }

    inflections
}

fn case_str_to_i(key: &str) -> Result<usize, String> {
    match key {
        "nominative" => Ok(0),
        "vocative" => Ok(1),
        "accusative" => Ok(2),
        "genitive" => Ok(3),
        "dative" => Ok(4),
        "ablative" => Ok(5),
        "locative" => Ok(6),
        _ => Err(format!("bad key '{}' for a case", key)),
    }
}

/// Returns a string which describes the enunciate of the given `word` as
/// inflected considering the singular/plural declension `row`.
pub fn get_inflected_from(word: &Word, row: &[DeclensionInfo; 2]) -> String {
    if word.is_flag_set("onlysingular") {
        row[0].inflected.join("/")
    } else if word.is_flag_set("onlyplural") {
        row[1].inflected.join("/")
    } else {
        format!(
            "{}, {}",
            row[0].inflected.join("/"),
            row[1].inflected.join("/")
        )
    }
}

/// Returns the declension table of the given `word` by assuming it's a noun.
pub fn get_noun_table(word: &Word) -> Result<DeclensionTable, String> {
    let gender = match word.gender {
        Gender::MasculineOrFeminine => Gender::Masculine as usize,
        _ => word.gender as usize,
    };
    group_declension_inflections(word, &word.kind, gender)
}

/// Returns the declension tables for each gender of the given `word` by
/// assuming it's an adjective.
pub fn get_adjective_table(word: &Word) -> Result<[DeclensionTable; 3], String> {
    // Unless the word is a special "unus nauta" variant, force 1/2 declension
    // adjectives in the feminine to grab the "a" kind.
    let kind_f = if word.kind.as_str() == "unusnauta" {
        &word.kind
    } else {
        match word.declension_id {
            Some(1 | 2) => &"a".to_string(),
            _ => &word.kind,
        }
    };

    let kind_n = if word.kind == "us" {
        &"um".to_owned()
    } else {
        &word.kind
    };

    Ok([
        group_declension_inflections(word, &word.kind, Gender::Masculine as usize)?,
        group_declension_inflections(word, kind_f, Gender::Feminine as usize)?,
        group_declension_inflections(word, kind_n, Gender::Neuter as usize)?,
    ])
}

/// Returns the declension table for the given `word` by using the given `kind`
/// and `gender`.
pub fn group_declension_inflections(
    word: &Word,
    kind: &String,
    gender: usize,
) -> Result<DeclensionTable, String> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, number, gender, \"case\", value, declension_id, \
                    kind, tense, mood, voice, person, conjugation_id \
             FROM forms \
             WHERE kind = ?1 AND gender = ?2
             ORDER BY id",
        )
        .unwrap();
    let mut it = stmt.query([kind, &gender.to_string()]).unwrap();

    let mut table = DeclensionTable::default();

    while let Some(row) = it.next().unwrap() {
        // Fetch the number and account for defectives on number.
        let number_i: isize = row.get(1).unwrap();
        let number: usize = usize::try_from(number_i).expect("not expecting a negative number");
        if (number == 0 && word.is_flag_set("onlyplural"))
            || (number == 1 && word.is_flag_set("onlysingular"))
        {
            continue;
        }

        let case_i: isize = row.get(3).unwrap();
        let term: String = row.get(4).unwrap();

        table.add(
            word,
            usize::try_from(case_i).expect("not expecting a negative number"),
            number,
            gender,
            &term,
        );
    }

    if let Some(sets) = word.flags.get("sets") {
        let object = sets.as_object().unwrap();

        for (case_gender, blob) in object.iter() {
            let case_gender_str = case_gender.as_str();
            match case_gender_str {
                "masculine" | "feminine" | "neuter" => {
                    if (gender == 0 && case_gender_str == "masculine")
                        || (gender == 1 && case_gender_str == "feminine")
                        || (gender == 2 && case_gender_str == "neuter")
                    {
                        let inner = blob.as_object().unwrap();
                        for (case, blob) in inner.iter() {
                            let case_i = case_str_to_i(case)?;
                            table.consume_blob(case_i, blob, word, gender, false);
                        }
                    }
                }
                _ => {
                    let case_i = case_str_to_i(case_gender)?;
                    table.consume_blob(case_i, blob, word, gender, false);
                }
            }
        }
    }

    if let Some(adds) = word.flags.get("adds") {
        let object = adds.as_object().unwrap();

        for (case_gender, blob) in object.iter() {
            let case_gender_str = case_gender.as_str();
            match case_gender_str {
                "masculine" | "feminine" | "neuter" => {
                    if (gender == 0 && case_gender_str == "masculine")
                        || (gender == 1 && case_gender_str == "feminine")
                        || (gender == 2 && case_gender_str == "neuter")
                    {
                        let inner = blob.as_object().unwrap();
                        for (case, blob) in inner.iter() {
                            let case_i = case_str_to_i(case)?;
                            table.consume_blob(case_i, blob, word, gender, true);
                        }
                    }
                }
                _ => {
                    let case_i = case_str_to_i(case_gender)?;
                    table.consume_blob(case_i, blob, word, gender, true);
                }
            }
        }
    }

    Ok(table)
}
