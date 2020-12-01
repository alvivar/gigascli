use clap::{App, AppSettings::ArgRequiredElseHelp, Arg, SubCommand};
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::{env, path::Path};
use walkdir::{DirEntry, WalkDir};

fn main() {
    // Command line options

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

    // Templates

    if let Some(matches) = matches.subcommand_matches("new") {
        if matches.is_present("alt") {
            println!("!Alt enabled");
        } else {
            println!("Simple component");
        }

        println!("File name: {}", matches.value_of("ComponentName").unwrap());
    }

    // Code Generation

    if let Some(_matches) = matches.subcommand_matches("generate") {
        println!("Generating EntitySet.cs[...]")
    }

    // Code Analizis

    if let Some(_matches) = matches.subcommand_matches("analize") {
        let current_dir = env::current_dir().unwrap();

        // Find all C# files

        let mut csharp_files: Vec<DirEntry> = Vec::new();
        for entry in WalkDir::new(current_dir) {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.ends_with(".cs") {
                csharp_files.push(entry);
            }
        }

        // Looking for !gigas and collecting classes

        let mut gigas_classes: Vec<DirEntry> = Vec::new();
        let mut gigas_alt_classes: Vec<DirEntry> = Vec::new();
        for entry in csharp_files {
            let input = File::open(entry.path()).unwrap();
            let buffered = BufReader::new(input);
            for line in buffered.lines() {
                let line = line.unwrap();
                if line.to_lowercase().contains("!gigas") {
                    if line.to_lowercase().contains("!alt") {
                        gigas_alt_classes.push(entry.clone());
                    } else {
                        gigas_classes.push(entry.clone());
                    }
                    break;
                }
            }
        }

        //

        println!("Normal classes:");
        for entry in gigas_classes {
            println!("{}", entry.file_name().to_string_lossy());
        }

        println!("\nAlt classes:");
        for entry in gigas_alt_classes {
            println!("{}", entry.file_name().to_string_lossy());
        }
    }
}
