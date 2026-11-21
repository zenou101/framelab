//! PresentMon CSV parsing.
//!
//! Column order differs between PresentMon versions (1.x vs 2.x) and casing is
//! not stable either, so every field is resolved through a case-insensitive
//! header map. `NA` and empty cells become `None` - never zero.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ParsedFrame {
    pub pid: Option<u32>,
    pub presented_ms: Option<f64>,
    pub displayed_ms: Option<f64>,
    pub present_mode: Option<String>,
    pub runtime: Option<String>,
    pub sync_interval: Option<i64>,
}

pub struct CsvParser {
    index: HashMap<String, usize>,
}

fn split_fields(line: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let bytes = line.as_bytes();
    let mut start = 0usize;
    let mut in_quotes = false;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => in_quotes = !in_quotes,
            b',' if !in_quotes => {
                fields.push(trim_quotes(&line[start..index]));
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    fields.push(trim_quotes(&line[start..]));
    fields
}

fn trim_quotes(value: &str) -> &str {
    value.trim().trim_matches('"')
}

fn parse_f64(value: &str) -> Option<f64> {
    if value.is_empty() || value.eq_ignore_ascii_case("NA") {
        return None;
    }
    value.parse::<f64>().ok().filter(|v| v.is_finite() && *v >= 0.0)
}

impl CsvParser {
    pub fn header(line: &str) -> Option<Self> {
        let fields = split_fields(line);
        if fields.len() < 3 {
            return None;
        }
        let mut index = HashMap::new();
        for (position, field) in fields.iter().enumerate() {
            index.insert(field.to_ascii_lowercase(), position);
        }
        if !index.contains_key("application") && !index.contains_key("processid") {
            return None;
        }
        Some(Self { index })
    }

    fn get<'a>(&self, fields: &[&'a str], name: &str) -> Option<&'a str> {
        let position = *self.index.get(name)?;
        fields.get(position).copied()
    }

    pub fn parse(&self, line: &str) -> Option<ParsedFrame> {
        if line.trim().is_empty() {
            return None;
        }
        let fields = split_fields(line);
        if fields.len() < self.index.len().min(4) {
            return None;
        }
        Some(ParsedFrame {
            pid: self.get(&fields, "processid").and_then(|v| v.parse::<u32>().ok()),
            presented_ms: self.get(&fields, "msbetweenpresents").and_then(parse_f64),
            displayed_ms: self.get(&fields, "msbetweendisplaychange").and_then(parse_f64),
            present_mode: self.get(&fields, "presentmode").map(|v| v.to_string()).filter(|v| !v.is_empty()),
            runtime: self
                .get(&fields, "presentruntime")
                .or_else(|| self.get(&fields, "runtime"))
                .map(|v| v.to_string())
                .filter(|v| !v.is_empty()),
            sync_interval: self.get(&fields, "syncinterval").and_then(|v| v.parse::<i64>().ok()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER_V2: &str = "Application,ProcessID,SwapChainAddress,PresentRuntime,SyncInterval,PresentFlags,AllowsTearing,PresentMode,CPUStartTime,MsBetweenPresents,MsBetweenDisplayChange,MsUntilDisplayed";

    #[test]
    fn resolves_columns_by_name_not_position() {
        let parser = CsvParser::header(HEADER_V2).expect("header");
        // Columns deliberately reordered relative to the header.
        let line = "game.exe,4242,0x1,DirectX 12,1,0,1,Hardware: Independent Flip,12.5,16.7,16.6,3.2";
        let frame = parser.parse(line).expect("frame");
        assert_eq!(frame.pid, Some(4242));
        assert!((frame.presented_ms.unwrap() - 16.7).abs() < 1e-9);
        assert!((frame.displayed_ms.unwrap() - 16.6).abs() < 1e-9);
        assert_eq!(frame.runtime.as_deref(), Some("DirectX 12"));
        assert_eq!(frame.sync_interval, Some(1));
    }

    #[test]
    fn na_and_empty_cells_are_none() {
        let parser = CsvParser::header(HEADER_V2).expect("header");
        let line = "game.exe,1,0x1,DirectX 12,1,0,1,Composed: Flip,1.0,NA,,";
        let frame = parser.parse(line).expect("frame");
        assert!(frame.presented_ms.is_none());
        assert!(frame.displayed_ms.is_none());
    }

    #[test]
    fn quoted_fields_with_commas_survive() {
        let header = "\"Application\",\"ProcessID\",\"MsBetweenPresents\"";
        let parser = CsvParser::header(header).expect("header");
        let frame = parser.parse("\"my, game.exe\",77,8.5").expect("frame");
        assert_eq!(frame.pid, Some(77));
        assert!(frame.presented_ms.is_some());
    }

    #[test]
    fn rejects_non_csv_noise() {
        assert!(CsvParser::header("PresentMon output").is_none());
    }
}
