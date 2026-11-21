use crate::sensors::now_ms;
use crate::types::{DiskInfo, DisplayInfo, HardwareInfo, ProcessInfo};
use sysinfo::System;
use windows_sys::Win32::Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS};
use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetLogicalDrives};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId,
};

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn open_key(path: &str) -> Option<HKEY> {
    let path = wide(path);
    let mut key: HKEY = std::ptr::null_mut();
    let status = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, path.as_ptr(), 0, KEY_READ | KEY_WOW64_64KEY, &mut key) };
    if status == 0 { Some(key) } else { None }
}

fn query_value(key: HKEY, name: &str) -> Option<(Vec<u8>, u32)> {
    let name = wide(name);
    let mut kind: u32 = 0;
    let mut size: u32 = 0;
    let status = unsafe { RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), &mut kind, std::ptr::null_mut(), &mut size) };
    if status != 0 || size == 0 || size > 65536 {
        return None;
    }
    let mut data = vec![0u8; size as usize];
    let status = unsafe { RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), &mut kind, data.as_mut_ptr(), &mut size) };
    if status != 0 {
        return None;
    }
    data.truncate(size as usize);
    Some((data, kind))
}

fn utf16_string(data: &[u8]) -> Option<String> {
    if data.len() < 2 {
        return None;
    }
    let mut units: Vec<u16> = data.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    while units.last() == Some(&0) {
        units.pop();
    }
    Some(String::from_utf16_lossy(&units))
}

pub fn reg_string(path: &str, name: &str) -> Option<String> {
    let key = open_key(path)?;
    let value = query_value(key, name).and_then(|(data, _)| utf16_string(&data));
    unsafe { RegCloseKey(key) };
    value
}

pub fn reg_u64(path: &str, name: &str) -> Option<u64> {
    let key = open_key(path)?;
    let value = query_value(key, name).and_then(|(data, _)| {
        let bytes: [u8; 8] = data.get(0..8)?.try_into().ok()?;
        Some(u64::from_le_bytes(bytes))
    });
    unsafe { RegCloseKey(key) };
    value.filter(|v| *v > 0)
}

const DISPLAY_CLASS: &str = r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";

/// The display adapter with the largest dedicated video memory. Virtual and
/// software adapters report none, so they never win this comparison.
pub fn primary_adapter() -> Option<(String, u64)> {
    let mut best: Option<(String, u64)> = None;
    for index in 0..8 {
        let path = format!("{DISPLAY_CLASS}\\{index:04}");
        let Some(name) = reg_string(&path, "DriverDesc") else { continue };
        let Some(memory) = reg_u64(&path, "HardwareInformation.qwMemorySize") else { continue };
        if best.as_ref().map(|(_, m)| memory > *m).unwrap_or(true) {
            best = Some((name, memory));
        }
    }
    best
}

fn disks() -> Vec<DiskInfo> {
    let mask = unsafe { GetLogicalDrives() };
    let mut out = Vec::new();
    for bit in 0..26u32 {
        if mask & (1 << bit) == 0 {
            continue;
        }
        let letter = (b'A' + bit as u8) as char;
        let root = format!("{letter}:\\");
        let root_w = wide(&root);
        let mut free_to_caller: u64 = 0;
        let mut total: u64 = 0;
        let mut total_free: u64 = 0;
        let ok = unsafe {
            GetDiskFreeSpaceExW(root_w.as_ptr(), &mut free_to_caller, &mut total, &mut total_free)
        };
        if ok != 0 && total > 0 {
            out.push(DiskInfo { mount: format!("{letter}:"), total_bytes: total, free_bytes: total_free });
        }
    }
    out
}

fn display() -> Option<DisplayInfo> {
    let mut devmode: DEVMODEW = unsafe { std::mem::zeroed() };
    devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
    let ok = unsafe { EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut devmode) };
    if ok == 0 {
        return None;
    }
    Some(DisplayInfo {
        width: devmode.dmPelsWidth,
        height: devmode.dmPelsHeight,
        refresh_hz: devmode.dmDisplayFrequency.max(1),
    })
}

pub fn foreground_pid() -> Option<u32> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return None;
    }
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    (pid != 0).then_some(pid)
}

pub fn collect_hardware(sampler: &crate::sensors::Sampler) -> HardwareInfo {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let adapter = primary_adapter();
    let threads = sys.cpus().len() as u32;
    let physical = System::physical_core_count(&sys).unwrap_or(threads as usize) as u32;
    let cpu_name = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Unknown processor".into());
    let product = reg_string(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "ProductName")
        .unwrap_or_else(|| "Windows".into());
    let build = reg_string(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "CurrentBuild")
        .unwrap_or_default();
    let release = reg_string(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "DisplayVersion")
        .unwrap_or_default();
    let mut notes = sampler.adapter_notes.clone();
    if adapter.is_none() {
        notes.push("No display adapter with dedicated memory was found in the registry".into());
    }
    HardwareInfo {
        cpu_name,
        cpu_cores: physical,
        cpu_threads: threads,
        gpu_name: adapter.as_ref().map(|(n, _)| n.clone()).unwrap_or_else(|| "Unknown adapter".into()),
        vram_total_bytes: adapter.map(|(_, m)| m),
        ram_total_bytes: sys.total_memory(),
        os_name: product,
        os_version: release,
        os_build: build,
        disks: disks(),
        display: display(),
        adapter_luid: sampler.selected_luid.clone(),
        notes,
        collected_at_ms: now_ms(),
    }
}

pub fn list_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let foreground = foreground_pid();
    let mut out: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            exe_path: process.exe().map(|p| p.to_string_lossy().to_string()),
            memory_bytes: process.memory(),
            cpu_percent: process.cpu_usage(),
            is_foreground: Some(pid.as_u32()) == foreground,
        })
        .collect();
    out.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    out.truncate(120);
    out
}

pub fn process_name(pid: u32) -> Option<String> {
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.process(sysinfo::Pid::from_u32(pid)).map(|p| p.name().to_string_lossy().to_string())
}
