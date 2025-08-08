use winreg::enums::*;
use winreg::RegKey;
use std::io::Result;

pub fn set_value(path: &str, key: &str, val: &str) -> Result<()> {
    let hkcu: RegKey = RegKey::predef(HKEY_CURRENT_USER);
    let (subkey, _) = hkcu.create_subkey(path)?;

    subkey.set_value(key, &val)?;
    Ok(())
}

pub fn delete_key(path: &str) -> Result<()> {
    let hkcu: RegKey = RegKey::predef(HKEY_CURRENT_USER);

    hkcu.delete_subkey_all(path)?;
    Ok(())
}