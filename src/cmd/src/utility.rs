use anyhow::Result;
use colored::Colorize;
use std::fs::File;
use std::io::{BufWriter, Write};

pub fn highlight_pattern(pattern: &str, candidates: Vec<u8>) -> Vec<u8> {
    if pattern.is_empty() {
        return candidates;
    }
    let text = match str::from_utf8(&candidates) {
        Ok(text) => text,
        Err(_) => return candidates,
    };
    let pattern_len = pattern.len();
    let mut last_index = 0;
    let mut result = Vec::with_capacity(candidates.len());
    for (index, _) in text.match_indices(pattern) {
        result.extend(text[last_index..index].as_bytes());
        let highlighted = format!(
            "{}",
            String::from_utf8_lossy(&text[index..index + pattern_len].as_bytes()).bright_magenta()
        )
        .into_bytes();
        result.extend(highlighted);
        last_index = index + pattern_len;
    }
    result.extend(text[last_index..].as_bytes());
    result
}

pub fn write_output_to_file<T: Iterator<Item = (Vec<u8>, Vec<u8>)>>(key_values: T, file_path: &str) -> Result<()> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);
    for (key, value) in key_values {
        let key_str = String::from_utf8_lossy(&key);
        let val_str = String::from_utf8_lossy(&value);
        writeln!(writer, "{}: {}", key_str, val_str).unwrap();
    }
    Ok(())
}
