use self_update::cargo_crate_version;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

fn main() {
    let current_dir = env::current_dir().unwrap();

    let usage_text = "
  Usage: gigascli.exe [command] [filter]

  Options:
    -h, --help        \tShow this help message

  Commands:
    - analyze [filter]\tScans *.cs files with !Gigas tags and prints relationships between classes
    - update          \tUpdates this tool to the latest version";

    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        println!("{}", usage_text);
        return;
    }

    let command = if args.len() > 1 { args[1].as_str() } else { "" };
    let filter = if args.len() > 2 { args[2].as_str() } else { "" };

    match command {
        "-h" | "--help" => {
            println!("{}", usage_text);
            return;
        }

        "analyze" => {}

        "update" => {
            println!();
            update().unwrap();
            return;
        }

        _ => {
            if !command.is_empty() {
                println!("\n  Unknown command: {}", command);
                println!("{}", usage_text);
                return;
            }
        }
    }

    let (system_component, component_system) = scan_gigas_files(current_dir);
    print_scan(system_component, component_system, filter);

    println!("\n\n  Done!");
}

fn scan_gigas_files(path: PathBuf) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
    let csharp_files: Vec<PathBuf> = find_files(path, ".cs");

    // Look for !Gigas files
    let mut gigas: Vec<PathBuf> = Vec::new();

    for entry in &csharp_files {
        for line in lines_from_file(entry).unwrap() {
            if line.to_lowercase().contains("!gigas") {
                gigas.push(entry.clone());
                break;
            }
        }
    }

    // Find relationships between classes
    let classes: Vec<String> = gigas
        .iter()
        .map(|x| x.file_stem().unwrap().to_string_lossy().to_string())
        .collect();

    let mut system_component: HashMap<String, Vec<String>> = HashMap::new();
    let mut component_system: HashMap<String, Vec<String>> = HashMap::new();

    for file in &csharp_files {
        for line in lines_from_file(file).unwrap() {
            for class in &classes {
                let system = file.file_stem().unwrap().to_str().unwrap();

                if system == "Femto" || system == "EntitySet" {
                    continue;
                }

                let names: Vec<String> = vec![
                    format!("EntitySet.{}Id", class),
                    format!("EntitySet.Get{}(", class),
                    format!("EntitySet.GetAll{}(", class),
                ];

                for name in names {
                    if line.contains(&name) {
                        // System to Component
                        let components = system_component
                            .entry(system.to_string())
                            .or_insert_with(Vec::new);

                        if !components.contains(&class.to_string()) {
                            components.push(class.to_string());
                        }

                        // Component to System
                        let systems = component_system
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

    (system_component, component_system)
}

fn print_scan(
    system_component: HashMap<String, Vec<String>>,
    component_system: HashMap<String, Vec<String>>,
    filter: &str,
) {
    // Relationships
    println!();
    println!("  System/Component Relationship");

    for (key, value) in &system_component {
        if !filter.is_empty() && !key.to_lowercase().contains(filter.to_lowercase().as_str()) {
            continue;
        }

        println!("\n    {}", key);
        for entry in value {
            println!("      - {}", entry);
        }
    }

    println!();
    println!();
    println!("  Component/System Relationship");

    for (key, value) in &component_system {
        if !filter.is_empty() && !key.to_lowercase().contains(filter.to_lowercase().as_str()) {
            continue;
        }

        println!("\n    {}", key);
        for entry in value {
            println!("      - {}", entry);
        }
    }
}

fn find_files(filepath: impl AsRef<Path>, extension: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(filepath) {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy();

        if name.to_lowercase().ends_with(extension) {
            files.push(entry.into_path());
        }
    }

    files
}

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let file = File::open(filename).unwrap();
    let buffer = BufReader::new(file);

    buffer.lines().collect()
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

    println!("Current version... v{}", status.version());

    Ok(())
}
