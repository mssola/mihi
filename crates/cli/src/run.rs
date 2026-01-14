use inquire::{Confirm, Editor, Text};
use mihi::touch_exercise;
use mihi::{select_relevant_words, update_success, Category, Exercise, ExerciseKind, Word};
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
    println!("   -k, --kind <KIND>\t\tOnly ask for exercises for the given <KIND>.");
}

// Run the quiz for all the given `words` while expecting answers to be
// delivered in the given `locale`.
fn run_words(words: Vec<Word>, locale: &Locale) -> bool {
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
                let _ = update_success(&word, word.succeeded + 1, 0);
            } else {
                let _ = update_success(&word, word.succeeded, word.steps + 1);
            }
            println!("\x1b[92m✓ {tr}\x1b[0m");
        } else {
            if word.succeeded > 0 {
                let _ = update_success(&word, word.succeeded - 1, 0);
            }
            println!("\x1b[91m❌{tr}\x1b[0m");
        }
    }

    true
}

// Returns a vector of words which contain a randomized set of words from
// different categories.
fn select_general_words(flags: &Vec<String>) -> Result<Vec<Word>, String> {
    let mut res = select_relevant_words(Category::Noun, flags, 4)?;
    res.append(&mut select_relevant_words(Category::Adjective, flags, 2)?);
    res.append(&mut select_relevant_words(Category::Verb, flags, 4)?);
    res.append(&mut select_relevant_words(Category::Pronoun, flags, 1)?);
    res.append(&mut select_relevant_words(Category::Adverb, flags, 2)?);
    res.append(&mut select_relevant_words(Category::Preposition, flags, 1)?);
    res.append(&mut select_relevant_words(Category::Conjunction, flags, 1)?);
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
fn accepted_diff(given: String, expected: &String) -> bool {
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
        if accepted_diff(solution, &exercise.solution) {
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
    let mut endless = false;
    let mut flags: Vec<String> = vec![];

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
            "--endless" => {
                endless = true;
            }
            "-f" | "--flag" => match it.next() {
                Some(flag) => {
                    if mihi::is_valid_word_flag(flag.as_str()) {
                        if flags.iter().any(|s| s.as_str() == flag) {
                            println!(
                                "warning: practice: flag '{flag}' was provided multiple times"
                            );
                        } else {
                            flags.push(flag);
                        }
                    } else {
                        let supported = mihi::BOOLEAN_FLAGS.join(", ");
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
        let words = match category {
            Some(cat) => select_relevant_words(cat, &flags, 15),
            None => select_general_words(&flags),
        };

        let exercises =
            match mihi::select_relevant_exercises(kind, if exercises_only { 5 } else { 1 }) {
                Ok(exercises) => exercises,
                Err(e) => {
                    println!("error: practice: {e}");
                    std::process::exit(1);
                }
            };

        let code = match words {
            Ok(list) => {
                if !exercises_only
                    && !run_words(list, &locale) {
                        std::process::exit(1);
                    }
                if !run_exercises(exercises) {
                    std::process::exit(1);
                }

                0
            }
            Err(e) => {
                println!("error: practice: {e}");
                1
            }
        };

        if !endless {
            std::process::exit(code);
        }
    }
}
