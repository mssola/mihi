use inquire::{Confirm, Editor, Select, Text};
use std::vec::IntoIter;

use mihi::{create_word, delete_word, select_enunciated, Category, Gender, Language, Word};

static NEW_MESSAGE: &str = "New word";
static NEXT_MESSAGE: &str = "Skip this one!";
static QUIT_MESSAGE: &str = "Quit!";

static FLAGS_TEXT: &str = r#"# Write a JSON blob with the following allowed keys.
#
# => Boolean
#
# deponent:            This is a Latin deponent verb.
# onlysingular:        It only has singular forms.
# onlyplural:          It only has plural forms.
# contracted_root:     The root contracts for certain forms (e.g. '_liber_' vs '_libr_ī').
# nonpositive:         This is a non-positive word.
# compsup_prefix:      Comparative and superlative forms require a prefix.
# indeclinable:        It cannot be declined :-)
# irregularsup:        The superlative is irregular.
# nopassive:           Verb has no passive form.
# nosupine:            Verb has no supine form.
# noperfect:           Verb has no perfect forms.
# nogerundive:         Verb has no gerundive.
# impersonal:          Verb is impersonal (only third person available).
# impersonalpassive:   Verb is impersonal only on its passive forms.
# noimperative:        Verb has no imperative forms.
# noinfinitive:        Verb has no infinitive forms.
# shortimperative:     The imperative form is a short version.
# onlythirdpassive:    Verb has only forms on the third person of the passive voice.
# enclitic:            This is simply an enclitic.
# notcomparable:       There cannot be a comparable version for this word
# onlyperfect:         Only perfect forms are available.
# semideponent:        This is a Latin semi-deponent verb.
# contracted_vocative  The vocative contracts the root by one character.
#
# => More complex flags
#
# adds:                There are some cases that are to be added to existing ones.
# sets:                There are some cases which need to replace the existing ones.
#
# For example:
#
# {
#   "onlysingular": true,
#   "sets": {
#     "accusative": {
#       "singular": ["im"]
#     }
#   }
# }

{
}
"#;

fn help(msg: Option<&str>) {
    if msg.is_some() {
        println!("{}.\n", msg.unwrap());
    }

    println!("mihi words: Manage words.\n");
    println!("usage: mihi words [OPTIONS] <subcommand>\n");

    println!("Options:");
    println!("   -h, --help\t\tPrint this message.");

    println!("\nSubcommands:");
    println!("   create\t\tCreate a new word.");
    println!("   ls\t\t\tList the words from the database.");
    println!("   rm\t\t\tRemove a word from the database.");
    println!("   show\t\t\tShow information from a word.");
}

#[derive(Default)]
struct Guess {
    particle: String,
    category: Category,
    inflection_id: usize,
    gender: Gender,
    kind: String,
}

fn get_initial_guess(value: &str) -> Guess {
    let parts = value.trim().split(',').collect::<Vec<_>>();

    if parts.len() == 2 {
        let first = parts.first().unwrap();
        let second = parts.last().unwrap();

        if first.ends_with('a') && second.ends_with("ae") {
            return Guess {
                particle: first[0..first.len() - 1].to_string(),
                category: Category::Noun,
                inflection_id: 1,
                gender: Gender::Feminine,
                kind: "a".to_string(),
            };
        } else if first.ends_with("us") && second.ends_with("ī") {
            return Guess {
                particle: first[0..first.len() - 2].to_string(),
                category: Category::Noun,
                inflection_id: 2,
                gender: Gender::Masculine,
                kind: "us".to_string(),
            };
        } else if first.ends_with("um") && second.ends_with("ī") {
            return Guess {
                particle: first[0..first.len() - 2].to_string(),
                category: Category::Noun,
                inflection_id: 2,
                gender: Gender::Neuter,
                kind: "um".to_string(),
            };
        } else if first.ends_with("us") && second.ends_with("ūs") {
            return Guess {
                particle: first[0..first.len() - 2].to_string(),
                category: Category::Noun,
                inflection_id: 4,
                gender: Gender::Masculine,
                kind: "fus".to_string(),
            };
        } else if first.ends_with("ū") && second.ends_with("ūs") {
            return Guess {
                particle: first[0..first.len() - 1].to_string(),
                category: Category::Noun,
                inflection_id: 4,
                gender: Gender::Masculine,
                kind: "fus".to_string(),
            };
        } else if first.ends_with("iēs") && second.ends_with("ēī") {
            return Guess {
                particle: first[0..first.len() - 3].to_string(),
                category: Category::Noun,
                inflection_id: 5,
                gender: Gender::Masculine,
                kind: "ies".to_string(),
            };
        } else if first.ends_with("ēs") && second.ends_with("eī") {
            return Guess {
                particle: first[0..first.len() - 2].to_string(),
                category: Category::Noun,
                inflection_id: 5,
                gender: Gender::Masculine,
                kind: "es".to_string(),
            };
        } else if second.ends_with("is") {
            return Guess {
                particle: second[0..second.len() - 2].to_string(),
                category: Category::Noun,
                inflection_id: 5,
                gender: Gender::Masculine,
                kind: "es".to_string(),
            };
        }
    }

    Guess::default()
}

// Remove comments from the "flags" text that was provided.
fn trim_flags(given: String) -> String {
    let mut res = String::new();

    for line in given.lines() {
        let trimmed = line.trim();

        if !line.trim().starts_with('#') {
            res.push_str(trimmed);
        }
    }

    res
}

fn do_create(enunciated: String) -> Result<(), String> {
    let guess = get_initial_guess(enunciated.as_str());

    let Ok(particle) = Text::new("Particle:")
        .with_initial_value(&guess.particle)
        .prompt()
    else {
        return Err("abort!".to_string());
    };

    let categories = vec![
        Category::Unknown,
        Category::Noun,
        Category::Adjective,
        Category::Verb,
        Category::Pronoun,
        Category::Adverb,
        Category::Preposition,
        Category::Conjunction,
        Category::Interjection,
        Category::Determiner,
    ];
    let Ok(category) = Select::new("Category:", categories)
        .with_starting_cursor(guess.category as usize)
        .prompt()
    else {
        return Err("abort!".to_string());
    };

    let genders = vec![
        Gender::Masculine,
        Gender::Feminine,
        Gender::MasculineOrFeminine,
        Gender::Neuter,
        Gender::None,
    ];
    let gender = match category {
        Category::Noun => {
            match Select::new("Gender:", genders)
                .with_starting_cursor(guess.gender as usize)
                .prompt()
            {
                Ok(selection) => selection,
                Err(_) => return Err("abort!".to_string()),
            }
        }
        _ => Gender::None,
    };

    let Ok(inflection) = Text::new("Inflection:")
        .with_initial_value(&guess.inflection_id.to_string())
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let Ok(inflection_id) = inflection.parse::<usize>() else {
        return Err(format!("bad value for inflection ID '{inflection}'"));
    };

    let Ok(kind) = Text::new("Kind:").with_initial_value(&guess.kind).prompt() else {
        return Err("abort!".to_string());
    };

    let Ok(regular) = Confirm::new("Regular:").with_default(true).prompt() else {
        return Err("abort!".to_string());
    };
    let Ok(locative) = Confirm::new("Locative:").with_default(false).prompt() else {
        return Err("abort!".to_string());
    };

    let Ok(flags) = Editor::new("Flags:")
        .with_predefined_text(FLAGS_TEXT)
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let trimmed_flags = trim_flags(flags);

    let Ok(translation_en) = Text::new("Translation (english):").prompt() else {
        return Err("abort!".to_string());
    };
    let Ok(translation_ca) = Text::new("Translation (catalan):").prompt() else {
        return Err("abort!".to_string());
    };

    let word = Word {
        id: 0,
        enunciated: enunciated.clone(),
        particle,
        language: Language::Latin,
        declension_id: if matches!(category, Category::Verb) {
            None
        } else {
            Some(inflection_id)
        },
        conjugation_id: if matches!(category, Category::Verb) {
            Some(inflection_id)
        } else {
            None
        },
        kind,
        category,
        regular,
        locative,
        gender,
        suffix: None,
        translation: serde_json::from_str(
            format!(
                "{{\"en\":\"{}\", \"ca\":\"{}\"}}",
                translation_en.trim(),
                translation_ca.trim()
            )
            .as_str(),
        )
        .unwrap(),
        flags: serde_json::from_str(&trimmed_flags).unwrap(),
        succeeded: 0,
        steps: 0,
    };

    match create_word(word) {
        Ok(_) => {
            println!("Word '{enunciated}' has been successfully created!");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn create(args: IntoIter<String>) -> i32 {
    if args.len() > 0 {
        help(Some(
            "error: words: no arguments were expected for this command",
        ));
        return 1;
    }

    loop {
        // Grab the enunciate from the word that we want to create.
        let Ok(enunciated) = Text::new("Enunciated:").prompt() else {
            return 1;
        };
        if enunciated.trim().is_empty() {
            return 0;
        }

        // Now we try to fetch whether the word already existed, by doing a
        // general search on the database.
        let mut words = match select_enunciated(Some(enunciated.clone())) {
            Ok(words) => words,
            Err(e) => {
                println!("error: words: {e}");
                return 1;
            }
        };
        words.push(NEW_MESSAGE.to_string());
        words.push(NEXT_MESSAGE.to_string());
        words.push(QUIT_MESSAGE.to_string());

        match words.len() {
            // Seems confusing, but we fill the "words" list with three default
            // "messages" which are part of the interface. Hence, if only three
            // "words" exist, then it's just the interface and we can go right
            // into creating the word.
            3 => {
                if let Err(e) = do_create(enunciated) {
                    println!("error: words: {e}");
                    return 1;
                }
            }
            _ => match Select::new("Is your word on this list?", words).prompt() {
                Ok(choice) => {
                    if choice == QUIT_MESSAGE {
                        return 0;
                    } else if choice == NEW_MESSAGE {
                        if let Err(e) = do_create(enunciated) {
                            println!("error: words: {e}");
                            return 1;
                        }
                    }
                }
                Err(_) => return 1,
            },
        };
    }
}

fn ls(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some("error: words: too many filters"));
        return 1;
    }

    let words = match select_enunciated(args.next()) {
        Ok(words) => words,
        Err(e) => {
            println!("error: words: {e}");
            return 1;
        }
    };

    // TODO: not just the enunciated, but being able to edit
    for enunciated in words {
        println!("{enunciated}");
    }

    0
}

fn rm(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some("error: words: too many filters"));
        return 1;
    }

    let words = match select_enunciated(args.next()) {
        Ok(words) => words,
        Err(e) => {
            println!("error: words: {e}");
            return 1;
        }
    };

    let selection: String = match words.len() {
        0 => {
            println!("errors: words: not found!");
            return 1;
        }
        1 => words.first().unwrap().to_owned(),
        _ => match Select::new("Which word?", words).prompt() {
            Ok(choice) => choice,
            Err(_) => return 1,
        },
    };

    let ans = Confirm::new(
        format!("Do you really want to remove '{selection}' from the database?").as_str(),
    )
    .with_default(true)
    .prompt();

    match ans {
        Ok(true) => match delete_word(&selection) {
            Ok(_) => println!("Removed '{selection}' from the database!"),
            Err(e) => {
                println!("error: words: {e}");
                return 1;
            }
        },
        Ok(false) => {
            println!("Doing nothing...");
        }
        Err(_) => return 1,
    }

    0
}

pub fn run(args: Vec<String>) {
    if args.is_empty() {
        help(Some(
            "error: words: you have to provide at least a subcommand",
        ));
        std::process::exit(1);
    }

    let mut it = args.into_iter();

    match it.next() {
        Some(first) => match first.as_str() {
            "-h" | "--help" => {
                help(None);
                std::process::exit(0);
            }
            "create" => {
                std::process::exit(create(it));
            }
            "ls" => {
                std::process::exit(ls(it));
            }
            "rm" => {
                std::process::exit(rm(it));
            }
            _ => {
                help(Some(
                    format!("error: words: unknown flag or command '{first}'").as_str(),
                ));
                std::process::exit(1);
            }
        },
        None => {
            help(Some(
                "error: words: you need to provide a command"
                    .to_string()
                    .as_str(),
            ));
            std::process::exit(1);
        }
    }
}
