use std::env;
use std::path::PathBuf;
use std::path::Path;
use std::iter::once;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

use windows::core::{Error, PCWSTR, PWSTR, ComInterface, Result as WinRes};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use windows::Win32::System::WindowsProgramming::GetUserNameW;

use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};

use windows::Win32::Storage::FileSystem::SetFileAttributesW;
use windows::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_SYSTEM};

use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::System::Com::{
    IPersistFile,
    CoCreateInstance, CoInitializeEx,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED
};

pub fn is_admin() -> bool {
    unsafe {
        let mut handle: HANDLE = HANDLE::default();

        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY,
            &mut handle
        ).is_err() {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size: u32 = 0u32;

        let result: Result<(), Error> = GetTokenInformation(
            handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );

        let _ = CloseHandle(handle);

        result.is_ok() && elevation.TokenIsElevated != 0
    }
}

pub fn get_name() -> Option<String> {
    let mut buffer: [u16; 256] = [0u16; 256];
    let mut size: u32 = buffer.len() as u32;

    let res: Result<(), Error> = unsafe {
        GetUserNameW(
            PWSTR(buffer.as_mut_ptr()),
            &mut size
        )
    };

    if res.is_ok() {
        let name: String = OsString::from_wide(
            &buffer[..(size as usize - 1)]
        )
            .to_string_lossy()
            .into_owned();

        Some(name)
    } else {
        None
    }
}

pub fn create_link(name: &str, target: &str, out: &str, icon: &str) -> WinRes<PathBuf> {
    unsafe {
        CoInitializeEx(
            None,
            COINIT_APARTMENTTHREADED
        )?;

        let link: IShellLinkW = CoCreateInstance(
            &ShellLink,
            None,
            CLSCTX_INPROC_SERVER
        )?;

        let target: Vec<u16> = target
            .encode_utf16()
            .chain(once(0))
            .collect();

        let icon: Vec<u16> = icon
            .encode_utf16()
            .chain(once(0))
            .collect();

        link.SetPath(PCWSTR(target.as_ptr()))?;
        link.SetIconLocation(PCWSTR(icon.as_ptr()), 0)?;

        let file: IPersistFile = link.cast()?;

        let startup: PathBuf = {
            let appdata: String = env::var("APPDATA")
                .map_err(|_| Error::from_win32())?;

            let mut path: PathBuf = PathBuf::from(appdata);
            path.push(out);
            path.push(format!("{}.lnk", name));
            path
        };

        let startup_w: Vec<u16> = startup
            .as_os_str()
            .encode_wide()
            .chain(once(0))
            .collect();

        file.Save(PCWSTR(startup_w.as_ptr()), true)?;

        Ok(startup)
    }
}

pub fn message_box(text: &str, caption: &str) {
    unsafe {
        let text: Vec<u16> = text
            .encode_utf16()
            .chain(once(0))
            .collect();

        let caption: Vec<u16> = caption
            .encode_utf16()
            .chain(once(0))
            .collect();

        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            PCWSTR(caption.as_ptr()),
            MB_OK
        );
    }
}

pub fn hide_item(path: &Path) -> WinRes<()> {
    unsafe {
        let path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(once(0))
            .collect();

        SetFileAttributesW(
            PCWSTR(path.as_ptr()),
            FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM
        )?;
    }

    Ok(())
}