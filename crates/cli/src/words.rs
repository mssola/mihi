use crate::inflection::print_full_inflection_for;
use crate::locale::current_locale;

use inquire::{Confirm, Editor, Select, Text};
use mihi::{Category, Gender, Language, Word};
use std::vec::IntoIter;

static NEW_MESSAGE: &str = "New word";
static NEXT_MESSAGE: &str = "Skip this one!";
static QUIT_MESSAGE: &str = "Quit!";

// Documentation text which is prepended to editing flags.
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
"#;

// Show the help message.
fn help(msg: Option<&str>) {
    if let Some(msg) = msg {
        println!("{}.\n", msg);
    }

    println!("mihi words: Manage words.\n");
    println!("usage: mihi words [OPTIONS] <subcommand>\n");

    println!("Options:");
    println!("   -h, --help\t\tPrint this message.");

    println!("\nSubcommands:");
    println!("   create\t\tCreate a new word.");
    println!("   edit\t\t\tEdit information from a word.");
    println!("   ls\t\t\tList the words from the database.");
    println!("   rm\t\t\tRemove a word from the database.");
    println!("   show\t\t\tShow information from a word.");
}

// Given an enunciated value, try to guess a word from it. If that's not
// possible then an empty word is given.
fn get_initial_guess(value: &str) -> Word {
    let parts = value.trim().split(',').collect::<Vec<_>>();

    if parts.len() == 2 {
        let first = parts.first().unwrap();
        let second = parts.last().unwrap();

        if first.ends_with('a') && second.ends_with("ae") {
            return Word::from(
                first[0..first.len() - 1].to_string(),
                Category::Noun,
                Some(1),
                None,
                Gender::Feminine,
                "a".to_string(),
            );
        } else if first.ends_with("us") && second.ends_with("ī") {
            return Word::from(
                first[0..first.len() - 2].to_string(),
                Category::Noun,
                Some(2),
                None,
                Gender::Masculine,
                "us".to_string(),
            );
        } else if first.ends_with("um") && second.ends_with("ī") {
            return Word::from(
                first[0..first.len() - 2].to_string(),
                Category::Noun,
                Some(2),
                None,
                Gender::Neuter,
                "um".to_string(),
            );
        } else if first.ends_with("us") && second.ends_with("ūs") {
            return Word::from(
                first[0..first.len() - 2].to_string(),
                Category::Noun,
                Some(4),
                None,
                Gender::Masculine,
                "fus".to_string(),
            );
        } else if first.ends_with("ū") && second.ends_with("ūs") {
            return Word::from(
                first[0..first.len() - 1].to_string(),
                Category::Noun,
                Some(4),
                None,
                Gender::Masculine,
                "fus".to_string(),
            );
        } else if first.ends_with("iēs") && second.ends_with("ēī") {
            return Word::from(
                first[0..first.len() - 3].to_string(),
                Category::Noun,
                Some(5),
                None,
                Gender::Masculine,
                "ies".to_string(),
            );
        } else if first.ends_with("ēs") && second.ends_with("eī") {
            return Word::from(
                first[0..first.len() - 2].to_string(),
                Category::Noun,
                Some(5),
                None,
                Gender::Masculine,
                "es".to_string(),
            );
        } else if second.ends_with("is") {
            return Word::from(
                second[0..second.len() - 2].to_string(),
                Category::Noun,
                Some(3),
                None,
                Gender::Masculine,
                "is".to_string(),
            );
        }
    }

    Word::from(
        value.to_string(),
        Category::Unknown,
        None,
        None,
        Gender::None,
        String::from("-"),
    )
}

// Remove comments from the "flags" text that was provided.
fn trim_flags(given: String) -> String {
    let mut res = String::new();

    for line in given.lines() {
        let trimmed = line.trim();

        if !trimmed.starts_with('#') {
            res.push_str(trimmed);
        }
    }

    res
}

// Get the translation from `word.translated` which matches the given language
// `key`. If that cannot be found, or for some reason is not a String, then an
// error is returned.
fn get_translated<'a>(word: &'a Word, key: &'a str) -> Result<&'a String, String> {
    match word.translation.get(key) {
        Some(value) => match value {
            serde_json::Value::String(s) => Ok(s),
            _ => Err("unexpected key type".to_string()),
        },
        None => Err("key does not exist".to_string()),
    }
}

// Interactively ask the user to provide information for a word by the given
// `enunciated`. The default values will be based on the given `word` parameter.
fn ask_for_word_based_on(enunciated: String, word: Word) -> Result<Word, String> {
    let Ok(particle) = Text::new("Particle:")
        .with_initial_value(&word.particle)
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let particle = particle.trim().to_string();

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
        .with_starting_cursor(word.category as usize)
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
                .with_starting_cursor(word.gender as usize)
                .prompt()
            {
                Ok(selection) => selection,
                Err(_) => return Err("abort!".to_string()),
            }
        }
        _ => Gender::None,
    };

    let inflection_id = match category {
        Category::Noun | Category::Adjective | Category::Verb => {
            let Ok(inflection) = Text::new("Inflection:")
                .with_initial_value(word.inflection_id().unwrap_or(0).to_string().as_str())
                .prompt()
            else {
                return Err("abort!".to_string());
            };
            let Ok(inflection_id) = inflection.parse::<usize>() else {
                return Err(format!("bad value for inflection ID '{inflection}'"));
            };
            Some(inflection_id)
        }
        _ => None,
    };

    // TODO: refine guess once the inflection is known: select from possible values.
    let kind = match category {
        Category::Noun | Category::Adjective => {
            let Ok(kind) = Text::new("Kind:").with_initial_value(&word.kind).prompt() else {
                return Err("abort!".to_string());
            };
            kind.trim().to_string()
        }
        Category::Verb => String::from("verb"),
        _ => String::from("-"),
    };

    let regular = if matches!(
        category,
        Category::Noun | Category::Adjective | Category::Verb
    ) {
        let Ok(regular) = Confirm::new("Regular:").with_default(word.regular).prompt() else {
            return Err("abort!".to_string());
        };
        regular
    } else {
        true
    };

    let locative = if matches!(category, Category::Noun) {
        let Ok(locative) = Confirm::new("Locative:")
            .with_default(word.locative)
            .prompt()
        else {
            return Err("abort!".to_string());
        };
        locative
    } else {
        false
    };

    let Ok(raw_weight) = Text::new("Weight:")
        .with_initial_value(word.weight.to_string().as_str())
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let Ok(weight) = raw_weight.parse::<usize>() else {
        return Err(format!(
            "bad value for inflection ID '{}'",
            inflection_id.unwrap_or(0)
        ));
    };
    if weight > 10 {
        return Err(format!(
            "weight has to be an integer between 0 and 10, but {weight} was given"
        ));
    }

    let raw_flags = serde_json::to_string(&word.flags).unwrap();

    let Ok(flags) = Editor::new("Flags:")
        .with_predefined_text(format!("{FLAGS_TEXT}\n{raw_flags}").as_str())
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let trimmed_flags = trim_flags(flags);

    let Ok(translation_en) = Text::new("Translation (english):")
        .with_initial_value(get_translated(&word, "en").unwrap_or(&String::from("")))
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let Ok(translation_ca) = Text::new("Translation (catalan):")
        .with_initial_value(get_translated(&word, "ca").unwrap_or(&String::from("")))
        .prompt()
    else {
        return Err("abort!".to_string());
    };

    Ok(Word {
        id: word.id,
        enunciated,
        particle,
        language: Language::Latin,
        declension_id: if matches!(category, Category::Verb) {
            None
        } else {
            inflection_id
        },
        conjugation_id: if matches!(category, Category::Verb) {
            inflection_id
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
        weight,
    })
}

// Interactively ask the user for the given `enunciated`, build up a Word object
// from it, and insert it into the database.
fn do_create(enunciated: String) -> Result<(), String> {
    let mut guess = get_initial_guess(enunciated.as_str());
    guess.enunciated = enunciated.trim().to_string();

    let word = ask_for_word_based_on(enunciated.clone(), guess)?;
    match mihi::create_word(word) {
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
        let mut words = match mihi::select_enunciated(Some(enunciated.clone())) {
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

    let words = match mihi::select_enunciated(args.next()) {
        Ok(words) => words,
        Err(e) => {
            println!("error: words: {e}");
            return 1;
        }
    };

    for enunciated in words {
        println!("{enunciated}");
    }

    0
}

// Given a search parameter, returns the word that match the enunciate. If
// multiple words match the same search parameter, then the user is asked to
// select one from a list of candidates.
fn select_single_word(search: Option<String>) -> Result<String, String> {
    let words = mihi::select_enunciated(search)?;

    match words.len() {
        0 => Err("not found".to_string()),
        1 => Ok(words.first().unwrap().to_owned()),
        _ => match Select::new("Which word?", words)
            .with_page_size(20)
            .prompt()
        {
            Ok(choice) => Ok(choice),
            Err(_) => Err("abort!".to_string()),
        },
    }
}

fn edit(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some(
            "error: words: only one argument. If it's an enunciate, wrap it in double quotes",
        ));
        return 1;
    }

    // Only one word can be modified at a time.
    let enunciated = match select_single_word(args.next()) {
        Ok(word) => word,
        Err(e) => {
            println!("error: words: {e}");
            return 1;
        }
    };

    // Fetch the word object for it which will serve as the initial values.
    let word = match mihi::find_by(enunciated.as_str()) {
        Ok(word) => word,
        Err(e) => {
            println!("error: words: {e}");
            return 1;
        }
    };

    // The enunciate might change, let's ask for it again. This way we get the
    // same experience as with the 'create' command.
    let Ok(enunciated) = Text::new("Enunciated:")
        .with_initial_value(&word.enunciated)
        .prompt()
    else {
        return 1;
    };
    if enunciated.trim().is_empty() {
        return 0;
    }

    // And ask again column by column to check for changes.
    let updated = match ask_for_word_based_on(enunciated.clone(), word) {
        Ok(word) => word,
        Err(e) => {
            println!("error: words: {e}");
            return 1;
        }
    };

    match mihi::update_word(updated) {
        Ok(_) => {
            println!("Word '{enunciated}' has been updated!");
            0
        }
        Err(e) => {
            println!("error: words: {e}");
            1
        }
    }
}

// Returns a string with a more human-readable declension kind.
fn humanize_kind(kind: &str) -> &str {
    match kind {
        // Noun
        "a" => "-a",
        "us" => "-us",
        "er/ir" => "-er/-ir",
        "um" => "-um",
        "ius" => "-ius; like 'fīlius'",
        "is" => "-is",
        "istem" => "i-stem; '-i-' also in the genitive plural",
        "pureistem" => "pure i-stem; '-i-' also in the ablative singular",
        "visvis" => "irregular 'vīs, vīs'",
        "sussuis" => "irregular 'sūs, suis'",
        "bosbovis" => "irregular 'bōs, bovis'",
        "iuppiteriovis" => "irregular 'Iuppiter, Iovis'",
        "fus" => "-u-",
        "domusdomus" => "irregular 'domus, domūs/domī'",
        "ies" => "-iēs; like 'diēs, diēī'",
        "es" => "-ēs; like 'rēs, reī'",
        "indeclinable" => "indeclinable",

        // Adjective
        "one" => "one termination adjective",
        "onenonistem" => "one termination adjective; non i-stem like 'melior, melius'",
        "two" => "two termination adjective",
        "three" => "three termination adjective",
        "unusnauta" => "'ūnus nauta' like 'ūnus, ūna, ūnum'",
        "unusnautaer/ir" => "'ūnus nauta' like 'neuter, neutra, neutrum'",
        "duo" => "number 'duo, duae, duo'",
        "tres" => "number 'trēs, trēs, tria'",
        "mille" => "number 'mīlle, mīlle'",

        // Others
        "egonos" => "'ego, nōs'",
        "demonstrative-weak" => "weak demonstrative",
        "demonstrative-proximal" => "proximal demonstrative",
        "demonstrative-distal" => "distal demonstrative",
        "demonstrative-medial" => "medial demonstrative",
        "demonstrative-idem" => "'īdem, eadem, idem' demonstrative",
        "tuvos" => "'tū, vōs'",
        "sesui" => "'sē, suī'",

        _ => kind,
    }
}

fn show_info(word: Word) -> Result<(), String> {
    // Title.
    match word.gender {
        Gender::None => println!("Word: {} ({})", word.enunciated, word.category),
        _ => println!(
            "Word: {} ({} {})",
            word.enunciated,
            word.gender.abbrev(),
            word.category
        ),
    }

    // Conjugation, declension + kind.
    // TODO: to_human
    match word.conjugation_id {
        Some(id) => println!("Conjugation: {}", id),
        None => {
            if let Some(did) = word.declension_id {
                if did > 5 {
                    println!("Declension: {}", humanize_kind(&word.kind));
                } else {
                    println!(
                        "Declension: {} ({})",
                        word.declension_id.unwrap(),
                        humanize_kind(&word.kind)
                    );
                }
            }
        }
    };

    // Show translation if available.
    let locale = current_locale();
    if let Some(translation) = word.translation.get(locale.to_code()) {
        let s = translation.as_str().unwrap_or("");
        if !s.is_empty() {
            println!("Translation ({}): {}.", locale.to_code(), s);
        }
    }

    print_full_inflection_for(word)?;

    Ok(())
}

fn show(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some(
            "error: words: only one argument. If it's an enunciate, wrap it in double quotes",
        ));
        return 1;
    }

    let enunciated = match select_single_word(args.next()) {
        Ok(word) => word,
        Err(e) => {
            println!("error: words: {e}.");
            return 1;
        }
    };

    let word = match mihi::find_by(enunciated.as_str()) {
        Ok(word) => word,
        Err(e) => {
            println!("error: words: {e}.");
            return 1;
        }
    };

    if let Err(e) = show_info(word) {
        println!("error: words: {e}.");
        return 1;
    }

    0
}

fn rm(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some("error: words: too many filters"));
        return 1;
    }

    let selection = match select_single_word(args.next()) {
        Ok(word) => word,
        Err(e) => {
            println!("error: words: {e}");
            return 1;
        }
    };

    let ans = Confirm::new(
        format!("Do you really want to remove '{selection}' from the database?").as_str(),
    )
    .with_default(false)
    .prompt();

    match ans {
        Ok(true) => match mihi::delete_word(&selection) {
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
            "edit" => {
                std::process::exit(edit(it));
            }
            "ls" => {
                std::process::exit(ls(it));
            }
            "rm" => {
                std::process::exit(rm(it));
            }
            "show" => {
                std::process::exit(show(it));
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
