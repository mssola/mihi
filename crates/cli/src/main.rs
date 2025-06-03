mod init;
mod nuke;
mod run;
mod words;

/// Version for this program.
const VERSION: &str = "0.1.0";

fn help() {
    println!("Self-assessment tool for language learning.\n");
    println!("usage: mihi [OPTIONS] [COMMAND] [COMMAND OPTIONS]\n");

    println!("Options:");
    println!("   -h, --help\t\tPrint this message.");
    println!("   -v, --version\tPrint the version of this program.\n");

    println!("Commands:");
    println!("   init\t\t\tInitialize the configuration for this application.");
    println!("   nuke\t\t\tRemove all files from this application and its database.");
    println!("   run\t\t\tRun exercises. Default command if none was given.");
    println!("   words\t\tManage the words for this application.");
}

fn main() {
    let mut args = std::env::args();
    let nargs = std::env::args().count();
    let mut count = 1;

    // Skip command name.
    args.next();

    // And iterate over the arguments.
    while let Some(arg) = args.next() {
        count += 1;

        match arg.as_str() {
            "-h" | "--help" => {
                if nargs > count {
                    println!(
                        "warning: arguments passed the '{}' flag will be ignored.",
                        arg.as_str()
                    );
                }
                help();
                std::process::exit(0);
            }
            "-v" | "--version" => {
                if nargs > count {
                    println!(
                        "warning: arguments passed the '{}' flag will be ignored.",
                        arg.as_str()
                    );
                }
                println!("mihi {}", VERSION);
                std::process::exit(0);
            }
            "init" => {
                let rest: Vec<String> = args.collect();
                return init::run(rest);
            }
            "nuke" => {
                let rest: Vec<String> = args.collect();
                return nuke::run(rest);
            }
            "words" => {
                let rest: Vec<String> = args.collect();
                return words::run(rest);
            }
            "run" => {
                let rest: Vec<String> = args.collect();
                return run::run(rest);
            }
            _ => {
                println!("error: unknown flag or command: '{}'", arg.as_str());
                std::process::exit(1);
            }
        }
    }

    run::run(Vec::new());
}
