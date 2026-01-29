use crate::cfg::Language;
use crate::get_connection;
use rusqlite::params;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::Result;
use serde_json::Value;

/// A word as represented in the 'words' table of the database.
#[derive(Clone, Debug)]
pub struct Word {
    pub id: i32,
    pub enunciated: String,
    pub particle: String,
    pub language: Language,
    pub declension: Option<Declension>,
    pub conjugation: Option<Conjugation>,
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
        declension: Option<Declension>,
        conjugation: Option<Conjugation>,
        gender: Gender,
        kind: String,
    ) -> Word {
        Word {
            id: 0,
            enunciated: "".to_string(),
            particle,
            category,
            declension,
            conjugation,
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

/// Identifies the declension for a given word, and it allows to do SQL to/from
/// conversions.
#[derive(Clone, Debug)]
pub enum Declension {
    First = 1,
    Second,
    Third,
    Fourth,
    Fifth,
    Other,
}

impl ToSql for Declension {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.clone() as isize))
    }
}

impl FromSql for Declension {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let val = value.as_i64().unwrap_or(0);

        match val {
            1 => Ok(Declension::First),
            2 => Ok(Declension::Second),
            3 => Ok(Declension::Third),
            4 => Ok(Declension::Fourth),
            5 => Ok(Declension::Fifth),
            _ => Ok(Declension::Other),
        }
    }
}

impl std::fmt::Display for Declension {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Declension::First => write!(f, "1st (-ae)"),
            Declension::Second => write!(f, "2nd (-ī)"),
            Declension::Third => write!(f, "3rd (-is)"),
            Declension::Fourth => write!(f, "4th (-ūs)"),
            Declension::Fifth => write!(f, "5th (-eī/-ēī)"),
            Declension::Other => write!(f, "other"),
        }
    }
}

/// Identifies the conjugation for a given verb, and it allows to do SQL to/from
/// conversions.
#[derive(Clone, Debug)]
pub enum Conjugation {
    First = 1,
    Second,
    Third,
    ThirdIo,
    Fourth,

    // The 'Other' conjugation is a container for a bunch of verbs like 'sum',
    // 'volō', etc. That is, we expect the user to realize this is an irregular
    // verb, and then we rely on the 'kind' column to determine which kind of
    // irregular verb that is. This is better than enlarging a list of kinds of
    // irregular verbs as it was done in the past.
    Other,
}

impl ToSql for Conjugation {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.clone() as isize))
    }
}

impl FromSql for Conjugation {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let val = value.as_i64().unwrap_or(0);

        match val {
            1 => Ok(Conjugation::First),
            2 => Ok(Conjugation::Second),
            3 => Ok(Conjugation::Third),
            4 => Ok(Conjugation::ThirdIo),
            5 => Ok(Conjugation::Fourth),
            _ => Ok(Conjugation::Other),
        }
    }
}

impl std::fmt::Display for Conjugation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Conjugation::First => write!(f, "1st (ā stems)"),
            Conjugation::Second => write!(f, "2nd (ē stems)"),
            Conjugation::Third => write!(f, "3rd (ĕ stems)"),
            Conjugation::ThirdIo => write!(f, "3rd (-iō variants)"),
            Conjugation::Fourth => write!(f, "4th (ī stems)"),
            Conjugation::Other => write!(f, "other"),
        }
    }
}

impl Conjugation {
    /// Returns a String containing how a conjugation would be displayed by also
    /// considering the given 'kind' identifier. This way we can display 'other'
    /// conjugations in a more natural way.
    pub fn display_with_kind(&self, kind: &str) -> String {
        if !matches!(self, Conjugation::Other) {
            return format!("{}", self);
        }

        match kind {
            "sum" => "irregular; like 'sum, esse, fuī, futūrus'",
            "possum" => "irregular; like 'possum, posse, potuī'",
            "eo" => "irregular; like 'eō, īre, iī, itum'",
            "volo" => "irregular; like 'volō, velle, voluī'",
            "nolo" => "irregular; like 'nōlō, nōlle, nōluī'",
            "malo" => "irregular; like 'mālō, mālle, māluī'",
            "fero" => "irregular; like 'ferō, ferre, tulī, lātum'",
            "facio" => "3rd (-iō variants) and suppletive; like 'faciō, facere, fēcī, factum'",
            "do" => "1st; irregular short ă in most forms",
            "inquam" => "irregular, highly defective",
            "aio" => "3rd (-iō variants); highly defective",
            _ => "other",
        }
        .to_string()
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

// Needed for inquire's (Multi)Select.
impl std::fmt::Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Comparative => write!(f, "comparative form"),
            Self::Superlative => write!(f, "superlative form"),
            Self::Adverb => write!(f, "adverbial form"),
            Self::Alternative => write!(f, "alternative word"),
            Self::Gendered => write!(f, "alternative word because of gender"),
        }
    }
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

/// Add a row in `word_relations` so the words identified by `one_id` and
/// `other_id` are set to have the `kind` relationship.
pub fn add_word_relationship(one_id: i64, other_id: i64, kind: RelationKind) -> Result<(), String> {
    let conn = get_connection()?;

    match conn.execute(
        "INSERT INTO word_relations (source_id, destination_id, kind, updated_at, created_at) \
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
        params![one_id, other_id, kind as isize],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
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
    match word.declension {
        Some(Declension::First | Declension::Second) => format!("{part}ē"),
        Some(Declension::Third) => format!("{part}iter"),
        _ => "<unknown>".to_string(),
    }
}

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

/// Creates the given word into the database and returns its ID on success.
pub fn create_word(word: Word) -> Result<i64, String> {
    match word.category {
        Category::Noun | Category::Adjective => {
            if word.declension.is_none() {
                return Err(String::from(
                    "you have to provide the declension for this verb",
                ));
            }
        }
        Category::Verb => {
            if word.conjugation.is_none() {
                return Err(String::from(
                    "you have to provide the conjugation for this verb",
                ));
            }
        }
        Category::Adverb
        | Category::Preposition
        | Category::Conjunction
        | Category::Interjection
        | Category::Determiner => {
            if word.declension.is_some() || word.conjugation.is_some() {
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
            word.declension,
            word.conjugation,
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
        Ok(_) => Ok(conn.last_insert_rowid()),
        Err(e) => Err(format!("could not create '{}': {}", word.enunciated, e)),
    }
}

/// Update the word that matches the ID on `word` and set it to the new values
/// contained in the `word` object.
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
            word.declension,
            word.conjugation,
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

/// Select words based on the given `filter` for the enunciated column, which
/// can be further filtered out by providing a set of `tags`. The words selected
/// must then have any of the given tags provided by this vector, and it will be
/// ignored if the passed vector is empty.
pub fn select_enunciated(filter: Option<String>, tags: &[String]) -> Result<Vec<String>, String> {
    let conn = get_connection()?;

    let mut stmt;
    let mut it = match filter {
        Some(filter) => {
            stmt = if tags.is_empty() {
                conn
                .prepare(
                    "SELECT enunciated FROM words WHERE enunciated LIKE ('%' || ?1 || '%') ORDER BY enunciated",
                )
                    .unwrap()
            } else {
                conn.prepare(
                    format!(
                        "SELECT w.enunciated \
                         FROM words w \
                         JOIN tag_associations ta ON w.id = ta.word_id \
                         JOIN tags t ON t.id = ta.tag_id \
                         WHERE w.enunciated LIKE ('%' || ?1 || '%') AND t.name IN ({}) \
                         ORDER BY w.enunciated",
                        tags.iter()
                            .map(|t| format!("'{}'", t))
                            .collect::<Vec<_>>()
                            .join(", "),
                    )
                    .as_str(),
                )
                .unwrap()
            };
            stmt.query([filter.as_str()]).unwrap()
        }
        None => {
            stmt = if tags.is_empty() {
                conn.prepare("SELECT enunciated FROM words ORDER BY enunciated")
                    .unwrap()
            } else {
                conn.prepare(
                    format!(
                        "SELECT w.enunciated \
                         FROM words w \
                         JOIN tag_associations ta ON w.id = ta.word_id \
                         JOIN tags t ON t.id = ta.tag_id \
                         WHERE t.name IN ({}) \
                         ORDER BY w.enunciated",
                        tags.iter()
                            .map(|t| format!("'{}'", t))
                            .collect::<Vec<_>>()
                            .join(", "),
                    )
                    .as_str(),
                )
                .unwrap()
            };
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
            declension: row.get(4).unwrap(),
            conjugation: row.get(5).unwrap(),
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
                declension: row.get(4).unwrap(),
                conjugation: row.get(5).unwrap(),
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
fn flags_clause(flags: &[String]) -> String {
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
// have set one of the given boolean `flags`. You may also pass a `tags` vector
// which contains the name of the tags for which each word must have at least
// one match.
pub fn select_relevant_words(
    category: Category,
    flags: &[String],
    tags: &[String],
    number: isize,
) -> Result<Vec<Word>, String> {
    let conn = get_connection()?;
    let mut stmt = if tags.is_empty() {
        conn.prepare(
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
        .unwrap()
    } else {
        conn.prepare(
            format!(
                "SELECT w.id, w.enunciated, w.particle, w.language_id, w.declension_id, w.conjugation_id, \
                    w.kind, w.category, w.regular, w.locative, w.gender, w.suffix, w.translation, \
                    w.succeeded, w.steps, w.flags, w.weight \
                 FROM words w \
                 JOIN tag_associations ta ON w.id = ta.word_id \
                 JOIN tags t ON t.id = ta.tag_id \
                 WHERE w.category = ?1 AND t.name IN ({}) AND w.translation != '{{}}' {} \
                 ORDER BY w.weight DESC, w.succeeded ASC, w.updated_at DESC
                 LIMIT ?2",
                tags.iter().map(|t| format!("'{}'", t)).collect::<Vec<_>>().join(", "),
                flags_clause(flags)
            )
            .as_str(),
        )
        .unwrap()
    };
    let mut it = stmt.query([category as isize, number]).unwrap();

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(Word {
            id: row.get(0).unwrap(),
            enunciated: row.get(1).unwrap(),
            particle: row.get(2).unwrap(),
            language: row.get::<usize, isize>(3).unwrap().try_into()?,
            declension: row.get(4).unwrap(),
            conjugation: row.get(5).unwrap(),
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
/// as with functions like `select_relevant_words`; and the `tags` filtering
/// option.
pub fn select_words_except(
    excluded: &[Word],
    categories: &[Category],
    flags: &[String],
    tags: &[String],
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
    let mut stmt = if tags.is_empty() {
        conn.prepare(
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
        .unwrap()
    } else {
        conn.prepare(
            format!(
                "SELECT w.id, w.enunciated, w.particle, w.language_id, w.declension_id, w.conjugation_id, \
                    w.kind, w.category, w.regular, w.locative, w.gender, w.suffix, w.translation, \
                    w.succeeded, w.steps, w.flags, w.weight \
                 FROM words w \
                 JOIN tag_associations ta ON w.id = ta.word_id \
                 JOIN tags t ON t.id = ta.tag_id \
                 WHERE w.id NOT IN ({}) AND t.name IN ({}) AND w.category IN ({}) AND w.translation != '{{}}' {} \
                 ORDER BY w.weight DESC, w.succeeded ASC, w.updated_at DESC
                 LIMIT 5",
                placeholders,
                tags.iter().map(|t| format!("'{}'", t)).collect::<Vec<_>>().join(", "),
                cats,
                flags_clause(flags)
            )
            .as_str(),
        )
        .unwrap()
    };

    let mut it = stmt.query(rusqlite::params_from_iter(ids)).unwrap();
    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(Word {
            id: row.get(0).unwrap(),
            enunciated: row.get(1).unwrap(),
            particle: row.get(2).unwrap(),
            language: row.get::<usize, isize>(3).unwrap().try_into()?,
            declension: row.get(4).unwrap(),
            conjugation: row.get(5).unwrap(),
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

/// Delete the given word while also removing any relationship with other words
/// and tags.
pub fn delete_word(word: &Word) -> Result<(), String> {
    let conn = get_connection()?;

    // Remove the word itself.
    if let Err(e) = conn.execute(
        "DELETE FROM words \
         WHERE id = ?1",
        params![word.id],
    ) {
        return Err(format!("could not remove '{}': {e}", word.enunciated));
    }

    // Remove any relationships that mention this word.
    if let Err(e) = conn.execute(
        "DELETE FROM word_relations \
         WHERE source_id = ?1 OR destination_id = ?1",
        params![word.id],
    ) {
        return Err(format!(
            "could not remove relationships from '{}': {e}",
            word.enunciated
        ));
    }

    // Remove any tag relationships with this now defunct word.
    match conn.execute(
        "DELETE FROM tag_associations \
         WHERE word_id = ?1",
        params![word.id],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "count not detach words for '{}': {e}",
            word.enunciated
        )),
    }
}
