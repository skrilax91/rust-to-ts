use crate::get_all_files;
use regex::Regex;
use std::io::Write;
use std::path::PathBuf;

pub struct EnumType {
    pub name: String,
    pub path: PathBuf,
    pub values: Vec<String>,
}

/// Find all enum types in the project
/// This function will search all files in the project and find all types that can be converted to enum
pub fn find_all_enum_types(path: PathBuf) -> Vec<EnumType> {
    let mut enum_types = Vec::new();
    let regex =
        Regex::new(r#"export type \w+\s*=\s*[a-zA-Z"0-9\s]+(\|[a-zA-Z"0-9\s]+)+;"#).unwrap();

    // Get all files in the project
    let files = get_all_files(path, Some("ts".to_string()));

    // Iterate over all files
    for file in files {
        let file_path = file.to_str().unwrap();
        let file = std::fs::read_to_string(file_path).unwrap();

        // Iterate over all lines in the file
        for line in file.lines() {
            let line = line.trim();

            // Check if the line is an enum type
            if regex.is_match(line) {
                let name = line.split_whitespace().nth(2).unwrap().to_string();
                let values = line.split_whitespace().skip(4).collect::<Vec<&str>>();
                let values = values
                    .iter()
                    .map(|v| v.trim_matches(';').trim_matches('"').to_string())
                    .filter(|v| !v.is_empty() && v != "|")
                    .collect();

                enum_types.push(EnumType {
                    name,
                    path: PathBuf::from(file_path),
                    values,
                });
            }
        }
    }

    enum_types
}

pub fn write_enum(enum_type: EnumType) {
    let mut file = std::fs::File::create(enum_type.path).unwrap();

    let mut buffer = Vec::new();

    writeln!(buffer, "export enum {} {{", enum_type.name).unwrap();
    for value in &enum_type.values {
        writeln!(buffer, "    {} = \"{}\",", value, value).unwrap();
    }
    writeln!(buffer, "}}").unwrap();

    file.write_all(&buffer).unwrap();
}

pub fn process_enums(ts_src_path: PathBuf) {
    println!("Processing enums in: {}", ts_src_path.to_str().unwrap());
    let enum_types = find_all_enum_types(ts_src_path);

    for enum_type in enum_types {
        println!("Processing enum: {}", enum_type.name);
        write_enum(enum_type);
    }
}
