mod args;
mod cli;
mod config;
mod generate;
mod github;
mod template;
mod utils;

use args::Args;
use clap::Parser;
use cli::{get_template_info, prompt_for_repo_name};
use generate::{handle_config_mode, handle_interactive_mode};
use github::{create_github_repository_with_code, extract_organization_from_repo_url};
use std::io::{Error, ErrorKind, Result};
use template::TemplateManager;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Set debug mode in the global context
    utils::context::set_debug_mode(args.debug);

    // Get template branch from config if available
    let template_branch = if let Some(config_path) = &args.config {
        let config = crate::config::file_config::from_file(config_path).ok();
        config
            .as_ref()
            .map(|c| c.get_template_branch())
            .map(|s| s.to_string())
    } else {
        None
    };

    // Initialize template manager and clone the repository
    let template_manager = TemplateManager::new(template_branch.as_deref()).unwrap_or_else(|err| {
        utils::error::print_error_and_exit_with_error("Failed to initialize template manager", &err)
    });

    // Get template info and path
    let (category, template_name) = get_template_info(&args, &template_manager)?;
    let template_path = template_manager.get_template_path(&category, &template_name);

    // Handle local generation first (early return)
    if !args.remote {
        // Handle generation based on mode
        if args.config.is_none() {
            return handle_interactive_mode(&template_path)
                .map_err(|e| Error::other(e.to_string()));
        }

        // Get project name from variables
        let project_name = utils::context::get_variable("project_name").unwrap_or_else(|| {
            utils::error::print_error_and_exit("project_name is required in configuration file")
        });

        return handle_config_mode(&template_path, &project_name)
            .map_err(|e| Error::other(e.to_string()));
    }

    // Remote mode: generate project locally, then create GitHub repo
    let token = args
        .token
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidInput,
                "GitHub token is required for remote mode. Set GITHUB_TOKEN env var or use --token",
            )
        })?;

    // Config file is required for remote mode - check early
    let config_path = args.config.as_ref().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "Config file is required for remote mode. Use --config to specify a config file.",
        )
    })?;

    // Read and parse config file early to get project name and validate github_tag
    let file_config = crate::config::file_config::from_file(config_path).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to read config file: {}", e),
        )
    })?;

    // Validate github_tag early (before pulling code)
    file_config.validate_github_tag().map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("GitHub tag validation failed: {}", e),
        )
    })?;

    let project_name = file_config.project_name.clone();
    if project_name.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "project_name is required in config file",
        ));
    }

    // Get organization from REPO_URL
    let organization = extract_organization_from_repo_url()?;
    println!("Using organization: {}", organization);

    // Ask for repository name with option to use project name
    let repo_name = prompt_for_repo_name(&project_name)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Repository name is required"))?;

    // Create temporary directory for remote mode
    let temp_dir = std::env::temp_dir().join(format!("project-generator-{}", project_name));
    let project_path = temp_dir;

    // Generate the project in temp directory
    if args.config.is_some() {
        crate::generate::handle_config_mode_with_path(
            &template_path,
            &project_name,
            &project_path,
            false,
        )
        .map_err(|e| Error::other(e.to_string()))?;
    } else {
        handle_interactive_mode(&template_path)
            .map_err(|e| Error::other(e.to_string()))?;
    }

    // Install dependencies AFTER copying template files but BEFORE Git operations
    crate::generate::project_generator::install_dependencies(&project_path).map_err(|e| {
        Error::other(
            format!("Failed to install dependencies: {}", e),
        )
    })?;

    // Get description from config or use default
    let description = file_config
        .additional_vars
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("Generated project")
        .to_string();

    // Create GitHub repository and push the code (includes full Git workflow)
    let github_tag = file_config.get_github_tag().map(|s| s.as_str());
    let create_develop = file_config.should_create_develop_branch();
    let result = create_github_repository_with_code(
        &token,
        &repo_name,
        &project_path,
        description.as_str(),
        github_tag,
        create_develop,
    )
    .await;

    // Clean up temporary directory
    if let Err(e) = std::fs::remove_dir_all(&project_path) {
        eprintln!(
            "Warning: Failed to clean up temporary directory '{}': {}",
            project_path.display(),
            e
        );
    } else {
        println!("Temporary directory cleaned up successfully");
    }

    result?;
    Ok(())
}
