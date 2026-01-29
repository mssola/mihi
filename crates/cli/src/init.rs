fn help() {
    println!("mihi init: Initialize 'mihi' for a given language.\n");
    println!("usage: mihi init [OPTIONS]\n");

    println!("Options:");
    println!("   -h, --help\t\tPrint this message.");
    println!("   -l, --language\tThe language to be used.");
}

pub fn run(args: Vec<String>) {
    let mut given_language: Option<String> = None;
    let mut it = args.into_iter();

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                help();
                std::process::exit(0);
            }
            "-l" | "--language" => match it.next() {
                Some(lang) => given_language = Some(lang),
                None => {
                    println!(
                        "error: init: you have to provide a value for the '-l/--language' flag"
                    );
                    std::process::exit(1);
                }
            },
            _ => {
                println!("error: init: unknown flag: '{}'", arg.as_str());
                std::process::exit(1);
            }
        }
    }

    // Assume latin for now. If we support more languages in the future, there
    // should be a prompt or something.
    let language = match given_language {
        Some(lang) => lang,
        None => String::from("latin"),
    };

    match init(language) {
        Ok(_) => {}
        Err(e) => {
            println!("error: init: {e}");
            std::process::exit(1);
        }
    }
}

fn init(language: String) -> Result<(), String> {
    mihi::cfg::add_language(language)
}
