use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::error::Error;
use std::path::PathBuf;
use std::borrow::Cow;

use lib::crypto::decrypt;
use lib::hash::hash;

fn read_line(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input: String = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    let ses_id: String = match read_line("session token: ") {
        Ok(res) => {
            if res.len() != 40 {
                println!("invalid session token");
                return Ok(());
            }

            if !res.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
                println!("invalid session token");
                return Ok(());
            }

            res
        },
        Err(err) => {
            println!("failed to read input: {}", err);
            return Ok(());
        }
    };

    let file_str: String = match read_line("file path: ") {
        Ok(res) => res,
        Err(err) => {
            eprintln!("failed to read input: {}", err);
            return Ok(());
        }
    };

    let (path, suffix) = match file_str.rsplit_once(".amen.") {
        Some((_, suffix)) if suffix.len() == 6 && suffix.chars().all(|c| c.is_ascii_alphanumeric()) => {
            let path: PathBuf = PathBuf::from(file_str.clone());
            if !path.exists() {
                println!("file does not exist");
                return Ok(());
            }

            (path, suffix.to_string())
        }
        _ => {
            eprintln!("invalid file path format");
            return Ok(());
        }
    };

    let key: String = format!("{}{}", suffix, ses_id);

    let mut file: File = File::open(&path)?;
    let mut content: String = String::new();
    file.read_to_string(&mut content)?;

    let decrypted: String = match decrypt(&content, &hash(&key)) {
        Ok(res) => res,
        Err(_) => return Ok(())
    };

    let mut new_file: File = File::create(&path)?;
    new_file.write_all(decrypted.as_bytes())?;

    let old_name: Cow<'_, str> = path
        .file_name()
        .unwrap()
        .to_string_lossy();

    let new_name: &str = old_name
        .strip_suffix(&format!(".amen.{}", suffix))
        .unwrap();

    let new_path: PathBuf = path.with_file_name(new_name);
    fs::rename(path, new_path)?;

    Ok(())
}
