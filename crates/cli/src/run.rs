use inquire::Text;
use mihi::{select_random_words, update_success, Category, Word};

fn help(msg: Option<&str>) {
    if msg.is_some() {
        println!("{}.\n", msg.unwrap());
    }

    println!("mihi run: Run exercises. Default command if none was given.\n");
    println!("usage: mihi run [OPTIONS]\n");

    println!("Options:");
    println!("   -h, --help\t\tPrint this message.");
}

enum Locale {
    English,
    Catalan,
}

impl Locale {
    fn to_code(&self) -> &str {
        match self {
            Self::English => "en",
            Self::Catalan => "ca",
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::English => write!(f, "english"),
            Self::Catalan => write!(f, "català"),
        }
    }
}

fn run_words(words: Vec<Word>, locale: Locale) -> i32 {
    let mut errors = 0;

    for word in words {
        // If the translation cannot be found, skip this word.
        let Some(translation) = word.translation.get(locale.to_code()) else {
            continue;
        };

        println!("Word: {}", word.enunciated);

        let Ok(raw) = Text::new(format!("Translation ({}):", locale).as_str()).prompt() else {
            return 1;
        };
        let answer = raw.trim();

        let tr = translation.as_str().unwrap_or("");
        let found = !answer.is_empty() && tr.split(',').any(|tr| tr.trim().contains(&answer));

        if found {
            let _ = update_success(&word, word.succeeded + 1);
            println!("\x1b[92m✓ {}\x1b[0m", tr);
        } else {
            if word.succeeded > 0 {
                let _ = update_success(&word, word.succeeded - 1);
            }
            println!("\x1b[91m❌{}\x1b[0m", tr);
            errors += 1;
        }
    }

    errors
}

fn select_general_words() -> Result<Vec<Word>, String> {
    let mut res = select_random_words(Category::Noun, 4)?;
    res.append(&mut select_random_words(Category::Adjective, 2)?);
    res.append(&mut select_random_words(Category::Verb, 4)?);
    res.append(&mut select_random_words(Category::Pronoun, 1)?);
    res.append(&mut select_random_words(Category::Adverb, 2)?);
    res.append(&mut select_random_words(Category::Preposition, 1)?);
    res.append(&mut select_random_words(Category::Conjunction, 1)?);
    Ok(res)
}

pub fn run(args: Vec<String>) {
    let mut it = args.into_iter();
    let mut category = None;

    while let Some(first) = it.next() {
        match first.as_str() {
            "-h" | "--help" => {
                help(None);
                std::process::exit(0);
            }
            "-c" | "--category" => {
                if category.is_some() {
                    help(Some("error: run: you cannot provide multiple categories"));
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
                            _ => return help(Some("error: run: category not allowed")),
                        };
                    }
                    None => help(Some("error: run: you have to provide a category")),
                }
            }
            _ => {
                help(Some(
                    format!("error: run: unknown flag or command '{}'", first).as_str(),
                ));
                std::process::exit(1);
            }
        }
    }

    let raw_locale = std::env::var("LC_ALL").unwrap_or("en".to_string());
    let locale = if raw_locale.starts_with("ca") {
        Locale::Catalan
    } else {
        Locale::English
    };

    let words = match category {
        Some(cat) => select_random_words(cat, 15),
        None => select_general_words(),
    };

    match words {
        Ok(list) => std::process::exit(run_words(list, locale)),
        Err(e) => {
            println!("error: run: {}", e);
            std::process::exit(1);
        }
    };
}
