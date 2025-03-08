use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn check_file_exists<S: AsRef<str>>(file: S, overwrite: bool) {
    let file = file.as_ref();
    if !overwrite && Path::new(file).exists() {
        panic!("File at {} already exists, and overwrite is set to false", file);
    }
}

pub fn check_valid_file<S: AsRef<str>>(path: S) {
    let path = path.as_ref();
    if !Path::new(path).is_file() {
        panic!("Path at {} is not a valid file!", path)
    }
}

pub fn write_to_file<S: AsRef<str>>(output_file: S, overwrite: bool, ascii: &[Vec<String>]) {
    let output_file = output_file.as_ref();
    check_file_exists(output_file, overwrite);
    match std::fs::write(
        output_file,
        ascii
            .iter()
            .map(|row| row.join(""))
            .collect::<Vec<String>>()
            .join("\n"),
    ) {
        Ok(_) => {
            println!("Successfully saved ascii art to {}", output_file);
        }
        Err(e) => {
            eprintln!("Failed to save ascii art to {}: {}", output_file, e);
        }
    }
}
