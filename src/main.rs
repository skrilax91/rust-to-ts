mod constants;
mod enums;

use crate::constants::process_constants;
use crate::enums::process_enums;
use serde_json::Value;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value as TomlValue;

/// Get all files in a directory
/// This function will get all files in a directory and its subdirectories
/// and return a vector of PathBuf
///
/// # Arguments
/// * `path` - A PathBuf object that represents the directory
/// * `ext` - A string that represents the file extension
///
/// # Returns
/// * A vector of PathBuf objects that represents the files in the directory
fn get_all_files(path: PathBuf, ext: Option<String>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let paths = std::fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            files.append(&mut get_all_files(path, ext.clone()));
        } else if let Some(ext) = &ext {
            if path.extension().unwrap_or_default().to_str().unwrap() == ext {
                files.push(path);
            }
        } else {
            files.push(path);
        }
    }
    files
}

async fn execute_ts_rs(path: PathBuf, export_path: PathBuf) {
    let status = Command::new("cargo")
        .current_dir(path)
        .arg("test")
        .arg("--all-features")
        .env("TS_RS_EXPORT_DIR", export_path)
        .status()
        .expect("Failed to execute cargo test");

    if !status.success() {
        panic!("Failed to execute cargo test");
    }
}

async fn build_ts_project(path: PathBuf) {
    let status = Command::new("npm")
        .current_dir(path.clone())
        .arg("i")
        .status()
        .expect("Failed to execute npm install");

    if !status.success() {
        panic!("Failed to execute npm install");
    }

    let status = Command::new("npx")
        .current_dir(path)
        .arg("tsc")
        .status()
        .expect("Failed to execute tsc");

    if !status.success() {
        panic!("Failed to execute tsc");
    }
}

fn generate_index(path: PathBuf) {
    let mut file = std::fs::File::create(path.join("index.ts")).unwrap();
    let mut buffer = Vec::new();

    writeln!(buffer, "// Auto-generated index file\n").unwrap();
    writeln!(buffer, "// Export all the constants").unwrap();
    writeln!(buffer, "export * from './const';\n").unwrap();

    let structs = get_all_files(path.join("generated-structs"), Some("ts".to_string()));

    for struct_file in structs {
        // Remove the path before the "generated-structs" directory
        let struct_file = struct_file
            .strip_prefix(path.clone())
            .unwrap()
            .to_str()
            .unwrap();
        //remove .ts
        let struct_file = struct_file.strip_suffix(".ts").unwrap();
        writeln!(buffer, "export * from './{}';", struct_file).unwrap();
    }

    file.write_all(&buffer).unwrap();
}

fn sync_project_version(rust_path: PathBuf, ts_path: PathBuf) {
    let rust_toml =
        std::fs::read_to_string(rust_path.join("Cargo.toml")).expect("Failed to read Cargo.toml");
    let cargo_toml: TomlValue = rust_toml.parse::<TomlValue>().unwrap();

    // Récupérer la version du projet Rust
    let rust_version = cargo_toml
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(|ver| ver.as_str())
        .ok_or("Version not found in Cargo.toml")
        .unwrap();

    let package_json_content =
        std::fs::read_to_string(ts_path.join("package.json")).expect("Failed to read package.json");
    let mut package_json: Value = serde_json::from_str(&package_json_content).unwrap();

    if let Some(obj) = package_json.as_object_mut() {
        obj.insert(
            "version".to_string(),
            Value::String(rust_version.to_string()),
        );
    }

    let updated_package_json = serde_json::to_string_pretty(&package_json).unwrap();
    let mut file = std::fs::File::create(ts_path.join("package.json")).unwrap();
    file.write_all(updated_package_json.as_bytes()).unwrap();

    println!("Version updated to {}", rust_version);
}

/// Process optional types
/// This function will process all optional types and change | undefined to ? in the typescript files
///
/// Example:
///   let a: string | undefined; -> let a?: string;
fn process_optionals(path: PathBuf) {
    let files = get_all_files(path, Some("ts".to_string()));

    for file in files {
        let file_path = file.to_str().unwrap();
        let mut file = std::fs::read_to_string(file_path).unwrap();
        let mut mutated_file = file.clone();

        // get all export types with regex
        let regex = regex::Regex::new(r"([A-Za-z0-9]+):\s*([A-Za-z0-9]+\s*\|\s*undefined)").unwrap();

        let result = regex.captures_iter(&file);

        for cap in result {
            let name = cap.get(1).unwrap().as_str();
            let mut value = cap.get(2).unwrap().as_str();

            // replace name\s*:\s*value | undefined with name?: value
            let new_line = format!("{}?: {}", name, value.replace(" | undefined", ""));
            mutated_file = mutated_file.replace(cap.get(0).unwrap().as_str(), &new_line);
        }

        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(mutated_file.as_bytes()).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Please provide a rust project file path as an argument and a typescript project file path as an argument");
        return;
    }

    if args[1] == "--help" {
        println!("Usage: cargo run <path_to_rust_project> <path_to_ts_project>");
        return;
    }

    let rust_full_path = Path::new(&args[1]).canonicalize().unwrap();
    let rust_src_path = rust_full_path.join("src");

    let ts_full_path = Path::new(&args[2]).canonicalize().unwrap();
    let ts_src_path = ts_full_path.join("src");

    println!("Rust project path: {:?}", rust_full_path);
    println!("Typescript project path: {:?}", ts_full_path);

    // remove the generated-structs directory
    let generated_structs = ts_src_path.join("generated-structs");
    if generated_structs.exists() {
        std::fs::remove_dir_all(generated_structs).unwrap();
    }

    execute_ts_rs(
        rust_full_path.clone(),
        ts_src_path.join("generated-structs"),
    )
    .await;
    process_enums(ts_src_path.join("generated-structs"));
    process_constants(rust_src_path, ts_src_path.clone());
    process_optionals(ts_src_path.clone());
    generate_index(ts_src_path);

    sync_project_version(rust_full_path, ts_full_path.clone());

    build_ts_project(ts_full_path).await;

    println!("Done!");
}
