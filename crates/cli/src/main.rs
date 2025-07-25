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
    let nargs = args.len();

    // Skip command name.
    args.next();

    match args.next() {
        Some(command_flag) => {
            match command_flag.as_str() {
                "-h" | "--help" => {
                    if nargs > 2 {
                        println!("warning: arguments passed the 'help' flag will be ignored.\n");
                    }
                    help();
                    std::process::exit(0);
                },
                "-v" | "--version" => {
                    if nargs > 2 {
                        println!("warning: arguments passed the 'version' flag will be ignored.\n");
                    }
                    println!("mihi {VERSION}");
                    std::process::exit(0);
                },
                "init" => {
                    let rest: Vec<String> = args.collect();
                    init::run(rest);
                },
                "nuke" => {
                    let rest: Vec<String> = args.collect();
                    nuke::run(rest);
                },
                "words" => {
                    let rest: Vec<String> = args.collect();
                    words::run(rest);
                },
                "run" => {
                    let rest: Vec<String> = args.collect();
                    run::run(rest);
                },
                _ => {
                    println!("error: unknown flag or command: '{command_flag}'");
                    std::process::exit(1);
                }
            }
        },
        None => run::run(Vec::new())
    }
}
