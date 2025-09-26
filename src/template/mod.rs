use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{REPO_URL, TEMPLATE_BRANCH, TEMPLATE_CATEGORIES, TEMPLATE_REPO_URL};

pub struct TemplateManager {
    repo_path: PathBuf,
}

impl TemplateManager {
    pub fn new(branch: Option<&str>) -> std::io::Result<Self> {
        // Create a unique temporary directory using timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let temp_dir = std::env::temp_dir().join(format!("project-templates-{}", timestamp));

        fs::create_dir_all(&temp_dir)?;
        let repo_path = temp_dir;

        // Configure Git to use HTTPS with PAT
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            // Get the PAT from environment variable
            let pat = std::env::var("GITHUB_TOKEN").map_err(|_| {
                git2::Error::new(
                    git2::ErrorCode::Auth,
                    git2::ErrorClass::Http,
                    "GITHUB_TOKEN environment variable not set",
                )
            })?;

            git2::Cred::userpass_plaintext(username_from_url.unwrap_or("git"), &pat)
        });

        // Set up fetch options with the callbacks
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Clone the repository
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        // Use provided branch or fallback to TEMPLATE_BRANCH constant
        let branch_to_use = branch.unwrap_or(TEMPLATE_BRANCH);
        builder.branch(branch_to_use);

        builder
            .clone(
                format!("{}{}", REPO_URL, TEMPLATE_REPO_URL).as_str(),
                &repo_path,
            )
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to clone repository: {}", e),
                )
            })?;

        Ok(Self { repo_path })
    }

    pub fn list_templates(&self) -> std::io::Result<Vec<(String, String)>> {
        let mut templates = Vec::new();

        for category in TEMPLATE_CATEGORIES {
            let category_path = self.repo_path.join(category);
            if !category_path.exists() {
                continue;
            }

            for entry in fs::read_dir(category_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        templates.push((category.to_string(), name.to_string()));
                    }
                }
            }
        }

        templates.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        Ok(templates)
    }

    pub fn get_template_path(&self, category: &str, template: &str) -> PathBuf {
        self.repo_path.join(category).join(template)
    }
}

impl Drop for TemplateManager {
    fn drop(&mut self) {
        // The temp directory will be automatically cleaned up
    }
}
