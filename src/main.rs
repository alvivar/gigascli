use clap::{App, AppSettings::ArgRequiredElseHelp, Arg, SubCommand};
use std::env;
use walkdir::WalkDir;

fn main() {
    let matches = App::new("gigas-cli")
        .version("0.1")
        .about("Check out github.com/alvivar/Gigas for more info!")
        .setting(ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("new")
                .about("Creates a Gigas Component and his System.")
                .arg(
                    Arg::with_name("alt")
                        .long("alt")
                        .help("Includes the !Alt API."),
                )
                .arg(
                    Arg::with_name("ComponentName")
                        .help("File name to be used as Component.")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("generate").about("Creates (or updates) the EntitySet.cs."),
        )
        .subcommand(
            SubCommand::with_name("analize").about("Analizes .cs files looking for relationships."),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        if matches.is_present("alt") {
            println!("!Alt enabled");
        } else {
            println!("Simple component");
        }

        println!("File name: {}", matches.value_of("ComponentName").unwrap());
    }

    if let Some(_matches) = matches.subcommand_matches("generate") {
        println!("Generating EntitySet.cs[...]")
    }

    if let Some(_matches) = matches.subcommand_matches("analize") {
        let current_dir = env::current_dir().unwrap();
        for entry in WalkDir::new(current_dir) {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.ends_with(".cs") {
                println!("{} at {}", name, entry.path().display());
            }
        }
    }
}
