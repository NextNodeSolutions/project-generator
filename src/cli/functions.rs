use inquire::{Confirm, Text};

use crate::utils::validation;

pub fn select_template(templates: Vec<(String, String)>) -> Option<(String, String)> {
    let options: Vec<String> = templates
        .iter()
        .map(|(category, name)| format!("{} ({})", name, category))
        .collect();

    let selection = inquire::Select::new("Select a template:", options).prompt();

    match selection {
        Ok(selected) => {
            // Extract the template name and category from the selection
            let parts: Vec<&str> = selected.split(" (").collect();
            if parts.len() == 2 {
                let name = parts[0].to_string();
                let category = parts[1].trim_end_matches(')').to_string();
                Some((category, name))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn prompt_for_variable(variable_name: &str) -> Option<String> {
    let prompt = format!("Enter value for {}:", variable_name);

    match variable_name {
        "project_name" => Text::new(&prompt)
            .with_validator(validation::validate_project_name)
            .prompt()
            .ok(),
        "name" => Text::new(&prompt)
            .with_validator(validation::validate_package_name)
            .prompt()
            .ok(),
        _ => Text::new(&prompt).prompt().ok(),
    }
}

pub fn prompt_for_repo_name(project_name: &str) -> Option<String> {
    println!("Project name: {}", project_name);

    let use_project_name =
        Confirm::new("Do you want to use the project name as the repository name?")
            .with_default(true)
            .prompt()
            .ok()?;

    if use_project_name {
        Some(project_name.to_string())
    } else {
        Text::new("Enter the name for the new GitHub repository:")
            .with_validator(validation::validate_project_name)
            .prompt()
            .ok()
    }
}
