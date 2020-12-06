use clap::{App, AppSettings::ArgRequiredElseHelp, Arg, SubCommand};
use std::io::{BufRead, BufReader};
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

        // All C# files

        let csharp_files = find_files(current_dir, ".cs");

        // Look and classify !gigas & !alt on files

        let mut gigas: Vec<DirEntry> = Vec::new();
        let mut gigas_alt: Vec<DirEntry> = Vec::new();

        for entry in &csharp_files {
            for line in lines_from_file(entry.path()) {
                let lowercase_line = line.to_lowercase();

                if lowercase_line.contains("!gigas") {
                    if lowercase_line.contains("!alt") {
                        gigas_alt.push(entry.clone());
                    } else {
                        gigas.push(entry.clone());
                    }

                    break;
                }
            }
        }

        // Find relationships between classes

        let mut gigas_all: Vec<DirEntry> = Vec::new();
        gigas_all.extend(gigas);
        gigas_all.extend(gigas_alt);

        let gigas_classes: Vec<String> = gigas_all
            .iter()
            .map(|x| x.path().file_stem().unwrap().to_string_lossy().to_string())
            .collect();

        let mut relation_fileclass: HashMap<String, Vec<String>> = HashMap::new();
        let mut relation_classfile: HashMap<String, Vec<String>> = HashMap::new();

        for file in &csharp_files {
            for line in lines_from_file(file.path()) {
                for class in &gigas_classes {
                    let filename = file.path().file_stem().unwrap().to_str().unwrap();

                    if filename == "Femto" || filename == "EntitySet" {
                        continue;
                    }

                    let gigasnames: Vec<String> = vec![
                        format!("Get{}", class),
                        format!("{}s", class),
                        format!("{}Ids", class),
                    ];

                    for name in gigasnames {
                        if line.contains(&name) {
                            // File to Class

                            let classes = relation_fileclass
                                .entry(filename.to_string())
                                .or_insert_with(Vec::new);

                            if !classes.contains(&class.to_string()) {
                                classes.push(class.to_string());
                            }

                            // Class to File

                            let files = relation_classfile
                                .entry(class.to_string())
                                .or_insert_with(Vec::new);

                            if !files.contains(&filename.to_string()) {
                                files.push(filename.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Print relationships

        println!("\n\t\t\t# From System to Component");
        for (key, value) in &relation_fileclass {
            println!("\n{}", key);
            for entry in value {
                println!("\t{}", entry);
            }
        }

        println!("\n\t\t\t# From Component to System");
        for (key, value) in &relation_classfile {
            println!("\n{}", key);
            for entry in value {
                println!("\t{}", entry);
            }
        }
    }
}

fn find_files(filepath: impl AsRef<Path>, extension: &str) -> Vec<DirEntry> {
    let mut files: Vec<DirEntry> = Vec::new();
    for entry in WalkDir::new(filepath) {
        let entry = entry.expect("Couldn't walk the path.");
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.ends_with(extension) {
            files.push(entry);
        }
    }

    files
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("Couldn't open the file.");
    let buffer = BufReader::new(file);

    buffer
        .lines()
        .map(|l| l.expect("Couldn't parse the line."))
        .collect()
}
