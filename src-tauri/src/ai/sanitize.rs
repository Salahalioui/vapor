use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedProcessMetadata {
    pub process_name: String,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub path_category: String,
    pub cpu_percent: f32,
    pub memory_mb: f32,
}

pub fn sanitize_process_info(
    process_name: &str,
    exe_path: Option<&str>,
    publisher: Option<&str>,
    description: Option<&str>,
    cpu_percent: f32,
    memory_mb: f32,
) -> SanitizedProcessMetadata {
    let clean_name = Path::new(process_name)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| process_name.to_string());

    let path_category = exe_path
        .map(categorize_and_strip_path)
        .unwrap_or_else(|| "Unknown / In-Memory".to_string());

    let clean_publisher = publisher.map(|p| sanitize_text(p));
    let clean_desc = description.map(|d| sanitize_text(d));

    SanitizedProcessMetadata {
        process_name: clean_name,
        publisher: clean_publisher,
        description: clean_desc,
        path_category,
        cpu_percent: (cpu_percent * 10.0).round() / 10.0,
        memory_mb: (memory_mb * 10.0).round() / 10.0,
    }
}

pub fn categorize_and_strip_path(raw_path: &str) -> String {
    let normalized = raw_path.replace('/', "\\");
    let lower = normalized.to_lowercase();

    if lower.contains("\\windows\\system32") {
        "System32 (Windows Core)".to_string()
    } else if lower.contains("\\windows\\syswow64") {
        "SysWOW64 (32-bit Windows subsystem)".to_string()
    } else if lower.contains("\\windows\\") {
        "Windows Directory".to_string()
    } else if lower.contains("\\program files (x86)\\") {
        extract_program_folder(&normalized, "Program Files (x86)")
    } else if lower.contains("\\program files\\") {
        extract_program_folder(&normalized, "Program Files")
    } else if lower.contains("\\appdata\\local\\programs\\") {
        extract_sub_path(&normalized, "\\appdata\\local\\programs\\", "AppData/Local/Programs")
    } else if lower.contains("\\appdata\\local\\") {
        extract_sub_path(&normalized, "\\appdata\\local\\", "AppData/Local")
    } else if lower.contains("\\appdata\\roaming\\") {
        extract_sub_path(&normalized, "\\appdata\\roaming\\", "AppData/Roaming")
    } else if lower.contains("\\users\\") {
        "User Directory (Scratch / Temp)".to_string()
    } else {
        "Non-Standard Path".to_string()
    }
}

fn extract_program_folder(path: &str, marker: &str) -> String {
    let lower_path = path.to_lowercase();
    let lower_marker = marker.to_lowercase();
    if let Some(pos) = lower_path.find(&lower_marker) {
        let remainder = &path[pos + marker.len()..];
        let segments: Vec<&str> = remainder
            .trim_start_matches('\\')
            .split('\\')
            .filter(|s| !s.is_empty())
            .collect();
        if !segments.is_empty() {
            return format!("{}/{}", marker, segments[0]);
        }
    }
    marker.to_string()
}

fn extract_sub_path(path: &str, needle: &str, prefix: &str) -> String {
    let lower_path = path.to_lowercase();
    if let Some(pos) = lower_path.find(needle) {
        let remainder = &path[pos + needle.len()..];
        let segments: Vec<&str> = remainder
            .trim_start_matches('\\')
            .split('\\')
            .filter(|s| !s.is_empty())
            .collect();
        if !segments.is_empty() {
            return format!("{}/{}", prefix, segments[0]);
        }
    }
    prefix.to_string()
}

pub fn sanitize_text(text: &str) -> String {
    let mut scrubbed = text.to_string();

    // Remove username if present in USERNAME env var
    if let Ok(user) = env::var("USERNAME") {
        if !user.is_empty() {
            scrubbed = scrubbed.replace(&user, "[USER]");
        }
    }
    if let Ok(user) = env::var("USERPROFILE") {
        if !user.is_empty() {
            scrubbed = scrubbed.replace(&user, "%USERPROFILE%");
        }
    }

    scrubbed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_sanitization_removes_user_name() {
        let raw = "C:\\Users\\JohnDoeSecret\\AppData\\Local\\Programs\\Figma\\Figma.exe";
        let categorized = categorize_and_strip_path(raw);
        assert!(!categorized.contains("JohnDoeSecret"));
        assert_eq!(categorized, "AppData/Local/Programs/Figma");
    }

    #[test]
    fn test_system32_sanitization() {
        let raw = "C:\\Windows\\System32\\svchost.exe";
        let categorized = categorize_and_strip_path(raw);
        assert_eq!(categorized, "System32 (Windows Core)");
    }

    #[test]
    fn test_program_files_sanitization() {
        let raw = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
        let categorized = categorize_and_strip_path(raw);
        assert_eq!(categorized, "Program Files/Google");
    }
}
