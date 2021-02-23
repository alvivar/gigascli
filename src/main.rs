use clap::{App, AppSettings::ArgRequiredElseHelp, Arg, SubCommand};
use core::panic;
use self_update::cargo_crate_version;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};
use walkdir::{DirEntry, WalkDir};

fn main() {
    // Command Line
    let matches = App::new("gigas-cli")
        .version(cargo_crate_version!())
        .about("Check out github.com/alvivar/Gigas for more info!")
        .setting(ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("new")
                .setting(ArgRequiredElseHelp)
                .about("Creates a Gigas Component System")
                .arg(
                    Arg::with_name("Name")
                        .help("File name to be used as Component System")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("alt")
                        .long("alt")
                        .help("Includes the !Alt API"),
                )
                .arg(Arg::with_name("output").short("o").help("Create the files"))
                .arg(
                    Arg::with_name("nocomp")
                        .long("nocomp")
                        .help("Ignore the Component"),
                )
                .arg(
                    Arg::with_name("nosys")
                        .long("nosys")
                        .help("Ignore the System"),
                ),
        )
        .subcommand(
            SubCommand::with_name("analize").about("Analizes .cs files looking for relationships"),
        )
        .subcommand(
            SubCommand::with_name("update").about("Self updates to the latest release on Github"),
        )
        .get_matches();

    // Component and System templates
    if let Some(matches) = matches.subcommand_matches("new") {
        let component_name = matches.value_of("Name").unwrap();
        let has_output = matches.is_present("output");
        let has_nocomp = matches.is_present("nocomp");
        let has_nosys = matches.is_present("nosys");

        // Templates
        let component = match matches.is_present("alt") {
            true => generate_alt_component_string(component_name),
            false => generate_component_string(component_name),
        };

        let system = generate_system_string(component_name);

        // Writing
        let component_file = format!("{}.cs", component_name);
        let system_file = format!("{}System.cs", component_name);

        if has_output {
            let current_dir = env::current_dir().unwrap();

            if !has_nocomp {
                write_file(component.as_str(), current_dir.join(&component_file));
            }

            if !has_nosys {
                write_file(system.as_str(), current_dir.join(&system_file));
            }
        }

        // Print
        if has_output {
            if !has_nocomp || !has_nosys {
                println!();
            }

            if !has_nocomp {
                println!("- {} generated", component_file);
            }

            if !has_nosys {
                println!("- {} generated", system_file);
            }
        }

        println!("\nDone!");
    }

    // Code analysis
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

        let gigas_components: Vec<String> = gigas_all
            .iter()
            .map(|x| x.path().file_stem().unwrap().to_string_lossy().to_string())
            .collect();

        let mut relation_system_component: HashMap<String, Vec<String>> = HashMap::new();
        let mut relation_component_system: HashMap<String, Vec<String>> = HashMap::new();

        for file in &csharp_files {
            for line in lines_from_file(file.path()) {
                for class in &gigas_components {
                    let system = file.path().file_stem().unwrap().to_str().unwrap();

                    if system == "Femto" || system == "EntitySet" {
                        continue;
                    }

                    let gigasnames: Vec<String> = vec![
                        format!(".{}s", class),
                        format!(".{}Ids", class),
                        format!("Get{}(", class),
                        format!("GetAlt{}(", class),
                    ];

                    for name in gigasnames {
                        if line.contains(&name) {
                            // System to Component
                            let components = relation_system_component
                                .entry(system.to_string())
                                .or_insert_with(Vec::new);

                            if !components.contains(&class.to_string()) {
                                components.push(class.to_string());
                            }

                            // Component to System
                            let systems = relation_component_system
                                .entry(class.to_string())
                                .or_insert_with(Vec::new);

                            if !systems.contains(&system.to_string()) {
                                systems.push(system.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Relationships
        println!();
        println!("\t------ - --------- ------------");
        println!("\tSystem + Component Relationship");
        println!("\t------ - --------- ------------");
        for (key, value) in &relation_system_component {
            println!("\n\t{}", key);
            for entry in value {
                println!("\t\t{}", entry);
            }
        }

        println!();
        println!("\t--------- - ------ ------------");
        println!("\tComponent + System Relationship");
        println!("\t--------- - ------ ------------");
        for (key, value) in &relation_component_system {
            println!("\n\t{}", key);
            for entry in value {
                println!("\t\t{}", entry);
            }
        }

        println!("\nDone!");
    }

    // Self Update
    if let Some(_matches) = matches.subcommand_matches("update") {
        println!();

        match update() {
            Ok(_) => {}
            Err(_) => {
                panic!("Error updating.")
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
    let file = File::open(filename).expect("I can't open the file.");
    let buffer = BufReader::new(file);

    buffer
        .lines()
        .map(|l| l.expect("I can't parse the line."))
        .collect()
}

fn lowercase_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

fn write_file(data: &str, filepath: impl AsRef<Path>) {
    let f = File::create(filepath).expect("I can't create the file.");
    let mut f = BufWriter::new(f);
    f.write_all(data.as_bytes())
        .expect("I can't write to the file.");
}

fn generate_component_string(name: &str) -> String {
    let template = r#"

using UnityEngine;

// !Gigas
public class @ComponentName : MonoBehaviour
{
    private void OnEnable()
    {
        EntitySet.Add@ComponentName(this);
    }

    private void OnDisable()
    {
        EntitySet.Remove@ComponentName(this);
    }
}

"#;

    template.replace("@ComponentName", name).trim().to_string()
}

fn generate_alt_component_string(name: &str) -> String {
    let template = r#"

using UnityEngine;

// !Gigas !Alt
public class @ComponentName : MonoBehaviour
{
    private void Awake()
    {
        EntitySet.AddAlt@ComponentName(this);
    }

    private void OnDestroy()
    {
        EntitySet.RemoveAlt@ComponentName(this);
    }

    private void OnEnable()
    {
        EntitySet.Add@ComponentName(this);
    }

    private void OnDisable()
    {
        EntitySet.Remove@ComponentName(this);
    }
}

"#;

    template.replace("@ComponentName", name).trim().to_string()
}

fn generate_system_string(name: &str) -> String {
    let template = r#"

using UnityEngine;

public class @ComponentSystem : MonoBehaviour
{
    void Update()
    {
        var @components = EntitySet.@Components;
        for (int i = 0; i < @components.Length; i++)
        {
            var @component = @components.Elements[i];
        }
    }
}

"#;

    template
        .replace("@Component", name)
        .replace("@component", lowercase_first(name).as_str())
        .trim()
        .to_string()
}

fn update() -> Result<(), Box<dyn std::error::Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("alvivar")
        .repo_name("gigascli")
        .bin_name("gigascli")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    println!("Update status: v{}", status.version());

    Ok(())
}
