//! Library discovery and launching.
//!
//! Discovery only reads launcher metadata that already exists on disk. Nothing
//! is guessed: a game appears here only if a launcher manifest names it.

use crate::hardware::reg_string;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Detected {
    pub name: String,
    pub source: String,
    pub launcher_id: String,
    pub exe_path: Option<String>,
    pub install_dir: Option<String>,
}

fn quoted_pairs(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split('"').collect();
        if fields.len() >= 4 {
            let key = fields[1].trim().to_string();
            let value = fields[3].trim().to_string();
            if !key.is_empty() && !value.is_empty() {
                out.push((key, value));
            }
        }
    }
    out
}

fn steam_root() -> Option<PathBuf> {
    if let Some(path) = reg_string(r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath") {
        let candidate = PathBuf::from(path);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    for candidate in [r"C:\Program Files (x86)\Steam", r"C:\Program Files\Steam"] {
        let path = PathBuf::from(candidate);
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

fn find_primary_exe(dir: &Path, hint: &str) -> Option<String> {
    let wanted = format!("{}.exe", hint.to_ascii_lowercase());
    let entries = std::fs::read_dir(dir).ok()?;
    let mut candidates: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false))
        .collect();
    if let Some(exact) = candidates.iter().find(|path| {
        path.file_name().map(|n| n.to_string_lossy().to_ascii_lowercase()) == Some(wanted.clone())
    }) {
        return Some(exact.to_string_lossy().to_string());
    }
    candidates.sort();
    candidates.first().map(|path| path.to_string_lossy().to_string())
}

pub fn steam_games() -> Vec<Detected> {
    let Some(root) = steam_root() else { return Vec::new() };
    let mut libraries: Vec<PathBuf> = vec![root.clone()];
    if let Ok(text) = std::fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf")) {
        for (key, value) in quoted_pairs(&text) {
            if key == "path" {
                let path = PathBuf::from(value);
                if path.is_dir() && !libraries.contains(&path) {
                    libraries.push(path);
                }
            }
        }
    }
    let mut out = Vec::new();
    for library in libraries {
        let apps = library.join("steamapps");
        let Ok(entries) = std::fs::read_dir(&apps) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().map(|n| n.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
            if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            let pairs = quoted_pairs(&text);
            let get = |key: &str| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
            let Some(app_id) = get("appid") else { continue };
            let Some(game_name) = get("name") else { continue };
            let install_dir = get("installdir");
            let folder = install_dir.as_ref().map(|dir| apps.join("common").join(dir));
            let exe = folder.as_ref().and_then(|dir| find_primary_exe(dir, install_dir.as_deref().unwrap_or(&game_name)));
            out.push(Detected {
                name: game_name,
                source: "steam".into(),
                launcher_id: format!("steam:{app_id}"),
                exe_path: exe,
                install_dir: folder.map(|p| p.to_string_lossy().to_string()),
            });
        }
    }
    out
}

#[derive(Deserialize)]
struct EpicEntry {
    #[serde(rename = "AppName")]
    app_name: String,
    #[serde(rename = "InstallLocation")]
    install_location: Option<String>,
    #[serde(rename = "AppDisplayName")]
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct EpicFile {
    #[serde(rename = "InstallationList")]
    installations: Vec<EpicEntry>,
}

pub fn epic_games() -> Vec<Detected> {
    let path = Path::new(r"C:\ProgramData\Epic\UnrealEngineLauncher\LauncherInstalled.dat");
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    let Ok(data) = serde_json::from_str::<EpicFile>(&text) else { return Vec::new() };
    data.installations
        .into_iter()
        .filter(|entry| !entry.app_name.contains("UnrealEngine") && !entry.app_name.starts_with("UE_"))
        .map(|entry| {
            let folder = entry.install_location.as_ref().map(PathBuf::from).filter(|p| p.is_dir());
            let exe = folder.as_ref().and_then(|dir| find_primary_exe(dir, &entry.app_name));
            Detected {
                name: entry.display_name.unwrap_or_else(|| entry.app_name.clone()),
                source: "epic".into(),
                launcher_id: format!("epic:{}", entry.app_name),
                exe_path: exe,
                install_dir: folder.map(|p| p.to_string_lossy().to_string()),
            }
        })
        .collect()
}

pub fn detect_all() -> Vec<Detected> {
    let mut out = steam_games();
    out.extend(epic_games());
    out.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
    out.dedup_by(|a, b| a.launcher_id == b.launcher_id);
    out
}

/// Launching is limited to entries that already exist in the user's library.
/// The path is canonicalised and must resolve to a regular .exe; nothing is
/// passed through a shell, so no string is ever interpreted as a command.
pub fn launch(exe_path: Option<&str>, launcher_id: Option<&str>) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect()
    }

    if let Some(launcher) = launcher_id.filter(|id| id.starts_with("steam:")) {
        let app_id = launcher.trim_start_matches("steam:");
        if !app_id.chars().all(|c| c.is_ascii_digit()) {
            return Err("Malformed Steam app id".into());
        }
        let url = format!("steam://rungameid/{app_id}");
        let operation = wide("open");
        let target = wide(&url);
        let result = unsafe {
            ShellExecuteW(std::ptr::null_mut(), operation.as_ptr(), target.as_ptr(), std::ptr::null(), std::ptr::null(), 1)
        };
        return if result as isize > 32 { Ok(()) } else { Err("Windows could not open the Steam link".into()) };
    }

    let Some(path) = exe_path else {
        return Err("This entry has no executable recorded. Add the game executable first.".into());
    };
    let canonical = std::fs::canonicalize(path).map_err(|_| "The executable no longer exists at that path".to_string())?;
    if !canonical.is_file() {
        return Err("The recorded path is not a file".into());
    }
    if canonical.extension().map(|e| !e.eq_ignore_ascii_case("exe")).unwrap_or(true) {
        return Err("Only .exe files can be launched".into());
    }
    let operation = wide("open");
    let target = wide(&canonical.to_string_lossy());
    let directory = wide(canonical.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default().as_str());
    let result = unsafe {
        ShellExecuteW(std::ptr::null_mut(), operation.as_ptr(), target.as_ptr(), std::ptr::null(), directory.as_ptr(), 1)
    };
    if result as isize > 32 { Ok(()) } else { Err("Windows could not start the executable".into()) }
}
