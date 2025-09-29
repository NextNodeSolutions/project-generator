use crate::config::{Replacement, EXCLUDED_DIRS, EXCLUDED_FILES};

use indexmap::IndexMap;
use serde_json::{self, Value};
use std::path::Path;
use std::{fs, io};

use super::functions;
use crate::utils::context;

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    context::debug_print(&format!(
        "Copying directory from '{}' to '{}'",
        src.display(),
        dst.display()
    ));
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if EXCLUDED_DIRS.contains(&file_name_str.as_ref()) {
            context::debug_print(&format!("Skipping excluded directory: {}", file_name_str));
            continue;
        }

        if EXCLUDED_FILES.contains(&file_name_str.as_ref()) {
            context::debug_print(&format!("Skipping excluded file: {}", file_name_str));
            continue;
        }

        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            context::debug_print(&format!("Copying subdirectory: {}", file_name_str));
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            context::debug_print(&format!("Copying file: {}", file_name_str));
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn replace_in_file(file_path: &Path, replacements: &[Replacement]) -> io::Result<()> {
    context::debug_print(&format!("Processing file: {}", file_path.display()));
    context::debug_print(&format!(
        "Found {} replacements to apply",
        replacements.len()
    ));

    let content = fs::read_to_string(file_path)?;

    if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
        context::debug_print("Detected JSON file, using JSON replacement logic");
        replace_in_json_file(file_path, &content, replacements)
    } else {
        context::debug_print("Using text replacement logic");
        replace_in_text_file(file_path, &content, replacements)
    }
}

fn write_json_to_file(file_path: &Path, ordered_map: IndexMap<String, Value>) -> io::Result<()> {
    context::debug_print(&format!("Writing JSON file: {}", file_path.display()));
    context::debug_print(&format!("JSON contains {} keys", ordered_map.len()));

    let json_str = serde_json::to_string_pretty(&ordered_map)?;
    fs::write(file_path, json_str)
}

fn replace_in_json_file(
    file_path: &Path,
    content: &str,
    replacements: &[Replacement],
) -> io::Result<()> {
    context::debug_print("Parsing JSON content");
    let template_json: IndexMap<String, Value> = serde_json::from_str(content)?;
    context::debug_print(&format!(
        "Template JSON contains {} keys",
        template_json.len()
    ));

    let mut ordered_map = functions::create_ordered_map(&template_json, replacements);
    functions::update_existing_values(&mut ordered_map, replacements);
    write_json_to_file(file_path, ordered_map)
}

fn replace_in_text_file(
    file_path: &Path,
    content: &str,
    replacements: &[Replacement],
) -> io::Result<()> {
    context::debug_print("Applying text replacements");
    let mut new_content = content.to_string();

    for replacement in replacements {
        // Try to get variable, fallback to default if not found
        let value = crate::utils::context::get_variable(&replacement.name)
            .or_else(|| replacement.default.clone());

        if let Some(value) = value {
            // For non-JSON files, use raw string values to avoid JSON quotes
            let formatted_value = match replacement.type_.as_str() {
                "array" => {
                    let json_value = functions::convert_value_to_json(&value, &replacement.type_);
                    serde_json::to_string(&json_value).unwrap_or(value)
                }
                _ => value, // Use raw string value for non-JSON files
            };

            let old_content = new_content.clone();

            // Determine replacement pattern
            let pattern = if let Some(key) = &replacement.key {
                key.clone()
            } else {
                format!("{{{{{}}}}}", replacement.name)
            };

            new_content = new_content.replace(&pattern, &formatted_value);

            if old_content != new_content {
                let source = if crate::utils::context::get_variable(&replacement.name).is_some() {
                    "variable"
                } else {
                    "default"
                };
                context::debug_print(&format!(
                    "Applied replacement for '{}' (pattern: '{}') with {} value '{}'",
                    replacement.name, pattern, source, formatted_value
                ));
            } else {
                context::debug_print(&format!(
                    "No matches found for replacement '{}' (pattern: '{}')",
                    replacement.name, pattern
                ));
            }
        } else {
            context::debug_print(&format!(
                "Warning: Variable '{}' not found and no default value provided",
                replacement.name
            ));
        }
    }

    fs::write(file_path, new_content)
}
