//! Light at sign-in: the `BezelEvoLight` entry of the user's `Run` key, the
//! same one the installer's option writes, so Studio and Setup turn one
//! start on and off.

use std::io;
use std::path::{Path, PathBuf};

/// The user's programs started at sign-in.
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// The entry that starts Light (the installer's name for it).
const LIGHT_ENTRY: &str = "BezelEvoLight";
/// The entry of Studio's former start at sign-in, which ran Studio itself
/// in the tray beside Light's: replaced by [`LIGHT_ENTRY`].
const STUDIO_ENTRY: &str = "Bezel Evo";
/// The script that starts Light, beside the installation's programs.
const LAUNCHER: &str = "start-light.ps1";

/// The start at sign-in of the Light of one installation.
pub struct LightLogin {
    key: String,
    launcher: PathBuf,
    studio: PathBuf,
}

impl LightLogin {
    /// The Light of the installation `executable` (Studio's program) is in.
    pub fn beside(executable: &Path) -> Self {
        Self::under(RUN_KEY, executable)
    }

    fn under(key: &str, executable: &Path) -> Self {
        Self {
            key: key.to_owned(),
            launcher: executable.with_file_name(LAUNCHER),
            studio: executable.to_owned(),
        }
    }

    /// Whether this installation's Light starts at sign-in.
    pub fn is_enabled(&self) -> io::Result<bool> {
        Ok(registry::read(&self.key, LIGHT_ENTRY)?.is_some_and(|entry| self.owns(&entry)))
    }

    /// Starts this installation's Light at sign-in, or no more. An entry of
    /// another installation is left as it is: turning it on says so, turning
    /// it off does nothing.
    pub fn set(&self, on: bool) -> io::Result<()> {
        let entry = registry::read(&self.key, LIGHT_ENTRY)?;
        let ours = entry.as_deref().is_some_and(|e| self.owns(e));
        if on {
            if !self.launcher.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "this installation has no Light launcher",
                ));
            }
            if entry.is_some() && !ours {
                return Err(io::Error::other(
                    "another installation owns the Light sign-in entry",
                ));
            }
            registry::write(&self.key, LIGHT_ENTRY, &self.command())
        } else if ours {
            registry::delete(&self.key, LIGHT_ENTRY)
        } else {
            Ok(())
        }
    }

    /// Replaces this installation's former start of Studio at sign-in with
    /// Light's, so that one program starts and not two that race for the
    /// screens. An entry of another installation is left as it is.
    pub fn migrate(&self) -> io::Result<()> {
        let Some(entry) = registry::read(&self.key, STUDIO_ENTRY)? else {
            return Ok(());
        };
        if !contains(&entry, &self.studio) {
            return Ok(());
        }
        registry::delete(&self.key, STUDIO_ENTRY)?;
        match self.set(true) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }

    fn owns(&self, entry: &str) -> bool {
        contains(entry, &self.launcher)
    }

    /// The installer's command: Windows PowerShell running the launcher hidden.
    fn command(&self) -> String {
        let windows = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        format!(
            r#""{windows}\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File "{}""#,
            self.launcher.display()
        )
    }
}

/// Whether `entry` runs `program`, quoted, as Windows compares paths (case aside).
fn contains(entry: &str, program: &Path) -> bool {
    let quoted = format!("\"{}\"", program.display()).to_lowercase();
    entry.to_lowercase().contains(&quoted)
}

#[allow(
    unsafe_code,
    reason = "Win32 FFI: the Run key is read and written through the registry API"
)]
mod registry {
    use std::io;
    use std::ptr;

    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR};
    use windows_sys::Win32::System::Registry::{
        HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
    };

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(Some(0)).collect()
    }

    fn check(status: WIN32_ERROR) -> io::Result<()> {
        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(status as i32))
        }
    }

    /// The string value `name` of `key` under the current user; `None` when
    /// the key or the value is missing.
    pub(super) fn read(key: &str, name: &str) -> io::Result<Option<String>> {
        let (key, name) = (wide(key), wide(name));
        let mut bytes = 0u32;
        // SAFETY: both names are NUL-terminated UTF-16 that outlive the call;
        // a null buffer with a valid size pointer only asks for the size.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut bytes,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        check(status)?;
        let mut buffer = vec![0u16; (bytes as usize).div_ceil(2)];
        let mut size = (buffer.len() * 2) as u32;
        // SAFETY: as above; `buffer` holds `size` bytes and lives past the call.
        check(unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut size,
            )
        })?;
        let text = &buffer[..(size as usize / 2)];
        let end = text.iter().position(|&c| c == 0).unwrap_or(text.len());
        Ok(Some(String::from_utf16_lossy(&text[..end])))
    }

    /// Sets the string value `name` of `key` (made if missing).
    pub(super) fn write(key: &str, name: &str, value: &str) -> io::Result<()> {
        let (key, name, value) = (wide(key), wide(name), wide(value));
        // SAFETY: NUL-terminated UTF-16 buffers alive for the call; the size
        // given is `value`'s, terminator included, in bytes.
        check(unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                REG_SZ,
                value.as_ptr().cast(),
                (value.len() * 2) as u32,
            )
        })
    }

    /// Removes the value `name` of `key`; nothing to do when it is missing.
    pub(super) fn delete(key: &str, name: &str) -> io::Result<()> {
        let (key, name) = (wide(key), wide(name));
        // SAFETY: NUL-terminated UTF-16 buffers alive for the call.
        let status = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr()) };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        check(status)
    }

    #[cfg(test)]
    pub(super) fn delete_key(key: &str) {
        use windows_sys::Win32::System::Registry::RegDeleteTreeW;
        let key = wide(key);
        // SAFETY: a NUL-terminated UTF-16 name alive for the call.
        unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, key.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway key under the current user, removed when dropped.
    struct Scratch(String);

    impl Scratch {
        fn new(name: &str) -> Self {
            Self(format!(
                r"Software\BezelPowerTests\{name}-{}",
                std::process::id()
            ))
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            registry::delete_key(&self.0);
        }
    }

    fn installation() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(LAUNCHER), "").unwrap();
        let studio = dir.path().join("bezel-studio.exe");
        (dir, studio)
    }

    #[test]
    fn turning_on_writes_the_installers_entry_and_off_removes_it() {
        let key = Scratch::new("toggle");
        let (_dir, studio) = installation();
        let login = LightLogin::under(&key.0, &studio);

        assert!(!login.is_enabled().unwrap());
        login.set(true).unwrap();
        assert!(login.is_enabled().unwrap());
        let entry = registry::read(&key.0, LIGHT_ENTRY).unwrap().unwrap();
        assert!(entry.contains("-ExecutionPolicy Bypass -File \""));
        assert!(entry.ends_with(&format!("\"{}\"", login.launcher.display())));

        login.set(false).unwrap();
        assert!(!login.is_enabled().unwrap());
        assert_eq!(registry::read(&key.0, LIGHT_ENTRY).unwrap(), None);
    }

    #[test]
    fn without_a_launcher_it_does_not_turn_on() {
        let key = Scratch::new("no-launcher");
        let dir = tempfile::tempdir().unwrap();
        let login = LightLogin::under(&key.0, &dir.path().join("bezel-studio.exe"));

        let error = login.set(true).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert_eq!(registry::read(&key.0, LIGHT_ENTRY).unwrap(), None);
    }

    #[test]
    fn another_installations_entry_is_left_alone() {
        let key = Scratch::new("other");
        let (_dir, studio) = installation();
        let login = LightLogin::under(&key.0, &studio);
        let other = r#""powershell.exe" -File "D:\Other\start-light.ps1""#;
        registry::write(&key.0, LIGHT_ENTRY, other).unwrap();

        assert!(!login.is_enabled().unwrap());
        assert!(login.set(true).is_err());
        login.set(false).unwrap();
        assert_eq!(
            registry::read(&key.0, LIGHT_ENTRY).unwrap().as_deref(),
            Some(other)
        );
    }

    #[test]
    fn studios_former_entry_becomes_lights() {
        let key = Scratch::new("migrate");
        let (_dir, studio) = installation();
        let login = LightLogin::under(&key.0, &studio);
        let former = format!("\"{}\" --hidden", studio.display()).to_uppercase();
        registry::write(&key.0, STUDIO_ENTRY, &former).unwrap();

        login.migrate().unwrap();
        assert_eq!(registry::read(&key.0, STUDIO_ENTRY).unwrap(), None);
        assert!(login.is_enabled().unwrap());
    }

    #[test]
    fn another_studios_former_entry_is_left_alone() {
        let key = Scratch::new("migrate-other");
        let (_dir, studio) = installation();
        let login = LightLogin::under(&key.0, &studio);
        let former = r#""D:\Other\bezel-studio.exe" --hidden"#;
        registry::write(&key.0, STUDIO_ENTRY, former).unwrap();

        login.migrate().unwrap();
        assert_eq!(
            registry::read(&key.0, STUDIO_ENTRY).unwrap().as_deref(),
            Some(former)
        );
        assert!(!login.is_enabled().unwrap());
    }
}
