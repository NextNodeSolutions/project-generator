use std::io::{self, Error, ErrorKind, Result, Write};
use std::path::{Path, PathBuf};

pub mod file_operations;
pub mod functions;
pub mod project_generator;

pub fn handle_interactive_mode(template_path: &Path) -> Result<()> {
    match crate::cli::interact(template_path) {
        Ok(_) => {
            println!("Project generated successfully");
            Ok(())
        }
        Err(e) => Err(Error::other(e.to_string())),
    }
}

pub fn handle_config_mode(template_path: &Path, project_name: &str) -> Result<()> {
    let default_project_path = std::path::Path::new(&crate::config::PACKAGE_ROOT_PATH)
        .join(crate::config::CREATION_PATH)
        .join(project_name);

    // Display the default path and ask for confirmation
    println!(
        "Project will be created in: {}",
        default_project_path.display()
    );
    print!("Is this path correct? (Y/n): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        Error::other(
            format!("Failed to read user input: {}", e),
        )
    })?;

    let input = input.trim().to_lowercase();
    let use_default = input.is_empty() || input == "y" || input == "yes";

    let project_path = if use_default {
        default_project_path
    } else {
        // Ask for custom path
        print!("Please enter the custom path: ");
        io::stdout().flush().unwrap();

        let mut custom_path = String::new();
        io::stdin().read_line(&mut custom_path).map_err(|e| {
            Error::other(
                format!("Failed to read custom path: {}", e),
            )
        })?;

        let custom_path = custom_path.trim();
        if custom_path.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Path cannot be empty"));
        }

        PathBuf::from(custom_path).join(project_name)
    };

    handle_config_mode_with_path(template_path, project_name, &project_path, true)
}

pub fn handle_config_mode_with_path(
    template_path: &Path,
    project_name: &str,
    project_path: &Path,
    install_deps: bool,
) -> Result<()> {
    println!(
        "Generating project '{}' with template '{}'",
        project_name,
        template_path.display()
    );

    project_generator::generate_project(template_path, project_path).map_err(|e| {
        Error::other(
            format!("An error occurred while generating the project: {}", e),
        )
    })?;

    if install_deps {
        project_generator::install_dependencies(project_path).map_err(|e| {
            Error::other(
                format!("An error occurred while installing dependencies: {}", e),
            )
        })?;
    }

    println!("Project generated successfully");
    Ok(())
}
