use crate::get_connection;
use crate::word::{Declension, Gender, Word};
use serde_json::Value;
use std::convert::TryFrom;

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
        match word.declension {
            Some(Declension::First | Declension::Second) => &"a".to_string(),
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
        let onlyplural = word.is_flag_set("onlyplural");

        // Fetch the number and account for defectives on number.
        let number_i: isize = row.get(1).unwrap();
        let number: usize = usize::try_from(number_i).expect("not expecting a negative number");
        if (number == 0 && onlyplural) || (number == 1 && word.is_flag_set("onlysingular")) {
            continue;
        }

        let case_i: isize = row.get(3).unwrap();
        let term: String = row.get(4).unwrap();

        // If this is the locative, on the plural, and 'onlyplural' was not
        // specified, then chances are that the locative in the plural doesn't
        // exist. That is because it only existed for defective nouns such as
        // 'Athēnīs'.
        if case_i == 6 && number == 1 && !onlyplural {
            continue;
        }

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
