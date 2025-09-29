pub mod file_config;

pub const PACKAGE_ROOT_PATH: &str = env!("CARGO_MANIFEST_DIR");
pub const CREATION_PATH: &str = "../";
pub const REPO_URL: &str = "https://github.com/NextNodeSolutions";

pub const TEMPLATE_REPO_URL: &str = "/utils_project-templates.git";
pub const TEMPLATE_BRANCH: &str = "main";
pub const TEMPLATE_CATEGORIES: &[&str] = &["apps", "packages", "utils"];

pub const TEMPLATE_CONFIG_FILE: &str = "template_config.json";
pub const EXCLUDED_DIRS: &[&str] = &["node_modules", ".next", ".turbo", "dist", "build", "out"];
pub const EXCLUDED_FILES: &[&str] = &[TEMPLATE_CONFIG_FILE];

#[derive(Debug, serde::Deserialize)]
pub struct Replacement {
    pub name: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub attribute: Option<String>,
    #[serde(rename = "type", default = "default_type")]
    pub type_: String,
    #[serde(default)]
    pub default: Option<String>,
}

fn default_type() -> String {
    "string".to_string()
}

#[derive(serde::Deserialize)]
pub struct TemplateConfig {
    pub files_to_replace: Vec<String>,
    pub replacements: Vec<Replacement>,
}

pub type TemplateJson = Vec<TemplateConfig>;
