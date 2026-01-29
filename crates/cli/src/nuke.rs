fn help() {
    println!("mihi nuke: Nuke the current installation.\n");
    println!("usage: mihi nuke [OPTIONS]\n");

    println!("Options:");
    println!("   -h, --help\t\tPrint this message.");
}

pub fn run(args: Vec<String>) {
    if let Some(arg) = args.into_iter().next() {
        match arg.as_str() {
            "-h" | "--help" => {
                help();
                std::process::exit(0);
            }
            _ => {
                println!("error: nuke: unknown flag: '{}'", arg.as_str());
                std::process::exit(1);
            }
        }
    }

    match mihi::cfg::get_config_path() {
        Ok(path) => match std::fs::remove_dir_all(path) {
            Ok(_) => {}
            Err(e) => {
                println!("error: nuke: {e}");
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("error: nuke: {e}");
            std::process::exit(1);
        }
    }
}
