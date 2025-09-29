use indexmap::IndexMap;
use serde_json::Value;

use crate::config::Replacement;
use crate::utils::context;

pub fn convert_value_to_json(value: &str, type_: &str) -> Value {
    context::debug_print(&format!("Converting value '{}' to type '{}'", value, type_));

    match type_ {
        "array" => {
            let array_values: Vec<Value> = value
                .split(',')
                .map(|s| Value::String(s.trim().to_string()))
                .collect();
            context::debug_print(&format!("Converted to array: {:?}", array_values));
            Value::Array(array_values)
        }
        _ => {
            context::debug_print(&format!("Converted to string: '{}'", value));
            Value::String(value.to_string())
        }
    }
}

pub fn create_ordered_map(
    template_json: &IndexMap<String, Value>,
    replacements: &[Replacement],
) -> IndexMap<String, Value> {
    let mut ordered_map = IndexMap::new();
    context::debug_print(&format!("Processing {} replacements", replacements.len()));

    // First, insert all existing keys in their original order
    for (key, value) in template_json.iter() {
        ordered_map.insert(key.clone(), value.clone());
    }

    // Then, insert new keys after the "name" key if it exists
    if let Some(name_pos) = ordered_map.get_index_of("name") {
        context::debug_print(&format!(
            "Found 'name' key at position {}, inserting new keys after it",
            name_pos
        ));

        // Insert new keys
        for replacement in replacements {
            let json_key = get_json_key(replacement);
            if !template_json.contains_key(&json_key) {
                if let Some(value) = context::get_variable(&replacement.name) {
                    let json_value = convert_value_to_json(&value, &replacement.type_);
                    ordered_map.insert(json_key.clone(), json_value);
                    context::debug_print(&format!(
                        "Added new key '{}' with value from variable '{}'",
                        json_key, replacement.name
                    ));
                } else {
                    context::debug_print(&format!(
                        "Warning: Variable '{}' not found for key '{}'",
                        replacement.name, json_key
                    ));
                }
            } else {
                context::debug_print(&format!(
                    "Key '{}' already exists in template, skipping",
                    json_key
                ));
            }
        }
    } else {
        context::debug_print(
            "Warning: No 'name' key found in template, new keys will be added at the end",
        );
        // Insert new keys at the end
        for replacement in replacements {
            let json_key = get_json_key(replacement);
            if !template_json.contains_key(&json_key) {
                if let Some(value) = context::get_variable(&replacement.name) {
                    let json_value = convert_value_to_json(&value, &replacement.type_);
                    ordered_map.insert(json_key.clone(), json_value);
                    context::debug_print(&format!(
                        "Added new key '{}' with value from variable '{}'",
                        json_key, replacement.name
                    ));
                } else {
                    context::debug_print(&format!(
                        "Warning: Variable '{}' not found for key '{}'",
                        replacement.name, json_key
                    ));
                }
            }
        }
    }

    ordered_map
}

fn get_json_key(replacement: &Replacement) -> String {
    // Priority: attribute > key > fallback to replacement name
    if let Some(attribute) = &replacement.attribute {
        attribute.clone()
    } else if let Some(key) = &replacement.key {
        key.clone()
    } else {
        replacement.name.clone()
    }
}

pub fn update_existing_values(
    ordered_map: &mut IndexMap<String, Value>,
    replacements: &[Replacement],
) {
    context::debug_print("Updating existing values in template");

    for replacement in replacements {
        if let Some(value) = context::get_variable(&replacement.name) {
            let json_key = get_json_key(replacement);
            if let Some(existing_value) = ordered_map.get_mut(&json_key) {
                let json_value = convert_value_to_json(&value, &replacement.type_);
                context::debug_print(&format!(
                    "Updated key '{}' from '{}' to '{}'",
                    json_key, existing_value, json_value
                ));
                *existing_value = json_value;
            } else {
                context::debug_print(&format!(
                    "Warning: Key '{}' not found in template for replacement",
                    json_key
                ));
            }
        } else {
            context::debug_print(&format!(
                "Warning: Variable '{}' not found for replacement of key '{}'",
                replacement.name, get_json_key(replacement)
            ));
        }
    }
}
