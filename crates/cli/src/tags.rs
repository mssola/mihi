use inquire::{Confirm, Select};
use mihi::{create_tag, delete_tag, select_tag_names};
use std::vec::IntoIter;

// Show the help message.
fn help(msg: Option<&str>) {
    if let Some(msg) = msg {
        println!("{}.\n", msg);
    }

    println!("mihi tags: Manage tags.\n");
    println!("usage: mihi tags [OPTIONS] <subcommand>\n");

    println!("Options:");
    println!("   -h, --help\t\tPrint this message.");

    println!("\nSubcommands:");
    println!("   create\t\tCreate a new tag.");
    println!("   ls\t\t\tList tags from the database.");
    println!("   rm\t\t\tRemove a tag from the database.");
}

fn create(mut args: IntoIter<String>) -> i32 {
    // We expect exactly one argument, which is the name of the tag. Note that
    // this is wholly different to what's in for words/exercises, as the
    // expected workflow on those is different as well.
    if args.len() != 1 {
        let mut msg = "error: tags: you have to pass exactly one argument, which is the name of the tag to be created".to_string();
        if args.len() > 1 {
            msg.push_str(". You might want to wrap the given arguments in quotes");
        }

        help(Some(msg.as_str()));
        return 1;
    }

    // Fetch the name and guarantee it's unique.
    let name = args.next().unwrap_or("".to_string());
    if let Ok(tags) = select_tag_names(&Some(name.clone())) {
        for tag in tags {
            if tag == name {
                println!("errors: tags: '{}' already exists", name);
                return 1;
            }
        }
    }

    if create_tag(&name).is_ok() {
        0
    } else {
        1
    }
}

fn ls(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some("error: tags: too many filters"));
        return 1;
    }

    let tags = match select_tag_names(&args.next()) {
        Ok(tags) => tags,
        Err(e) => {
            println!("error: tags: {e}.");
            return 1;
        }
    };

    for tag in tags {
        println!("{tag}");
    }

    0
}

fn select_single_tag(search: Option<String>) -> Result<String, String> {
    let tags = select_tag_names(&search)?;

    match tags.len() {
        0 => Err("not found".to_string()),
        1 => Ok(tags.first().unwrap().to_owned()),
        _ => match Select::new("Which tag?", tags).with_page_size(20).prompt() {
            Ok(choice) => Ok(choice),
            Err(_) => Err("abort!".to_string()),
        },
    }
}

fn rm(mut args: IntoIter<String>) -> i32 {
    // We expect exactly one argument, which is the name of the tag. Note that
    // this is wholly different to what's in for words/exercises, as the
    // expected workflow on those is different as well.
    if args.len() != 1 {
        let mut msg = "error: tags: you have to pass exactly one argument, which is the name of the tag to be created".to_string();
        if args.len() > 1 {
            msg.push_str(". You might want to wrap the given arguments in quotes");
        }

        help(Some(msg.as_str()));
        return 1;
    }

    // Select exactly one tag, with or without user feedback. If that's not
    // possible, bail out. If one is selected, allow the user to think it
    // through.
    let selection = match select_single_tag(args.next()) {
        Ok(tag) => tag,
        Err(e) => {
            println!("error: tags: {e}.");
            return 1;
        }
    };
    let ans = Confirm::new(
        format!("Do you really want to remove '{selection}' from the database?").as_str(),
    )
    .with_default(false)
    .prompt();

    // We have a selected tag and the user confirmed its selection, go for it!
    match ans {
        Ok(true) => match delete_tag(&selection) {
            Ok(_) => println!("Removed '{selection}' from the database!"),
            Err(e) => {
                println!("error: tags: {e}.");
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
            "error: tags: you have to provide at least a subcommand",
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
                    format!("error: tags: unknown flag or command '{first}'").as_str(),
                ));
                std::process::exit(1);
            }
        },
        None => {
            help(Some(
                "error: tags: you need to provide a command"
                    .to_string()
                    .as_str(),
            ));
            std::process::exit(1);
        }
    }
}
