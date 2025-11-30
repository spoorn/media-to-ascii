use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn check_file_exists<S: AsRef<str>>(file: S, overwrite: bool) -> Result<(), String> {
    let file = file.as_ref();
    if !overwrite && Path::new(file).exists() {
        return Err(format!("File at {} already exists, and overwrite is set to false", file));
    }
    Ok(())
}

pub fn check_valid_file<S: AsRef<str>>(path: S) -> Result<(), String> {
    let path = path.as_ref();
    if !Path::new(path).is_file() {
        return Err(format!("Path at {} is not a valid file!", path));
    }
    Ok(())
}

pub fn write_to_file<S: AsRef<str>>(output_file: S, overwrite: bool, ascii: &[Vec<&str>]) -> Result<(), String> {
    let output_file = output_file.as_ref();
    check_file_exists(output_file, overwrite)?;

    // TODO: change to create_new
    let file_option = OpenOptions::new().write(true).create(true).truncate(true).open(output_file);

    match file_option {
        Ok(mut file) => {
            for row in ascii {
                file.write_all(row.join("").as_bytes()).unwrap();
                file.write_all("\r\n".as_bytes()).unwrap();
            }
            Ok(())
        }
        Err(_) => Err(format!("Could not write output to file {}", output_file)),
    }
}
