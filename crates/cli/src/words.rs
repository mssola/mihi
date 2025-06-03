use inquire::{Confirm, Select};
use std::vec::IntoIter;

use mihi::{delete_word, select_enunciated};

fn help() {
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

fn create(args: IntoIter<String>) -> i32 {
    println!("TBD: {:#?}", args);

    return 0;
}

fn show(args: IntoIter<String>) -> i32 {
    println!("TBD: {:#?}", args);

    return 0;
}

fn ls(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        println!("error: words: too many filters");
        return 1;
    }

    let words = match select_enunciated(args.next()) {
        Ok(words) => words,
        Err(e) => {
            println!("error: words: {}", e);
            return 1;
        }
    };

    for enunciated in words {
        println!("{}", enunciated);
    }

    return 0;
}

fn rm(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        println!("error: words: too many filters");
        return 1;
    }

    let words = match select_enunciated(args.next()) {
        Ok(words) => words,
        Err(e) => {
            println!("error: words: {}", e);
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
        format!(
            "Do you really want to remove '{}' from the database?",
            selection
        )
        .as_str(),
    )
    .with_default(true)
    .prompt();

    match ans {
        Ok(true) => match delete_word(&selection) {
            Ok(_) => println!("Removed '{}' from the database!", selection),
            Err(e) => {
                println!("error: words: {}", e);
                return 1;
            }
        },
        Ok(false) => {
            println!("Doing nothing...");
        }
        Err(_) => return 1,
    }

    return 0;
}

pub fn run(args: Vec<String>) {
    if args.is_empty() {
        println!("error: words: you have to provide at least a subcommand");
        std::process::exit(1);
    }

    let mut it = args.into_iter();

    while let Some(first) = it.next() {
        match first.as_str() {
            "-h" | "--help" => {
                help();
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
            "show" => {
                std::process::exit(show(it));
            }
            _ => {
                println!(
                    "error: words: unknown flag or command: '{}'",
                    first.as_str()
                );
                std::process::exit(1);
            }
        }
    }
}
