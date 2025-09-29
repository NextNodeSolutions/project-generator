use crate::utils::context;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

#[derive(serde::Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub project_name: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub template_category: Option<String>,
    #[serde(default)]
    pub template_name: Option<String>,
    #[serde(default)]
    pub template_branch: Option<String>,
    #[serde(default)]
    pub github_tag: Option<String>,
    #[serde(default)]
    pub create_develop_branch: Option<bool>,
    #[serde(flatten)]
    pub additional_vars: std::collections::HashMap<String, serde_json::Value>,
}

impl FileConfig {
    pub fn get_template_info(&self) -> Option<(String, String)> {
        match (&self.template_category, &self.template_name) {
            (Some(category), Some(name)) => {
                context::debug_print(&format!(
                    "Template info: category='{}', name='{}'",
                    category, name
                ));
                Some((category.clone(), name.clone()))
            }
            _ => {
                context::debug_print("Template info not found: missing category or name");
                None
            }
        }
    }

    pub fn get_template_branch(&self) -> &str {
        self.template_branch.as_deref().unwrap_or("main")
    }

    pub fn get_github_tag(&self) -> Option<&String> {
        self.github_tag.as_ref()
    }

    pub fn validate_github_tag(&self) -> Result<()> {
        if let Some(tag) = &self.github_tag {
            let valid_tags = ["apps", "packages", "utils"];
            if !valid_tags.contains(&tag.as_str()) {
                let error_msg = format!(
                    "Invalid github_tag '{}'. Allowed values are: {}",
                    tag,
                    valid_tags.join(", ")
                );
                context::debug_print(&format!("ERROR: {}", error_msg));
                return Err(Error::new(ErrorKind::InvalidData, error_msg));
            }
            context::debug_print(&format!("Valid github_tag found: '{}'", tag));
        }
        Ok(())
    }

    pub fn should_create_develop_branch(&self) -> bool {
        self.create_develop_branch.unwrap_or(false)
    }

    pub fn to_variables(&self) -> std::collections::HashMap<String, String> {
        context::debug_print("Converting config to variables");
        context::debug_print(&format!("Project name: '{}'", self.project_name));
        context::debug_print(&format!("Package name: '{}'", self.name));
        context::debug_print(&format!("Additional variables: {:?}", self.additional_vars));

        // Convert serde_json::Value to String for additional variables
        let mut vars: HashMap<String, String> = self
            .additional_vars
            .iter()
            .map(|(k, v)| {
                let value_str = match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Array(arr) => {
                        // Convert array to comma-separated string for template replacement
                        arr.iter()
                            .map(|item| match item {
                                serde_json::Value::String(s) => s.clone(),
                                _ => item.to_string().trim_matches('"').to_string(),
                            })
                            .collect::<Vec<String>>()
                            .join(",")
                    },
                    _ => v.to_string().trim_matches('"').to_string(),
                };
                (k.clone(), value_str)
            })
            .collect();

        // Add required variables in specific order
        vars.insert("project_name".to_string(), self.project_name.clone());
        vars.insert("name".to_string(), self.name.clone());

        // Sort variables according to template_config.json
        let mut sorted_vars = HashMap::new();

        // First add required variables in specific order
        if let Some(project_name) = vars.remove("project_name") {
            sorted_vars.insert("project_name".to_string(), project_name);
        }
        if let Some(name) = vars.remove("name") {
            sorted_vars.insert("name".to_string(), name);
        }

        // Then add remaining variables in alphabetical order
        let mut remaining_keys: Vec<_> = vars.keys().collect();
        remaining_keys.sort();
        for key in remaining_keys {
            if let Some(value) = vars.get(key) {
                sorted_vars.insert(key.clone(), value.clone());
            }
        }

        context::debug_print(&format!("Final variables: {:?}", sorted_vars));
        sorted_vars
    }
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<FileConfig> {
    let path_ref = path.as_ref();
    context::debug_print(&format!("Reading config file: {}", path_ref.display()));

    let content = fs::read_to_string(path_ref).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to read config file: {}", e),
        )
    })?;

    context::debug_print(&format!("Config file size: {} bytes", content.len()));

    // Check if file is empty
    if content.trim().is_empty() {
        context::debug_print("ERROR: Configuration file is empty");
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Configuration file is empty. Please add configuration content.",
        ));
    }

    // Try YAML first, then JSON
    context::debug_print("Attempting YAML parsing");
    if let Ok(config) = serde_yaml::from_str::<FileConfig>(&content) {
        context::debug_print("Successfully parsed YAML config");
        Ok(config)
    } else {
        context::debug_print("YAML parsing failed, attempting JSON parsing");
        let config: FileConfig = serde_json::from_str(&content).map_err(|e| {
            let error_msg = if content.trim().is_empty() {
                "Configuration file is empty".to_string()
            } else {
                format!("Failed to parse config file (neither YAML nor JSON): {}", e)
            };
            context::debug_print(&format!("ERROR: {}", error_msg));
            Error::new(ErrorKind::InvalidData, error_msg)
        })?;
        context::debug_print("Successfully parsed JSON config");
        Ok(config)
    }
}
