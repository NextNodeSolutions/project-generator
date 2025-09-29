use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

mod functions;

use crate::args::Args;
use crate::config::file_config;
use crate::config::{CREATION_PATH, PACKAGE_ROOT_PATH};
use crate::generate::project_generator;
use crate::template::TemplateManager;
use crate::utils::{context, strings};

pub use functions::prompt_for_repo_name;

pub fn get_template_info(
    args: &Args,
    template_manager: &TemplateManager,
) -> Result<(String, String)> {
    if let Some(config_path) = &args.config {
        // Try to get template info from config file
        let config = file_config::from_file(config_path).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to read config file: {}", e),
            )
        })?;

        // Set variables from config
        context::set_variables(config.to_variables());

        // Get template info from config
        config.get_template_info().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "template_category and template_name are required in configuration file",
            )
        })
    } else if let (Some(cat), Some(tmpl)) = (&args.category, &args.template) {
        Ok((cat.clone(), tmpl.clone()))
    } else {
        // List available templates
        let templates = template_manager.list_templates().map_err(|e| {
            Error::other(format!("Failed to list templates: {}", e))
        })?;

        // Select template
        functions::select_template(templates)
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Failed to select template"))
    }
}

pub fn interact(template_path: &Path) -> Result<()> {
    // Get project name first
    let project_name = functions::prompt_for_variable("project_name").ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "An error occurred while entering project name",
        )
    })?;

    // Get package name
    let package_name = functions::prompt_for_variable("name").ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "An error occurred while entering package name",
        )
    })?;

    // Initialize variables with both names
    let mut variables = HashMap::from([
        ("project_name".to_string(), project_name.to_string()),
        ("name".to_string(), package_name.to_string()),
    ]);

    // Try to get additional variables from template config
    match strings::extract_unique_keys(template_path) {
        Ok(unique_keys) => {
            for key in &unique_keys {
                if key != "project_name" && key != "name" {
                    let value = functions::prompt_for_variable(key).ok_or_else(|| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("An error occurred while entering {}", key),
                        )
                    })?;
                    variables.insert(key.to_string(), value);
                }
            }
        }
        Err(err) => {
            context::debug_print(&format!("Note: No template configuration found: {}", err));
            println!("Note: No template configuration found. Using basic template generation.");
        }
    }

    context::set_variables(variables);

    let project_path = Path::new(&PACKAGE_ROOT_PATH)
        .join(CREATION_PATH)
        .join(&project_name);

    println!(
        "Generating project '{}' with template '{}'",
        project_name,
        template_path.file_name().unwrap().to_string_lossy()
    );

    project_generator::generate_project(template_path, &project_path).map_err(|e| {
        Error::other(
            format!("An error occurred while generating the project: {}", e),
        )
    })?;

    project_generator::install_dependencies(&project_path).map_err(|e| {
        Error::other(
            format!("An error occurred while installing dependencies: {}", e),
        )
    })?;

    println!("Project generated successfully");
    Ok(())
}
