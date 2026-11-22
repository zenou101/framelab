//! Library discovery and launching.
//!
//! Discovery only reads launcher metadata that already exists on disk. Nothing
//! is guessed: a game appears here only if a launcher manifest names it.

use crate::hardware::reg_string;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
};

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

const PACKAGE_REPOSITORY: &str =
    r"Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\Repository\Packages";

#[derive(Debug, Clone)]
struct AppxPackage {
    /// The package name, which is the identity name shared with MicrosoftGame.config.
    name: String,
    full_name: String,
    root: Option<PathBuf>,
    display_name: Option<String>,
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn read_reg_string(key: HKEY, name: &str) -> Option<String> {
    let name = wide(name);
    let mut kind: u32 = 0;
    let mut size: u32 = 0;
    let status = unsafe {
        RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), &mut kind, std::ptr::null_mut(), &mut size)
    };
    if status != 0 || size < 2 || size > 65_536 {
        return None;
    }
    let mut data = vec![0u8; size as usize];
    let status = unsafe {
        RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), &mut kind, data.as_mut_ptr(), &mut size)
    };
    if status != 0 {
        return None;
    }
    data.truncate(size as usize);
    let mut units: Vec<u16> = data.chunks_exact(2).map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]])).collect();
    while units.last() == Some(&0) {
        units.pop();
    }
    let value = String::from_utf16_lossy(&units);
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn open_subkey(parent: HKEY, name: &str) -> Option<HKEY> {
    let name = wide(name);
    let mut key: HKEY = std::ptr::null_mut();
    let status = unsafe { RegOpenKeyExW(parent, name.as_ptr(), 0, KEY_READ, &mut key) };
    (status == 0).then_some(key)
}

/// Every Store/MSIX package registered for the current user. Game packages carry
/// a MicrosoftGame.config in their install root; the rest are ignored later.
fn appx_packages() -> Vec<AppxPackage> {
    let mut out = Vec::new();
    let Some(repository) = open_subkey(HKEY_CURRENT_USER, PACKAGE_REPOSITORY) else {
        return out;
    };
    let mut index = 0;
    loop {
        let mut buffer = [0u16; 512];
        let mut length = buffer.len() as u32;
        let status = unsafe {
            RegEnumKeyExW(
                repository,
                index,
                buffer.as_mut_ptr(),
                &mut length,
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if status != 0 {
            break;
        }
        index += 1;
        let full_name = String::from_utf16_lossy(&buffer[..length as usize]);
        let Some(name) = full_name.split('_').next().map(str::to_string) else { continue };
        let Some(key) = open_subkey(repository, &full_name).filter(|_| !name.is_empty()) else {
            continue;
        };
        let root = read_reg_string(key, "PackageRootFolder").map(PathBuf::from).filter(|path| path.is_dir());
        let display_name = read_reg_string(key, "DisplayName");
        unsafe { RegCloseKey(key) };
        out.push(AppxPackage { name, full_name, root, display_name });
    }
    unsafe { RegCloseKey(repository) };
    out
}

/// MicrosoftGame.config is shipped with mixed casing, so look it up without
/// assuming how the installer wrote the file name.
fn find_file_ci(dir: &Path, file_name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        entry
            .file_name()
            .to_string_lossy()
            .as_ref()
            .eq_ignore_ascii_case(file_name)
            .then(|| entry.path())
    })
}

/// Tags are small and their attributes are well-formed, so a targeted scanner
/// avoids pulling in an XML dependency just to read a handful of values.
fn tags<'a>(xml: &'a str, name: &str) -> Vec<&'a str> {
    let needle = format!("<{name}");
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(position) = xml[cursor..].find(&needle) {
        let start = cursor + position;
        let after = &xml[start + needle.len()..];
        let boundary = after.chars().next();
        if matches!(boundary, Some(character) if character.is_whitespace() || character == '>' || character == '/') {
            if let Some(end) = xml[start..].find('>') {
                out.push(&xml[start..start + end + 1]);
            }
        }
        cursor = start + needle.len();
    }
    out
}

fn tag_attr(tag: &str, key: &str) -> Option<String> {
    let mut rest = tag;
    loop {
        let position = rest.find(key)?;
        let before_ok = position == 0
            || rest.as_bytes().get(position - 1).map(|byte| byte.is_ascii_whitespace()).unwrap_or(false);
        let after = rest[position + key.len()..].trim_start();
        if before_ok {
            if let Some(value) = after.strip_prefix('=') {
                let value = value.trim_start();
                let quote = value.chars().next()?;
                if quote == '"' || quote == '\'' {
                    let end = value[1..].find(quote)?;
                    return Some(decode_xml_entities(&value[1..1 + end]));
                }
            }
        }
        rest = &rest[position + key.len()..];
    }
}

fn decode_xml_entities(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn clean_display_name(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"');
    if value.is_empty() || value.starts_with("ms-resource:") || value.contains("@{") {
        return None;
    }
    let name: String = value.chars().take(120).collect();
    (!name.is_empty()).then_some(name)
}

struct XboxManifest {
    name: Option<String>,
    identity: Option<String>,
    executable_name: String,
    executable_id: Option<String>,
}

/// Reads the parts of MicrosoftGame.config that identify a playable title.
/// Add-on packages have no ExecutableList (or point at a main package), so they
/// are deliberately rejected instead of appearing as separate games.
fn parse_xbox_config(xml: &str) -> Option<XboxManifest> {
    let xml = xml.trim_start_matches('\u{feff}');
    if !tags(xml, "MainPackageDependency").is_empty() {
        return None;
    }
    let identity = tags(xml, "Identity").first().and_then(|tag| tag_attr(tag, "Name"));
    let name = tags(xml, "ShellVisuals")
        .first()
        .and_then(|tag| tag_attr(tag, "DefaultDisplayName"))
        .and_then(|value| clean_display_name(&value));
    let executable = tags(xml, "Executable").into_iter().find_map(|tag| {
        let dev_only = tag_attr(&tag, "IsDevOnly")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if dev_only {
            return None;
        }
        let executable_name = tag_attr(&tag, "Name")?;
        let executable_id = tag_attr(&tag, "Id");
        Some((executable_name, executable_id))
    })?;
    Some(XboxManifest {
        name,
        identity,
        executable_name: executable.0,
        executable_id: executable.1,
    })
}

fn parse_application(xml: &str, wanted_id: Option<&str>) -> Option<(String, Option<String>)> {
    let applications = tags(xml, "Application");
    let chosen = wanted_id
        .and_then(|wanted| applications.iter().find(|tag| tag_attr(tag, "Id").as_deref() == Some(wanted)))
        .or_else(|| applications.first())?;
    let id = tag_attr(chosen, "Id")?;
    let executable = tag_attr(chosen, "Executable");
    Some((id, executable))
}

fn package_family(full_name: &str) -> Option<String> {
    let name = full_name.split('_').next()?.trim();
    let publisher = full_name.rsplit('_').next()?.trim();
    if name.is_empty() || publisher.len() < 8 {
        return None;
    }
    Some(format!("{name}_{publisher}"))
}

/// A readable fallback for machines where the WindowsApps package directory is
/// protected. The Xbox app stages the same manifests under <drive>:\XboxGames.
fn staged_xbox_manifests() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let mask = unsafe { GetLogicalDrives() };
    for bit in 0..26u32 {
        if mask & (1 << bit) == 0 {
            continue;
        }
        let letter = (b'A' + bit as u8) as char;
        let root = PathBuf::from(format!("{letter}:\\XboxGames"));
        if !root.is_dir() {
            continue;
        }
        let drive = wide(&format!("{letter}:\\"));
        if unsafe { GetDriveTypeW(drive.as_ptr()) } != 3 {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&root) else { continue };
        for entry in entries.flatten() {
            let folder = entry.path();
            if !folder.is_dir() {
                continue;
            }
            let content = folder.join("Content");
            let candidate = if content.is_dir() { content } else { folder };
            let Some(config) = find_file_ci(&candidate, "MicrosoftGame.config") else { continue };
            let Ok(xml) = std::fs::read_to_string(&config) else { continue };
            let Some(identity) = tags(&xml, "Identity").first().and_then(|tag| tag_attr(tag, "Name")) else {
                continue;
            };
            out.push((identity, config));
        }
    }
    out
}

pub fn xbox_games() -> Vec<Detected> {
    let packages = appx_packages();
    let staged = staged_xbox_manifests();
    let mut out = Vec::new();

    for package in packages {
        let direct = package
            .root
            .as_deref()
            .and_then(|root| find_file_ci(root, "MicrosoftGame.config"));
        let fallback = staged
            .iter()
            .find(|(identity, _)| identity.eq_ignore_ascii_case(&package.name))
            .map(|(_, path)| path.clone());
        let Some(config_path) = direct.or(fallback) else { continue };
        let Ok(xml) = std::fs::read_to_string(&config_path) else { continue };
        let Some(manifest) = parse_xbox_config(&xml) else { continue };
        let Some(family) = package_family(&package.full_name) else { continue };

        let app_manifest = find_file_ci(config_path.parent().unwrap_or(Path::new(".")), "AppxManifest.xml")
            .and_then(|path| std::fs::read_to_string(path).ok());
        let application = app_manifest
            .as_deref()
            .and_then(|xml| parse_application(xml, manifest.executable_id.as_deref()));
        let app_id = application
            .as_ref()
            .map(|(id, _)| id.clone())
            .or_else(|| manifest.executable_id.clone());
        let Some(app_id) = app_id else { continue };
        if app_id.is_empty() {
            continue;
        }

        let directory = config_path.parent().map(Path::to_path_buf);
        let exe_path = directory.as_ref().and_then(|dir| {
            application
                .as_ref()
                .and_then(|(_, executable)| executable.as_ref())
                .map(|executable| dir.join(executable))
                .filter(|path| path.is_file())
                .or_else(|| {
                    let path = dir.join(&manifest.executable_name);
                    path.is_file().then_some(path)
                })
        });
        let name = manifest
            .name
            .or_else(|| package.display_name.as_deref().and_then(clean_display_name))
            .or_else(|| manifest.identity.as_deref().and_then(clean_display_name))
            .unwrap_or_else(|| package.name.clone());

        out.push(Detected {
            name,
            source: "xbox".into(),
            launcher_id: format!("xbox:{family}!{app_id}"),
            exe_path: exe_path.map(|path| path.to_string_lossy().to_string()),
            install_dir: directory.map(|path| path.to_string_lossy().to_string()),
        });
    }

    out.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
    out.dedup_by(|a, b| a.launcher_id == b.launcher_id);
    out
}

pub fn detect_all() -> Vec<Detected> {
    let mut out = steam_games();
    out.extend(epic_games());
    out.extend(xbox_games());
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

    if let Some(launcher) = launcher_id.filter(|id| id.starts_with("xbox:")) {
        let aumid = launcher.trim_start_matches("xbox:");
        let valid = !aumid.is_empty()
            && aumid.matches('!').count() == 1
            && aumid
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_' | '!'));
        if !valid {
            return Err("Malformed Xbox app identifier".into());
        }
        // Store games must be activated through their package identity so that
        // the game receives its licence and saves. The executable alone is not
        // enough for many Xbox app titles.
        let target = wide(&format!("shell:AppsFolder\\{aumid}"));
        let operation = wide("open");
        let result = unsafe {
            ShellExecuteW(std::ptr::null_mut(), operation.as_ptr(), target.as_ptr(), std::ptr::null(), std::ptr::null(), 1)
        };
        return if result as isize > 32 { Ok(()) } else { Err("Windows could not start this Xbox app game".into()) };
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

#[cfg(test)]
mod tests {
    use super::*;

    const GAME_CONFIG: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Game configVersion="1">
  <Identity Name="MadDogGamesLLC.5f2da432-9573-4e3d-ba9a-2e27c5e0f47" Publisher="CN=09506C77" Version="1.1.47.0"/>
  <ExecutableList>
    <Executable Name="wwzRetail.exe" TargetDeviceFamily="PC" Id="WWZRetail" IsDevOnly="false"/>
  </ExecutableList>
  <ShellVisuals DefaultDisplayName="World War Z" PublisherDisplayName="Saber Interactive Inc."/>
</Game>"#;

    const ADDON_CONFIG: &str = r#"<Game configVersion="1">
  <Identity Name="MadDogGamesLLC.WWZAftermathUpgrade" Publisher="CN=09506C77" Version="1.0.1.0"/>
  <ShellVisuals DefaultDisplayName="World War Z: Aftermath Upgrade"/>
  <DesktopRegistration>
    <MainPackageDependency Name="MadDogGamesLLC.5f2da432-9573-4e3d-ba9a-2e27c5e0f47"/>
  </DesktopRegistration>
</Game>"#;

    const APPX_MANIFEST: &str = r#"<Package>
  <Applications>
    <Application Id="WWZRetail" Executable="GameLaunchHelper.exe" EntryPoint="Windows.FullTrustApplication"/>
  </Applications>
</Package>"#;

    #[test]
    fn reads_game_manifest() {
        let manifest = parse_xbox_config(GAME_CONFIG).expect("manifest should parse");
        assert_eq!(manifest.name.as_deref(), Some("World War Z"));
        assert_eq!(manifest.executable_name, "wwzRetail.exe");
        assert_eq!(manifest.executable_id.as_deref(), Some("WWZRetail"));
    }

    #[test]
    fn ignores_addon_manifest() {
        assert!(parse_xbox_config(ADDON_CONFIG).is_none());
    }

    #[test]
    fn reads_application_launch_command() {
        let application = parse_application(APPX_MANIFEST, Some("WWZRetail")).expect("application should parse");
        assert_eq!(application.0, "WWZRetail");
        assert_eq!(application.1.as_deref(), Some("GameLaunchHelper.exe"));
    }

    #[test]
    fn builds_package_family_from_full_name() {
        assert_eq!(
            package_family("Microsoft.ForteBaseGame_3.483.970.0_x64__8wekyb3d8bbwe").as_deref(),
            Some("Microsoft.ForteBaseGame_8wekyb3d8bbwe")
        );
        assert_eq!(
            package_family("MadDogGamesLLC.5f2da432-9573-4e3d-ba9a-2e27c5e0f47_1.1.47.0_x64__6wkgvezv94m76")
                .as_deref(),
            Some("MadDogGamesLLC.5f2da432-9573-4e3d-ba9a-2e27c5e0f47_6wkgvezv94m76")
        );
    }
}
