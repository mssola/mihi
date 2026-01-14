use inquire::{Confirm, Editor, Select, Text};
use mihi::{Exercise, ExerciseKind};
use std::vec::IntoIter;

// Show the help message.
fn help(msg: Option<&str>) {
    if let Some(msg) = msg {
        println!("{}.\n", msg);
    }

    println!("mihi exercises: Manage exercises.\n");
    println!("usage: mihi exercises [OPTIONS] <subcommand>\n");

    println!("Options:");
    println!("   -h, --help\t\tPrint this message.");

    println!("\nSubcommands:");
    println!("   create\t\tCreate a new exercise.");
    println!("   edit\t\t\tEdit information from an exercise.");
    println!("   ls\t\t\tList exercises from the database.");
    println!("   rm\t\t\tRemove an exercises from the database.");
}

// Interactively ask the user to fill up an exercise based on the given
// `exercise` object.
fn ask_for_exercise_based_on(exercise: Exercise) -> Result<Exercise, String> {
    let Ok(title) = Text::new("Title:")
        .with_initial_value(&exercise.title)
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    if title.trim().is_empty() {
        return Err("the title is required".to_string());
    }

    let kinds = vec![
        ExerciseKind::Pensum,
        ExerciseKind::Translation,
        ExerciseKind::Transformation,
        ExerciseKind::Numerical,
    ];
    let Ok(kind) = Select::new("Kind:", kinds)
        .with_starting_cursor(exercise.kind as usize)
        .prompt()
    else {
        return Err("abort!".to_string());
    };

    let Ok(enunciate) = Editor::new("Enunciate:")
        .with_predefined_text(&exercise.enunciate)
        .with_file_extension(".md")
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let enunciate = enunciate.trim().to_string();
    if enunciate.is_empty() {
        return Err("the enunciate is required".to_string());
    }

    let Ok(solution) = Editor::new("Solution:")
        .with_predefined_text(&exercise.solution)
        .with_file_extension(".md")
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let solution = solution.trim().to_string();
    if solution.trim().is_empty() {
        return Err("the solution is required".to_string());
    }

    let Ok(lessons) = Editor::new("Lessons:")
        .with_predefined_text(&exercise.lessons)
        .with_file_extension(".md")
        .prompt()
    else {
        return Err("abort!".to_string());
    };
    let lessons = lessons.trim().to_string();

    Ok(Exercise {
        id: exercise.id,
        title,
        enunciate,
        solution,
        lessons,
        kind,
    })
}

fn create(args: IntoIter<String>) -> i32 {
    if args.len() > 0 {
        help(Some(
            "error: exercises: no arguments were expected for this command",
        ));
        return 1;
    }

    let exercise = match ask_for_exercise_based_on(Exercise::default()) {
        Ok(ex) => ex,
        Err(e) => {
            println!("error: exercises: {e}");
            return 1;
        }
    };

    let title = exercise.title.clone();
    match mihi::create_exercise(exercise) {
        Ok(_) => {
            println!("Exercise '{title}' has been successfully created!");
            0
        }
        Err(e) => {
            println!("error: exercises: {e}");
            1
        }
    }
}

fn select_single_exercise(search: Option<String>) -> Result<Exercise, String> {
    let exercises = mihi::select_by_title(search)?;

    let title = match exercises.len() {
        0 => return Err("not found".to_string()),
        1 => exercises.first().unwrap().to_owned(),
        _ => match Select::new("Which exercise?", exercises)
            .with_page_size(20)
            .prompt()
        {
            Ok(choice) => choice,
            Err(_) => return Err("abort!".to_string()),
        },
    };

    mihi::find_exercise_by_title(title.as_str())
}

fn edit(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some("error: exercises: too many filters"));
        return 1;
    }

    let exercise = match select_single_exercise(args.next()) {
        Ok(exercise) => exercise,
        Err(e) => {
            println!("error: exercises: {e}");
            return 1;
        }
    };

    let exercise = match ask_for_exercise_based_on(exercise) {
        Ok(ex) => ex,
        Err(e) => {
            println!("error: exercises: {e}");
            return 1;
        }
    };

    let title = exercise.title.clone();
    match mihi::update_exercise(exercise) {
        Ok(_) => {
            println!("Exercise '{title}' has been successfully updated!");
            0
        }
        Err(e) => {
            println!("error: exercises: {e}");
            1
        }
    }
}

fn ls(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some("error: exercises: too many filters"));
        return 1;
    }

    let exercises = mihi::select_by_title(args.next()).unwrap_or(vec![]);
    for exe in exercises {
        println!("- '{}'", exe);
    }

    0
}

fn rm(mut args: IntoIter<String>) -> i32 {
    if args.len() > 1 {
        help(Some("error: exercises: too many filters"));
        return 1;
    }

    let exercise = match select_single_exercise(args.next()) {
        Ok(exercise) => exercise,
        Err(e) => {
            println!("error: words: {e}");
            return 1;
        }
    };
    let selection = exercise.title.as_str();

    let ans = Confirm::new(
        format!("Do you really want to remove '{selection}' from the database?",).as_str(),
    )
    .with_default(false)
    .prompt();

    match ans {
        Ok(true) => match mihi::delete_exercise(selection) {
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
            "error: exercises: you have to provide at least a subcommand",
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
            _ => {
                help(Some(
                    format!("error: exercises: unknown flag or command '{first}'").as_str(),
                ));
                std::process::exit(1);
            }
        },
        None => {
            help(Some(
                "error: exercises: you need to provide a command"
                    .to_string()
                    .as_str(),
            ));
            std::process::exit(1);
        }
    }
}
