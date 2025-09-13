use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;

use crate::config::TemplateJson;
use crate::generate::file_operations;
use crate::utils::context;
use crate::utils::strings;

pub fn generate_project(template_path: &Path, project_path: &Path) -> std::io::Result<()> {
    context::debug_print(&format!("Starting project generation"));
    context::debug_print(&format!("Template path: {}", template_path.display()));
    context::debug_print(&format!("Project path: {}", project_path.display()));

    if !template_path.exists() {
        context::debug_print(&format!(
            "ERROR: Template not found at {}",
            template_path.display()
        ));
        return Err(Error::new(ErrorKind::NotFound, "Template not found"));
    }

    context::debug_print("Creating project directory");
    fs::create_dir_all(&project_path)?;

    context::debug_print("Copying template files");
    file_operations::copy_dir_all(&template_path, &project_path)?;

    println!(
        "Project '{}' copied from template '{}' successfully",
        project_path.file_name().unwrap().to_string_lossy(),
        template_path.file_name().unwrap().to_string_lossy()
    );

    context::debug_print("Reading template configuration");
    let config = strings::read_template_config(template_path)?;
    context::debug_print(&format!("Found {} template configurations", config.len()));

    context::debug_print("Applying template configuration");
    apply_template_config(&project_path, &config)?;

    context::debug_print("Project generation completed successfully");
    Ok(())
}

fn apply_template_config(project_path: &Path, config: &TemplateJson) -> std::io::Result<()> {
    context::debug_print(&format!(
        "Applying {} template configurations",
        config.len()
    ));

    for (i, file) in config.iter().enumerate() {
        context::debug_print(&format!(
            "Processing configuration {}: {} files to replace",
            i + 1,
            file.files_to_replace.len()
        ));

        for file_to_replace in &file.files_to_replace {
            let file_path = project_path.join(&file_to_replace);
            context::debug_print(&format!("Processing file: {}", file_path.display()));

            if let Err(e) = file_operations::replace_in_file(&file_path, &file.replacements) {
                context::debug_print(&format!("ERROR updating file {}: {}", file_to_replace, e));
                println!("Error updating file {}: {}", file_to_replace, e);
            } else {
                context::debug_print(&format!("Successfully updated file: {}", file_to_replace));
            }
        }
    }
    Ok(())
}

pub fn install_dependencies(project_path: &Path) -> std::io::Result<()> {
    context::debug_print(&format!(
        "Installing dependencies in: {}",
        project_path.display()
    ));

    let status = Command::new("pnpm")
        .arg("install")
        .current_dir(project_path)
        .status()?;

    if !status.success() {
        context::debug_print(&format!(
            "ERROR: pnpm install failed with status: {}",
            status
        ));
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to install dependencies",
        ));
    }

    context::debug_print("Dependencies installed successfully");
    Ok(())
}
