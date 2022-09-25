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

pub fn write_to_file<S: AsRef<str>>(output_file: S, overwrite: bool, ascii: &[Vec<&str>]) {
    let output_file = output_file.as_ref();
    check_file_exists(output_file, overwrite);

    // TODO: change to create_new
    let file_option = OpenOptions::new().write(true).create(true).truncate(true).open(output_file);

    match file_option {
        Ok(mut file) => {
            for row in ascii {
                file.write_all(row.join("").as_bytes()).unwrap();
                file.write_all("\r\n".as_bytes()).unwrap();
            }
        }
        Err(_) => {
            panic!("Could not write output to file {}", output_file);
        }
    }
}
