//! Thin, panic-free wrapper over the Windows PDH API.
//!
//! Counter paths are always added with `PdhAddEnglishCounterW` so a localized
//! Windows install cannot break them. Wildcard counters are read with
//! `PdhGetFormattedCounterArrayW` using the documented two-call buffer
//! protocol; a failed call yields an empty result instead of a zero.

use windows_sys::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhExpandWildCardPathW,
    PdhGetFormattedCounterArrayW, PdhGetFormattedCounterValue, PdhOpenQueryW,
    PDH_FMT_COUNTERVALUE_ITEM_W,
};

#[allow(non_camel_case_types)]
type PDH_HQUERY = isize;
#[allow(non_camel_case_types)]
type PDH_HCOUNTER = isize;

const ERROR_SUCCESS: u32 = 0;
const PDH_MORE_DATA: u32 = 0x8000_07D2;
const PDH_CSTATUS_NEW_DATA: u32 = 0x0000_0001;
const PDH_FMT_DOUBLE: u32 = 0x0000_0200;

fn wide(path: &str) -> Vec<u16> {
    path.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn read_wide(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
        if len > 4096 {
            break;
        }
    }
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
}

pub struct PdhQuery {
    handle: PDH_HQUERY,
}

unsafe impl Send for PdhQuery {}

impl PdhQuery {
    pub fn open() -> Option<Self> {
        let mut handle: PDH_HQUERY = 0;
        let status = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut handle) };
        if status == ERROR_SUCCESS {
            Some(Self { handle })
        } else {
            None
        }
    }

    pub fn add(&self, path: &str) -> Option<PDH_HCOUNTER> {
        let path = wide(path);
        let mut counter: PDH_HCOUNTER = 0;
        let status = unsafe { PdhAddEnglishCounterW(self.handle, path.as_ptr(), 0, &mut counter) };
        if status == ERROR_SUCCESS {
            Some(counter)
        } else {
            None
        }
    }

    pub fn collect(&self) -> bool {
        unsafe { PdhCollectQueryData(self.handle) == ERROR_SUCCESS }
    }
}

impl Drop for PdhQuery {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.handle) };
    }
}

/// Read every instance of a wildcard counter.
pub fn read_array(counter: PDH_HCOUNTER) -> Vec<(String, f64)> {
    let mut size: u32 = 0;
    let mut count: u32 = 0;
    let status = unsafe {
        PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, std::ptr::null_mut())
    };
    if status != PDH_MORE_DATA || size == 0 {
        return Vec::new();
    }
    let mut buffer = vec![0u8; size as usize];
    let items = buffer.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
    let status =
        unsafe { PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, items) };
    if status != ERROR_SUCCESS {
        return Vec::new();
    }
    let slice = unsafe { std::slice::from_raw_parts(items, count as usize) };
    let mut out = Vec::with_capacity(slice.len());
    for item in slice {
        let cstatus = item.FmtValue.CStatus;
        if cstatus != ERROR_SUCCESS && cstatus != PDH_CSTATUS_NEW_DATA {
            continue;
        }
        let value = unsafe { item.FmtValue.Anonymous.doubleValue };
        if value.is_finite() {
            out.push((unsafe { read_wide(item.szName) }, value));
        }
    }
    out
}

pub fn read_value(counter: PDH_HCOUNTER) -> Option<f64> {
    let mut value: windows_sys::Win32::System::Performance::PDH_FMT_COUNTERVALUE =
        unsafe { std::mem::zeroed() };
    let status =
        unsafe { PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, std::ptr::null_mut(), &mut value) };
    if status != ERROR_SUCCESS {
        return None;
    }
    let cstatus = value.CStatus;
    if cstatus != ERROR_SUCCESS && cstatus != PDH_CSTATUS_NEW_DATA {
        return None;
    }
    let raw = unsafe { value.Anonymous.doubleValue };
    raw.is_finite().then_some(raw)
}

/// Expand a wildcard path into concrete instance paths (used to validate that
/// the instance set the sampler expects actually exists on this machine).
pub fn expand_wildcard(path: &str) -> Vec<String> {
    let path = wide(path);
    let mut length: u32 = 0;
    let source: *const u16 = std::ptr::null();
    let status = unsafe {
        PdhExpandWildCardPathW(source, path.as_ptr(), std::ptr::null_mut(), &mut length, 0)
    };
    if status != PDH_MORE_DATA || length == 0 {
        return Vec::new();
    }
    let mut buffer = vec![0u16; length as usize];
    let status = unsafe {
        PdhExpandWildCardPathW(source, path.as_ptr(), buffer.as_mut_ptr(), &mut length, 0)
    };
    if status != ERROR_SUCCESS {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;
    while index < buffer.len() {
        if buffer[index] == 0 {
            if index == start {
                break;
            }
            out.push(String::from_utf16_lossy(&buffer[start..index]));
            start = index + 1;
        }
        index += 1;
    }
    out
}
