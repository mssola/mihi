extern crate rand;
use inquire::{Confirm, Editor, Text};
use mihi::cfg::configuration;
use mihi::exercise::{select_relevant_exercises, touch_exercise, Exercise, ExerciseKind};
use mihi::inflection::{get_adjective_table, get_inflected_from, get_noun_table, DeclensionTable};
use mihi::tag::{select_tag_names, update_success};
use mihi::word::{
    adverb, comparative, is_valid_word_flag, joint_related_words, select_related_words,
    select_relevant_words, select_words_except, superlative, Category, RelationKind, Word,
    BOOLEAN_FLAGS,
};
use rand::prelude::*;
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

use crate::locale::{current_locale, Locale};

// Maximum number of times a word has to be run in order to increase the number
// of successful runs.
const MAX_STEPS: usize = 5;

fn help(msg: Option<&str>) {
    if let Some(msg) = msg {
        println!("{}.\n", msg);
    }

    println!("mihi run: Run exercises. Default command if none was given.\n");
    println!("usage: mihi practice [OPTIONS]\n");

    println!("Options:");
    println!("   -c, --category <CATEGORY>\tOnly ask for words on the given <CATEGORY>.");
    println!("   -e, --exercises\t\tOnly practice with exercises.");
    println!("   -f, --flag\t\t\tFilter words by a boolean flag. Multiple flags can be provided.");
    println!("   -h, --help\t\t\tPrint this message.");
    println!("   -i, --inflection\t\tOnly practice word inflections (completing enunciates, declensions and conjugations.");
    println!("   -k, --kind <KIND>\t\tOnly ask for exercises for the given <KIND>.");
    println!("   -t, --tag <NAME>\t\tFilter words which match the given tag NAME. Multiple tags can be provided to match words with any of the tags provided.");
}

// Run the quiz for all the given `words` while expecting answers to be
// delivered in the given `locale`.
fn run_words(words: &Vec<Word>, locale: &Locale) -> bool {
    for word in words {
        // If the translation cannot be found, skip this word.
        let Some(translation) = word.translation.get(locale.to_code()) else {
            continue;
        };

        println!("Word: {}", word.enunciated);

        let Ok(raw) = Text::new(format!("Translation ({locale}):").as_str()).prompt() else {
            return false;
        };
        let answer = raw.trim();

        let tr = translation.as_str().unwrap_or("");
        let found = !answer.is_empty() && tr.split(',').any(|tr| tr.trim().contains(answer));

        if found {
            if word.steps as usize == MAX_STEPS - 1 {
                let _ = update_success(word, word.succeeded + 1, 0);
            } else {
                let _ = update_success(word, word.succeeded, word.steps + 1);
            }
            println!("\x1b[92m✓ {tr}\x1b[0m");
        } else {
            if word.succeeded > 0 {
                let _ = update_success(word, word.succeeded - 1, 0);
            }
            println!("\x1b[91m❌{tr}\x1b[0m");
        }
    }

    true
}

fn fill_out_enunciated(word: &Word) -> String {
    match word.category {
        Category::Noun | Category::Adjective | Category::Pronoun => {
            // For nouns and adjectives this should be as simple as just showing
            // the first part of the enunciate, as the "hard" part is to figure
            // out the second part.
            let en = word.enunciated.split(',').next().unwrap_or("");
            en.to_string()
        }
        Category::Verb => {
            // Split the enunciate and pick at random the index of the one to be
            // shown.
            let en: Vec<&str> = word.enunciated.split(',').map(|s| s.trim()).collect();
            let mut rng = rand::rng();
            let selection = rng.random_range(0..en.len());

            // And now reconstruct the string by replacing all parts except
            // 'selection' with '___'.
            en.iter()
                .enumerate()
                .map(|(i, part)| {
                    if i == selection {
                        part.to_string()
                    } else {
                        "___".to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(", ")
        }
        cat => panic!(
            "trying to fill out an enunciated from a non-supported category '{}'",
            cat
        ),
    }
}

// Returns true if both strings are either more or less the same, or the user
// considers it so.
fn same_answer(given: &String, expected: &String) -> bool {
    // If it's literally the same string, then return true.
    if given == expected {
        return true;
    }

    // If it's the same string but just with differences in the white spacing,
    // return true as well.
    let trimmed_given: String = given.chars().filter(|c| !c.is_whitespace()).collect();
    let trimmed_expected: String = expected.chars().filter(|c| !c.is_whitespace()).collect();
    if trimmed_given == trimmed_expected {
        return true;
    }

    // It's something else, then let the user to decide.
    accepted_diff(given, expected)
}

fn ask_for_table(word: &Word, table: &DeclensionTable, id: Option<&str>) -> bool {
    let added = match id {
        Some(s) => format!(" ({}) ", s),
        None => " ".to_string(),
    };
    let mut initial = format!("== {}{}==\n\n", word.enunciated, added);
    let mut expected = format!("== {}{}==\n\n", word.enunciated, added);

    for idx in configuration().case_order.to_usizes() {
        match idx {
            0 => {
                initial.push_str("Nominative: \n");
                expected.push_str(
                    format!(
                        "Nominative: {}\n",
                        get_inflected_from(word, &table.nominative)
                    )
                    .as_str(),
                );
            }
            1 => {
                initial.push_str("Vocative: \n");
                expected.push_str(
                    format!("Vocative: {}\n", get_inflected_from(word, &table.vocative)).as_str(),
                );
            }
            2 => {
                initial.push_str("Accusative: \n");
                expected.push_str(
                    format!(
                        "Accusative: {}\n",
                        get_inflected_from(word, &table.accusative)
                    )
                    .as_str(),
                );
            }
            3 => {
                initial.push_str("Genitive: \n");
                expected.push_str(
                    format!("Genitive: {}\n", get_inflected_from(word, &table.genitive)).as_str(),
                );
            }
            4 => {
                initial.push_str("Dative: \n");
                expected.push_str(
                    format!("Dative: {}\n", get_inflected_from(word, &table.dative)).as_str(),
                );
            }
            5 => {
                initial.push_str("Ablative: \n");
                expected.push_str(
                    format!("Ablative: {}\n", get_inflected_from(word, &table.ablative)).as_str(),
                );
            }
            6 => {
                if word.locative {
                    initial.push_str("Locative: \n");
                    expected.push_str(
                        format!("Locative: {}\n", get_inflected_from(word, &table.locative))
                            .as_str(),
                    );
                }
            }
            _ => {}
        }
    }

    // Inflection time!
    let Ok(solution) = Editor::new("Open the editor to inflect:")
        .with_predefined_text(initial.as_str())
        .with_file_extension(".md")
        .prompt()
    else {
        return false;
    };

    same_answer(&solution, &expected)
}

// Ask for alternative forms (gendered or otherwise) about a given word.
fn ask_for_alternatives(related: &[Vec<Word>; 5]) -> bool {
    let alternatives = &related[RelationKind::Alternative as usize - 1];
    if !alternatives.is_empty() {
        let Ok(raw) =
            Text::new("Do you know of any alternative (not asking about a gendered one)?").prompt()
        else {
            return false;
        };
        let expected = joint_related_words(alternatives);
        if !same_answer(&raw, &expected) {
            return false;
        }
    }

    let gendered = &related[RelationKind::Gendered as usize - 1];
    if !gendered.is_empty() {
        let Ok(raw) = Text::new("Do you know of the same word but on the other gender?").prompt()
        else {
            return false;
        };
        let expected = joint_related_words(gendered);
        if !same_answer(&raw, &expected) {
            return false;
        }
    }

    true
}

// Ask for other forms for the given word (i.e. comparative, superlative,
// adverbial).
//
// NOTE: this word _has_ to be an adjective.
fn ask_for_others(word: &Word, related: &[Vec<Word>; 5]) -> bool {
    assert!(matches!(word.category, Category::Adjective));

    let comparative = comparative(word, &related[RelationKind::Comparative as usize - 1]);
    let Ok(raw) = Text::new("Comparative:").prompt() else {
        return false;
    };
    if !same_answer(&raw, &comparative) {
        return false;
    }

    let superlative = superlative(word, &related[RelationKind::Superlative as usize - 1]);
    let Ok(raw) = Text::new("Superlative:").prompt() else {
        return false;
    };
    if !same_answer(&raw, &superlative) {
        return false;
    }

    let adverbial = adverb(word, &related[RelationKind::Adverb as usize - 1]);
    let Ok(raw) = Text::new("Adverb:").prompt() else {
        return false;
    };
    if !same_answer(&raw, &adverbial) {
        return false;
    }

    true
}

fn good_noun_inflection(word: &Word) -> bool {
    if let Ok(table) = get_noun_table(word) {
        if !ask_for_table(word, &table, None) {
            return false;
        }
        if let Ok(related) = select_related_words(word) {
            return ask_for_alternatives(&related);
        }
    }
    true
}

fn good_adjective_inflection(word: &Word) -> bool {
    if let Ok(tables) = get_adjective_table(word) {
        // Pick which gender from the adjective table to ask.
        let mut rng = rand::rng();
        let gender = rng.random_range(0..=2);
        let suffix = match gender {
            1 => Some("in the feminine"),
            2 => Some("in the neuter"),
            _ => Some("in the masculine"),
        };

        if !ask_for_table(word, &tables[gender], suffix) {
            return false;
        }
        if let Ok(related) = select_related_words(word) {
            if !ask_for_others(word, &related) {
                return false;
            }
            return ask_for_alternatives(&related);
        }
    }
    true
}

fn good_inflection(word: &Word) -> bool {
    match word.category {
        Category::Noun => good_noun_inflection(word),
        Category::Adjective => good_adjective_inflection(word),
        cat => panic!("error: practice: trying to inflect {cat}"),
    }
}

fn run_inflect_words(words: &Vec<Word>, locale: &Locale) -> bool {
    for word in words {
        // If the translation cannot be found, skip this word.
        let Some(translation) = word.translation.get(locale.to_code()) else {
            continue;
        };

        // Enunciate.
        println!("Fill out this {}:", word.category);
        println!("Translation: {}.", translation);

        // Complete the enunciate.
        let Ok(raw) = Text::new("Enunciated:")
            .with_initial_value(&fill_out_enunciated(word))
            .prompt()
        else {
            return false;
        };
        let answer = raw.trim();

        // Check the answer and update the success rate on the database if
        // needed.
        if same_answer(&answer.to_string(), &word.enunciated) {
            if word.steps as usize == MAX_STEPS - 1 {
                let _ = update_success(word, word.succeeded + 1, 0);
            } else {
                let _ = update_success(word, word.succeeded, word.steps + 1);
            }
            println!("\x1b[92m✓\x1b[0m\n");
        } else {
            if word.succeeded > 0 {
                let _ = update_success(word, word.succeeded - 1, 0);
            }
            println!("\x1b[91m❌\x1b[0m\n");
        }

        // We only ask to inflect nouns, adjectives and pronouns.
        if matches!(word.category, Category::Noun | Category::Adjective) {
            // Now ask for inflecting the given word in various ways depending on
            // the word category.
            if good_inflection(word) {
                if word.steps as usize == MAX_STEPS - 1 {
                    let _ = update_success(word, word.succeeded + 1, 0);
                } else {
                    let _ = update_success(word, word.succeeded, word.steps + 1);
                }
                println!("\x1b[92m✓\x1b[0m\n");
            } else {
                if word.succeeded > 0 {
                    let _ = update_success(word, word.succeeded - 1, 0);
                }
                println!("\x1b[91m❌\x1b[0m\n");
            }
        }
    }

    true
}

// Returns a vector of words which contain a randomized set of words from
// different categories.
fn select_general_words(flags: &[String], tags: &[String]) -> Result<Vec<Word>, String> {
    let mut res = select_relevant_words(Category::Noun, flags, tags, 4)?;
    res.append(&mut select_relevant_words(
        Category::Adjective,
        flags,
        tags,
        2,
    )?);
    res.append(&mut select_relevant_words(Category::Verb, flags, tags, 4)?);
    res.append(&mut select_relevant_words(
        Category::Pronoun,
        flags,
        tags,
        1,
    )?);
    res.append(&mut select_relevant_words(
        Category::Adverb,
        flags,
        tags,
        2,
    )?);
    res.append(&mut select_relevant_words(
        Category::Preposition,
        flags,
        tags,
        1,
    )?);
    res.append(&mut select_relevant_words(
        Category::Conjunction,
        flags,
        tags,
        1,
    )?);
    Ok(res)
}

// Assuming that the `given` string is the answer for an exercise enunciate,
// remove the enunciate proper (enveloped via '---' comments) and return only
// what the user typed in.
fn remove_exercise_enunciate(given: String) -> String {
    let mut res = vec![];
    let mut found = false;

    for line in given.lines() {
        let trimmed = line.trim();

        if found {
            res.push(line);
        }
        if trimmed.starts_with("---!") {
            found = true;
        }
    }

    res.join("\n").to_string()
}

// Returns true if the given `bin` exists on the PATH, false otherwise.
fn is_executable(bin: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{p}/{bin}");
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

// Returns the string for the name of the command that should be used to show
// diffs.
fn diff_tool() -> Option<&'static str> {
    ["difft", "vimdiff", "diff"]
        .into_iter()
        .find(|&cmd| is_executable(cmd))
}

// Perform a diff with the `given` and the `expected` answers for an exercise
// and interactively ask the user if things are ok. Returns a boolean depending
// on the user's answer to that final question, or false if something went
// wrong.
fn accepted_diff(given: &String, expected: &String) -> bool {
    // If a diff tool could be fetched, then write into temporary files and call
    // the diff tool against both temporary files; otherwise just print things
    // out into the stdout.
    match diff_tool() {
        Some(cmd) => {
            let Ok(mut given_file) = NamedTempFile::new() else {
                return false;
            };
            if writeln!(given_file, "{given}").is_err() {
                return false;
            }

            let Ok(mut expected_file) = NamedTempFile::new() else {
                return false;
            };
            if writeln!(expected_file, "{expected}").is_err() {
                return false;
            }

            let mut cmd = Command::new(cmd);
            cmd.arg(given_file.path()).arg(expected_file.path());
            cmd.status().expect("process failed to execute");
            println!();
        }
        None => {
            println!("---Given:\n{given}\n---Expected:\n{expected}");
        }
    }

    Confirm::new("Do you think that you did well?")
        .with_default(false)
        .prompt()
        .unwrap_or(false)
}

// Run the quiz for all the given `exercises`.
fn run_exercises(exercises: Vec<Exercise>) -> bool {
    if exercises.is_empty() {
        println!("practice: no exercises!");
        return true;
    }

    for exercise in exercises {
        let Ok(solution) = Editor::new(
            format!("Exercise '{}' (kind: {}):", exercise.title, exercise.kind).as_str(),
        )
        .with_predefined_text(
            format!(
                "---Enunciate: {}\n{}\n---!",
                exercise.title, exercise.enunciate
            )
            .as_str(),
        )
        .with_file_extension(".md")
        .prompt() else {
            return false;
        };

        let mut solution = remove_exercise_enunciate(solution);
        if solution.is_empty() {
            solution = String::from("<no solution given>");
        }
        println!(
            "Enunciate for '{}':\n\n{}\n\nGiven:\n",
            exercise.title, exercise.enunciate
        );

        // If the exercise is seen as correct by the user, then "touch"
        // (i.e. refresh the 'updated_at' date). This way, next time we select
        // exercises to show the user, we can prevent this one showing up first.
        if accepted_diff(&solution, &exercise.solution) {
            let _ = touch_exercise(exercise);
        }
    }

    true
}

pub fn run(args: Vec<String>) {
    let mut it = args.into_iter();
    let mut category = None;
    let mut kind: Option<ExerciseKind> = None;
    let mut exercises_only = false;
    let mut inflection_only = false;
    let mut endless = false;
    let mut flags: Vec<String> = vec![];
    let mut tags: Vec<String> = vec![];

    while let Some(first) = it.next() {
        match first.as_str() {
            "-h" | "--help" => {
                help(None);
                std::process::exit(0);
            }
            "-c" | "--category" => {
                if category.is_some() {
                    help(Some(
                        "error: practice: you cannot provide multiple categories",
                    ));
                    std::process::exit(1);
                }
                match it.next() {
                    Some(cat) => {
                        category = match cat.trim().to_lowercase().as_str() {
                            "noun" => Some(Category::Noun),
                            "adjective" => Some(Category::Adjective),
                            "verb" => Some(Category::Verb),
                            "pronoun" => Some(Category::Pronoun),
                            "adverb" => Some(Category::Adverb),
                            "preposition" => Some(Category::Preposition),
                            "conjunction" => Some(Category::Conjunction),
                            "determiner" => Some(Category::Determiner),
                            _ => return help(Some("error: practice: category not allowed")),
                        };
                    }
                    None => {
                        help(Some("error: practice: you have to provide a category"));
                        std::process::exit(1);
                    }
                }
            }
            "-e" | "--exercises" => {
                exercises_only = true;
            }
            "-i" | "--inflection" => {
                inflection_only = true;
            }
            "--endless" => {
                endless = true;
            }
            "-f" | "--flag" => match it.next() {
                Some(flag) => {
                    if is_valid_word_flag(flag.as_str()) {
                        if flags.iter().any(|s| s.as_str() == flag) {
                            println!(
                                "warning: practice: flag '{flag}' was provided multiple times"
                            );
                        } else {
                            flags.push(flag);
                        }
                    } else {
                        let supported = BOOLEAN_FLAGS.join(", ");
                        help(Some(
                            format!(
                                "error: practice: unknown flag value '{flag}'. You have to pick between: {supported}"
                            )
                                .as_str(),
                        ));
                        std::process::exit(1);
                    }
                }
                None => {
                    help(Some(
                        "error: practice: you have to provide a value for the flag",
                    ));
                    std::process::exit(1);
                }
            },
            "-k" | "--kind" => {
                if kind.is_some() {
                    help(Some(
                        "error: practice: you cannot provide multiple exercise kinds",
                    ));
                    std::process::exit(1);
                }
                match it.next() {
                    Some(k) => {
                        kind = match k.trim().to_lowercase().as_str().try_into() {
                            Ok(k) => Some(k),
                            Err(e) => return help(Some(format!("error: practice: {e}").as_str())),
                        };
                    }
                    None => {
                        help(Some("error: practice: you have to provide a category"));
                        std::process::exit(1);
                    }
                }
            }
            "-t" | "--tag" => match it.next() {
                Some(t) => {
                    let name = t.trim().to_string();
                    if let Ok(results) = select_tag_names(&Some(name.clone())) {
                        if results.is_empty() {
                            println!("warning: practice: the tag '{}' does not exist.", name);
                        } else {
                            tags.push(name)
                        }
                    }
                }
                None => {
                    help(Some("error: practice: you have to provide a tag name"));
                    std::process::exit(1);
                }
            },
            _ => {
                help(Some(
                    format!("error: practice: unknown flag or command '{first}'").as_str(),
                ));
                std::process::exit(1);
            }
        }
    }

    let locale = current_locale();

    loop {
        // Select the words depending on the selected category, flags, etc.
        let words = match category {
            Some(cat) => select_relevant_words(cat, &flags, &tags, 15),
            None => select_general_words(&flags, &tags),
        };

        if !exercises_only {
            if let Ok(mut list) = words {
                if inflection_only {
                    // If the '-i/--inflection' flag is passed, then don't
                    // discard the current selection, as that might be all of
                    // them when picking up a short category like pronouns.
                    list = vec![];
                } else if !run_words(&list, &locale) {
                    break;
                }

                let cats = match category {
                    Some(cat) => vec![cat],
                    None => vec![
                        Category::Noun,
                        Category::Adjective,
                        Category::Verb,
                        Category::Pronoun,
                    ],
                };
                if let Ok(words_to_inflect) = select_words_except(&list, &cats, &flags, &tags) {
                    if !run_inflect_words(&words_to_inflect, &locale) {
                        break;
                    }
                }
            }
        }

        if !inflection_only {
            if let Ok(exercises) =
                select_relevant_exercises(kind, if exercises_only { 5 } else { 1 })
            {
                if !run_exercises(exercises) {
                    break;
                }
            }
        }

        if !endless {
            break;
        }
    }
}
