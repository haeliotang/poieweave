//! API-key storage.
//!
//! The API key is never written to SQLite. Production uses the platform
//! credential store:
//! - macOS: Keychain through the system `security` command;
//! - Windows: Credential Manager through `advapi32` (`CredWriteW`/`CredReadW`);
//! - Linux: Secret Service through the `secret-tool` helper (GNOME Keyring /
//!   KWallet providers).
//! `POIEWEAVE_INFERENCE_API_KEY` is a read-only headless override. Tests inject
//! an in-memory store, so no developer keychain is touched by `cargo test`.

use std::process::Command;

pub trait SecretStore: Send + Sync {
    fn set(&self, service: &str, account: &str, value: &str) -> Result<(), String>;
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, String>;
    fn delete(&self, service: &str, account: &str) -> Result<(), String>;
}

pub struct SystemSecretStore;

fn env_override() -> Option<String> {
    std::env::var("POIEWEAVE_INFERENCE_API_KEY").ok().filter(|value| !value.is_empty())
}

impl SecretStore for SystemSecretStore {
    fn set(&self, service: &str, account: &str, value: &str) -> Result<(), String> {
        if env_override().is_some() {
            return Err("inference key is managed by POIEWEAVE_INFERENCE_API_KEY".into());
        }
        platform_set(service, account, value)
    }

    fn get(&self, service: &str, account: &str) -> Result<Option<String>, String> {
        if let Some(value) = env_override() {
            return Ok(Some(value));
        }
        platform_get(service, account)
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), String> {
        if env_override().is_some() {
            return Err("inference key is managed by POIEWEAVE_INFERENCE_API_KEY".into());
        }
        platform_delete(service, account)
    }
}

// ── macOS ────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn platform_set(service: &str, account: &str, value: &str) -> Result<(), String> {
    let output = Command::new("security")
        .args(["add-generic-password", "-U", "-s", service, "-a", account, "-w", value])
        .output()
        .map_err(|error| format!("macOS Keychain: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("macOS Keychain refused the API key: {}", String::from_utf8_lossy(&output.stderr).trim()))
    }
}

#[cfg(target_os = "macos")]
fn platform_get(service: &str, account: &str) -> Result<Option<String>, String> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", service, "-a", account, "-w"])
        .output()
        .map_err(|error| format!("macOS Keychain: {error}"))?;
    if !output.status.success() {
        return Ok(None);
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if value.is_empty() { None } else { Some(value) })
}

#[cfg(target_os = "macos")]
fn platform_delete(service: &str, account: &str) -> Result<(), String> {
    let output = Command::new("security")
        .args(["delete-generic-password", "-s", service, "-a", account])
        .output()
        .map_err(|error| format!("macOS Keychain: {error}"))?;
    if output.status.success() || String::from_utf8_lossy(&output.stderr).contains("could not be found") {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

// ── Windows ──────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_store {
    use std::ffi::c_void;
    use std::ptr::null_mut;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct CredentialW {
        flags: u32,
        kind: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: FileTime,
        credential_blob_size: u32,
        credential_blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut c_void,
        target_alias: *mut u16,
        username: *mut u16,
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn CredWriteW(credential: *const CredentialW, flags: u32) -> i32;
        fn CredReadW(target: *const u16, kind: u32, flags: u32, credential: *mut *mut CredentialW) -> i32;
        fn CredDeleteW(target: *const u16, kind: u32, flags: u32) -> i32;
        fn CredFree(buffer: *mut c_void);
    }

    const CRED_TYPE_GENERIC: u32 = 1;
    const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;
    const ERROR_NOT_FOUND: i32 = 1168;

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn target(service: &str, account: &str) -> String {
        format!("{service}:{account}")
    }

    pub fn set(service: &str, account: &str, value: &str) -> Result<(), String> {
        let mut target_name = wide(&target(service, account));
        let mut username = wide(account);
        let mut blob = value.as_bytes().to_vec();
        let credential = CredentialW {
            flags: 0,
            kind: CRED_TYPE_GENERIC,
            target_name: target_name.as_mut_ptr(),
            comment: null_mut(),
            last_written: FileTime { low: 0, high: 0 },
            credential_blob_size: blob.len() as u32,
            credential_blob: blob.as_mut_ptr(),
            persist: CRED_PERSIST_LOCAL_MACHINE,
            attribute_count: 0,
            attributes: null_mut(),
            target_alias: null_mut(),
            username: username.as_mut_ptr(),
        };
        let ok = unsafe { CredWriteW(&credential, 0) };
        if ok != 0 {
            Ok(())
        } else {
            Err(format!("Windows Credential Manager: {}", std::io::Error::last_os_error()))
        }
    }

    pub fn get(service: &str, account: &str) -> Result<Option<String>, String> {
        let target_name = wide(&target(service, account));
        let mut credential: *mut CredentialW = null_mut();
        let ok = unsafe { CredReadW(target_name.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
        if ok == 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(ERROR_NOT_FOUND) {
                return Ok(None);
            }
            return Err(format!("Windows Credential Manager: {error}"));
        }
        if credential.is_null() {
            return Ok(None);
        }
        let result = unsafe {
            let credential = &*credential;
            let bytes =
                std::slice::from_raw_parts(credential.credential_blob, credential.credential_blob_size as usize);
            let value = String::from_utf8(bytes.to_vec()).map_err(|error| format!("stored key is not UTF-8: {error}"));
            CredFree(credential as *const CredentialW as *mut c_void);
            value.map(Some)
        };
        result
    }

    pub fn delete(service: &str, account: &str) -> Result<(), String> {
        let target_name = wide(&target(service, account));
        let ok = unsafe { CredDeleteW(target_name.as_ptr(), CRED_TYPE_GENERIC, 0) };
        if ok != 0 {
            return Ok(());
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(ERROR_NOT_FOUND) {
            Ok(())
        } else {
            Err(format!("Windows Credential Manager: {error}"))
        }
    }
}

#[cfg(target_os = "windows")]
fn platform_set(service: &str, account: &str, value: &str) -> Result<(), String> {
    windows_store::set(service, account, value)
}

#[cfg(target_os = "windows")]
fn platform_get(service: &str, account: &str) -> Result<Option<String>, String> {
    windows_store::get(service, account)
}

#[cfg(target_os = "windows")]
fn platform_delete(service: &str, account: &str) -> Result<(), String> {
    windows_store::delete(service, account)
}

// ── Linux ────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn secret_tool(args: &[&str], stdin: Option<&str>) -> Result<std::process::Output, String> {
    use std::io::Write;
    use std::process::Stdio;
    let mut command = Command::new("secret-tool");
    command.args(args);
    if let Some(input) = stdin {
        command.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|error| {
            format!("Secret Service helper `secret-tool` is unavailable ({error}); install libsecret-tools")
        })?;
        if let Some(mut pipe) = child.stdin.take() {
            pipe.write_all(input.as_bytes()).map_err(|error| format!("secret-tool stdin: {error}"))?;
        }
        return child.wait_with_output().map_err(|error| format!("secret-tool: {error}"));
    }
    command.output().map_err(|error| {
        format!("Secret Service helper `secret-tool` is unavailable ({error}); install libsecret-tools")
    })
}

#[cfg(target_os = "linux")]
fn platform_set(service: &str, account: &str, value: &str) -> Result<(), String> {
    let output = secret_tool(
        &["store", "--label=Poieweave inference key", "service", service, "account", account],
        Some(value),
    )?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Secret Service refused the API key: {}", String::from_utf8_lossy(&output.stderr).trim()))
    }
}

#[cfg(target_os = "linux")]
fn platform_get(service: &str, account: &str) -> Result<Option<String>, String> {
    let output = secret_tool(&["lookup", "service", service, "account", account], None)?;
    if !output.status.success() {
        return Ok(None);
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if value.is_empty() { None } else { Some(value) })
}

#[cfg(target_os = "linux")]
fn platform_delete(service: &str, account: &str) -> Result<(), String> {
    let output = secret_tool(&["clear", "service", service, "account", account], None)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

// ── fallback ─────────────────────────────────────────────────────────────

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_set(_service: &str, _account: &str, _value: &str) -> Result<(), String> {
    Err("no native credential store on this platform".into())
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_get(_service: &str, _account: &str) -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_delete(_service: &str, _account: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
pub struct MemorySecretStore {
    values: std::sync::Mutex<std::collections::BTreeMap<(String, String), String>>,
}

#[cfg(test)]
impl MemorySecretStore {
    pub fn new() -> MemorySecretStore {
        MemorySecretStore { values: std::sync::Mutex::new(std::collections::BTreeMap::new()) }
    }
}

#[cfg(test)]
impl SecretStore for MemorySecretStore {
    fn set(&self, service: &str, account: &str, value: &str) -> Result<(), String> {
        self.values.lock().unwrap().insert((service.to_string(), account.to_string()), value.to_string());
        Ok(())
    }

    fn get(&self, service: &str, account: &str) -> Result<Option<String>, String> {
        Ok(self.values.lock().unwrap().get(&(service.to_string(), account.to_string())).cloned())
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), String> {
        self.values.lock().unwrap().remove(&(service.to_string(), account.to_string()));
        Ok(())
    }
}
