use clap::{App, AppSettings::ArgRequiredElseHelp, Arg, SubCommand};
use std::io::{BufRead, BufReader, Error, Write};
use std::{collections::HashMap, fs::File};
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
        println!("Generating EntitySet.cs[...]");
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

        // Look and classify !gigas & !alt on files

        let mut gigas: Vec<DirEntry> = Vec::new();
        let mut gigas_alt: Vec<DirEntry> = Vec::new();
        for entry in &csharp_files {
            let input = File::open(entry.path()).unwrap();
            let buffered = BufReader::new(input);
            for line in buffered.lines() {
                let line = line.unwrap();
                if line.to_lowercase().contains("!gigas") {
                    if line.to_lowercase().contains("!alt") {
                        gigas_alt.push(entry.clone());
                    } else {
                        gigas.push(entry.clone());
                    }
                    break;
                }
            }
        }

        // Debug printing

        // println!("\nGIGAS");
        // for entry in &gigas {
        //     println!("{}", entry.file_name().to_string_lossy());
        // }

        // println!("\nALT");
        // for entry in &gigas_alt {
        //     println!("{}", entry.path().file_stem().unwrap().to_string_lossy());
        // }

        // Find relationships between classes

        let gigas_classes: Vec<&str> = gigas
            .iter()
            .map(|x| x.path().file_stem().unwrap().to_str().unwrap())
            .collect();

        let mut relation: HashMap<String, Vec<String>> = HashMap::new();

        for file in &csharp_files {
            for line in lines_from_file(file.path()) {
                for class in &gigas_classes {
                    let filename = file.path().file_stem().unwrap().to_str().unwrap();

                    if filename == "Femto" || filename == "EntitySet" {
                        continue;
                    }

                    let gigasnames: Vec<String> =
                        vec![format!("{}s", class), format!("{}Ids", class)];

                    for name in gigasnames {
                        if line.contains(&name) {
                            // print!("\nFound {} in {} at \n{}\n", class, filename, line);

                            let classes = relation
                                .entry(filename.to_string())
                                .or_insert_with(Vec::new);

                            if !classes.contains(&class.to_string()) {
                                classes.push(class.to_string());
                            }
                        }
                    }
                }
            }
        }

        for (key, value) in &relation {
            println!("\n{}", key);
            for entry in value {
                println!("\t{}", entry);
            }
        }
    }
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("No file name.");
    let buffer = BufReader::new(file);
    buffer
        .lines()
        .map(|l| l.expect("Couldn't parse line."))
        .collect()
}
