use crate::config::REPO_URL;
use git2::{Cred, RemoteCallbacks, Repository, Signature};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_json::json;
use std::path::Path;

pub struct GitHubRepo {
    token: String,
}

impl GitHubRepo {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    pub async fn create_repository(
        &self,
        name: &str,
        description: &str,
        private: bool,
        topic: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Extract organization from REPO_URL constant
        // REPO_URL = "https://github.com/NextNodeSolutions"
        let org_name = REPO_URL
            .split('/')
            .last()
            .ok_or("Could not extract organization from REPO_URL")?;

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .map_err(|_| "Failed to create authorization header")?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_str("application/vnd.github.v3+json")
                .map_err(|_| "Failed to create accept header")?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str("NextNode-Project-Generator/1.0")
                .map_err(|_| "Failed to create user-agent header")?,
        );

        // Build request body
        let body = json!({
            "name": name,
            "description": description,
            "private": private,
            "auto_init": false
        });

        // Make GitHub API call to create repository
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("https://api.github.com/orgs/{}/repos", org_name))
            .headers(headers.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to GitHub API: {}", e))?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error: {}", error).into());
        }

        let repo_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let repo_url = repo_data["html_url"]
            .as_str()
            .ok_or("No html_url in response")?
            .to_string();

        // Add topic if provided
        if let Some(topic_name) = topic {
            println!("Adding topic '{}' to repository...", topic_name);

            let topics_body = json!({
                "names": [topic_name]
            });

            let topics_response = client
                .put(&format!(
                    "https://api.github.com/repos/{}/{}/topics",
                    org_name, name
                ))
                .headers(headers)
                .json(&topics_body)
                .send()
                .await
                .map_err(|e| format!("Failed to add topic: {}", e))?;

            if !topics_response.status().is_success() {
                let error = topics_response
                    .text()
                    .await
                    .map_err(|e| format!("Failed to read topics error response: {}", e))?;
                // Don't fail the entire operation for topic addition failure, just warn
                eprintln!("Warning: Failed to add topic '{}': {}", topic_name, error);
            } else {
                println!("Successfully added topic '{}' to repository", topic_name);
            }
        }

        Ok(repo_url)
    }

    pub fn initialize_git_and_push(
        &self,
        local_path: &Path,
        repo_url: &str,
        author_name: &str,
        author_email: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Remove existing .git directory if it exists
        let git_dir = local_path.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir)?;
        }

        // 1. git init
        let repo = Repository::init(local_path)?;

        // 2. git branch -M main (la branche main est créée par défaut avec git2)
        // Note: git2 crée automatiquement la branche main lors du premier commit

        // 3. À ce stade, pnpm install a déjà été fait avant d'appeler cette fonction

        // 4. git add .
        let mut index = repo.index()?;
        index.add_all(["*"], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // 5. git commit -m "first commit"
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = Signature::now(author_name, author_email)?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "first commit",
            &tree,
            &[],
        )?;

        // 6. git remote add origin <url>
        let mut remote = repo.remote("origin", repo_url)?;

        // 7. git push -u origin main (utiliser HEAD pour éviter les problèmes de référence)
        let mut callbacks = RemoteCallbacks::new();
        let token = self.token.clone();
        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            Cred::userpass_plaintext(username_from_url.unwrap_or("git"), &token)
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);
        remote.push(&["HEAD:refs/heads/main"], Some(&mut push_options))?;

        Ok(())
    }

    pub async fn trigger_workflow_dispatch(
        &self,
        repo_name: &str,
        workflow_file: &str,
        branch: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Extract organization from REPO_URL constant
        let org_name = REPO_URL
            .split('/')
            .last()
            .ok_or("Could not extract organization from REPO_URL")?;

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .map_err(|_| "Failed to create authorization header")?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_str("application/vnd.github.v3+json")
                .map_err(|_| "Failed to create accept header")?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str("NextNode-Project-Generator/1.0")
                .map_err(|_| "Failed to create user-agent header")?,
        );

        // Build request body for workflow dispatch
        let body = json!({
            "ref": branch
        });

        // Make GitHub API call to trigger workflow
        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches",
                org_name, repo_name, workflow_file
            ))
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to trigger workflow {}: {}", workflow_file, e))?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(
                format!("GitHub API error for workflow {}: {}", workflow_file, error).into(),
            );
        }

        println!("✅ Successfully triggered workflow: {}", workflow_file);
        Ok(())
    }

    pub async fn trigger_deployments(
        &self,
        repo_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if auto-deployment is disabled
        if let Some(no_deploy) = crate::utils::context::get_variable("no_deploy") {
            let is_disabled = match no_deploy.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                _ => false,
            };
            if is_disabled {
                println!(
                    "🚫 Auto-deployment disabled (no_deploy={}), skipping workflow triggers",
                    no_deploy
                );
                return Ok(());
            }
        }

        println!("🚀 Triggering deployment workflows...");

        // Wait longer for GitHub to index the workflows
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        // Trigger dev deployment on develop branch
        match self
            .trigger_workflow_dispatch(repo_name, "deploy-dev.yml", "develop")
            .await
        {
            Ok(_) => println!("✅ Dev deployment workflow triggered on develop branch"),
            Err(e) => eprintln!("⚠️  Warning: Failed to trigger dev deployment: {}", e),
        }

        // Wait between requests to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Trigger prod deployment on main branch
        match self
            .trigger_workflow_dispatch(repo_name, "deploy-prod.yml", "main")
            .await
        {
            Ok(_) => println!("✅ Production deployment workflow triggered on main branch"),
            Err(e) => eprintln!("⚠️  Warning: Failed to trigger prod deployment: {}", e),
        }

        println!("🎉 Deployment workflows have been triggered! Check GitHub Actions for status.");
        Ok(())
    }

    pub async fn create_develop_branch(
        &self,
        repo_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Extract organization from REPO_URL constant
        let org_name = REPO_URL
            .split('/')
            .last()
            .ok_or("Could not extract organization from REPO_URL")?;

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .map_err(|_| "Failed to create authorization header")?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_str("application/vnd.github.v3+json")
                .map_err(|_| "Failed to create accept header")?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str("NextNode-Project-Generator/1.0")
                .map_err(|_| "Failed to create user-agent header")?,
        );

        let client = reqwest::Client::new();

        // First, get the SHA of the main branch
        let main_ref_response = client
            .get(&format!(
                "https://api.github.com/repos/{}/{}/git/refs/heads/main",
                org_name, repo_name
            ))
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| format!("Failed to get main branch SHA: {}", e))?;

        if !main_ref_response.status().is_success() {
            let error = main_ref_response
                .text()
                .await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error getting main branch: {}", error).into());
        }

        let main_ref_data: serde_json::Value = main_ref_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse main branch response: {}", e))?;

        let main_sha = main_ref_data["object"]["sha"]
            .as_str()
            .ok_or("No SHA found in main branch response")?;

        println!("📋 Main branch SHA: {}", main_sha);

        // Check if develop branch already exists
        let develop_check_response = client
            .get(&format!(
                "https://api.github.com/repos/{}/{}/git/refs/heads/develop",
                org_name, repo_name
            ))
            .headers(headers.clone())
            .send()
            .await;

        if let Ok(response) = develop_check_response {
            if response.status().is_success() {
                println!("ℹ️  Develop branch already exists, skipping creation");
                return Ok(());
            }
        }

        // Create develop branch from main SHA
        let create_branch_body = json!({
            "ref": "refs/heads/develop",
            "sha": main_sha
        });

        let create_response = client
            .post(&format!(
                "https://api.github.com/repos/{}/{}/git/refs",
                org_name, repo_name
            ))
            .headers(headers)
            .json(&create_branch_body)
            .send()
            .await
            .map_err(|e| format!("Failed to create develop branch: {}", e))?;

        if !create_response.status().is_success() {
            let error = create_response
                .text()
                .await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error creating develop branch: {}", error).into());
        }

        println!("✅ Successfully created develop branch from main");
        Ok(())
    }

    pub async fn setup_repository_branches(
        &self,
        repo_name: &str,
        create_develop: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if create_develop {
            println!("🔧 Creating develop branch...");

            // Wait a bit for the repository to be fully initialized after push
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            match self.create_develop_branch(repo_name).await {
                Ok(_) => println!("✅ Develop branch created successfully"),
                Err(e) => eprintln!("⚠️  Warning: Failed to create develop branch: {}", e),
            }
        } else {
            println!("ℹ️  Skipping develop branch creation (not configured)");
        }

        println!("✅ Repository setup completed!");
        Ok(())
    }
}
