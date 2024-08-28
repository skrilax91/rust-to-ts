use crate::get_all_files;
use regex::Regex;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub value: String,
}

pub fn find_all_public_constants(rust_src_path: PathBuf) -> Vec<Constant> {
    let files = get_all_files(rust_src_path, Some("rs".to_string()));
    let mut constants = Vec::new();
    let regex = Regex::new(r#"\s*pub const ([A-Z_0-9]+): &str =\s*".+";"#).unwrap();

    for file in files {
        let file_path = file.to_str().unwrap();
        let file = std::fs::read_to_string(file_path).unwrap();

        let mut multiline_buffer = String::new();

        for line in file.lines() {
            multiline_buffer.push_str(line);


            if !multiline_buffer.trim().trim_matches('\t').starts_with("pub const") {
                multiline_buffer.clear();
                continue;
            }

            if !multiline_buffer.ends_with(";") {
                continue;
            }

            multiline_buffer = multiline_buffer.trim().to_string();

            if regex.is_match(multiline_buffer.as_str()) {
                let name = multiline_buffer
                    .split_whitespace()
                    .nth(2)
                    .unwrap()
                    .trim_matches(':')
                    .to_string();
                let value = multiline_buffer
                    .split_whitespace()
                    .last()
                    .unwrap()
                    .to_string();
                let value = value.trim_matches(';').trim_matches('"').to_string();

                constants.push(Constant { name, value });
            }

            multiline_buffer.clear();
        }
    }

    constants
}

pub fn write_constants(constants: Vec<Constant>, path: PathBuf) {
    let mut file = std::fs::File::create(path).unwrap();

    let mut buffer = Vec::new();

    for constant in constants {
        writeln!(
            buffer,
            "export const {}: string = \"{}\";",
            constant.name, constant.value
        )
        .unwrap();
    }

    file.write_all(&buffer).unwrap();
}

pub fn process_constants(rust_src_path: PathBuf, ts_dest_path: PathBuf) {
    println!(
        "Processing constants in: {}",
        rust_src_path.to_str().unwrap()
    );
    let constants = find_all_public_constants(rust_src_path);
    println!("Found {} constants", constants.len());
    write_constants(constants, ts_dest_path.join("const.ts"));
}
