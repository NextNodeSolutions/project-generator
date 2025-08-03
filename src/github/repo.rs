use git2::{Repository, Signature, Cred, RemoteCallbacks};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, ACCEPT, USER_AGENT};
use serde_json::json;
use std::path::Path;
use crate::config::REPO_URL;

pub struct GitHubRepo {
    token: String,
}

impl GitHubRepo {
    pub fn new(token: &str) -> Self {
        Self { 
            token: token.to_string() 
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
            let error = response.text().await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error: {}", error).into());
        }

        let repo_data: serde_json::Value = response.json().await
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
                .put(&format!("https://api.github.com/repos/{}/{}/topics", org_name, name))
                .headers(headers)
                .json(&topics_body)
                .send()
                .await
                .map_err(|e| format!("Failed to add topic: {}", e))?;

            if !topics_response.status().is_success() {
                let error = topics_response.text().await
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
            let error = response.text().await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error for workflow {}: {}", workflow_file, error).into());
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
                println!("🚫 Auto-deployment disabled (no_deploy={}), skipping workflow triggers", no_deploy);
                return Ok(());
            }
        }

        println!("🚀 Triggering deployment workflows...");
        
        // Wait longer for GitHub to index the workflows
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        // Trigger dev deployment on develop branch
        match self.trigger_workflow_dispatch(repo_name, "deploy-dev.yml", "develop").await {
            Ok(_) => println!("✅ Dev deployment workflow triggered on develop branch"),
            Err(e) => eprintln!("⚠️  Warning: Failed to trigger dev deployment: {}", e),
        }

        // Wait between requests to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Trigger prod deployment on main branch
        match self.trigger_workflow_dispatch(repo_name, "deploy-prod.yml", "main").await {
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
            let error = main_ref_response.text().await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error getting main branch: {}", error).into());
        }

        let main_ref_data: serde_json::Value = main_ref_response.json().await
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
            let error = create_response.text().await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error creating develop branch: {}", error).into());
        }

        println!("✅ Successfully created develop branch from main");
        Ok(())
    }

    pub async fn get_available_status_checks(
        &self,
        repo_name: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
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
        
        // Get the latest commit on main branch to check for available status checks
        let main_commit_response = client
            .get(&format!(
                "https://api.github.com/repos/{}/{}/commits/main",
                org_name, repo_name
            ))
            .headers(headers.clone())
            .send()
            .await;

        if let Ok(response) = main_commit_response {
            if response.status().is_success() {
                let commit_data: serde_json::Value = response.json().await
                    .map_err(|e| format!("Failed to parse commit response: {}", e))?;
                
                let commit_sha = commit_data["sha"].as_str().unwrap_or("");
                
                // Get status checks for this commit
                let status_response = client
                    .get(&format!(
                        "https://api.github.com/repos/{}/{}/commits/{}/check-runs",
                        org_name, repo_name, commit_sha
                    ))
                    .headers(headers)
                    .send()
                    .await;

                if let Ok(status_resp) = status_response {
                    if status_resp.status().is_success() {
                        let status_data: serde_json::Value = status_resp.json().await
                            .map_err(|e| format!("Failed to parse status response: {}", e))?;
                        
                        let mut check_names = Vec::new();
                        if let Some(check_runs) = status_data["check_runs"].as_array() {
                            for check_run in check_runs {
                                if let Some(name) = check_run["name"].as_str() {
                                    check_names.push(name.to_string());
                                }
                            }
                        }
                        
                        println!("📋 Found available status checks: {:?}", check_names);
                        return Ok(check_names);
                    }
                }
            }
        }

        // Fallback to default checks if API call fails
        println!("⚠️  Could not fetch status checks, using defaults");
        Ok(vec![
            "Quality Checks / 🔍 Lint".to_string(),
            "Quality Checks / 🔷 Type Check".to_string(),
            "Quality Checks / 🧪 Test".to_string(),
            "Quality Checks / 🔨 Build".to_string(),
        ])
    }

    pub async fn setup_branch_protection_main(
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

        // Use predictable status check names based on workflow structure
        // The test.yml workflow calls quality-checks.yml, creating checks like "Quality Checks / 🔍 Lint"
        let status_checks = vec![
            "Quality Checks / 🔍 Lint".to_string(),
            "Quality Checks / 🔷 Type Check".to_string(), 
            "Quality Checks / 🧪 Test".to_string(),
        ];

        println!("🔒 Configuring branch protection with status checks: {:?}", status_checks);

        // Build branch protection body for main (strict)
        let protection_body = json!({
            "required_status_checks": {
                "strict": true,
                "contexts": status_checks
            },
            "required_pull_request_reviews": {
                "required_approving_review_count": 0,
                "dismiss_stale_reviews": false,
                "require_code_owner_reviews": false
            },
            "required_conversation_resolution": true,
            "required_linear_history": true,
            "enforce_admins": true,
            "restrictions": null
        });

        let client = reqwest::Client::new();
        let response = client
            .put(&format!(
                "https://api.github.com/repos/{}/{}/branches/main/protection",
                org_name, repo_name
            ))
            .headers(headers)
            .json(&protection_body)
            .send()
            .await
            .map_err(|e| format!("Failed to set main branch protection: {}", e))?;

        if !response.status().is_success() {
            let error = response.text().await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error setting main branch protection: {}", error).into());
        }

        println!("🔒 Successfully set up branch protection for main (strict: PR required, quality-checks, conversation resolution, linear history, enforce admins)");
        Ok(())
    }

    pub async fn setup_branch_protection_develop(
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

        // Build branch protection body for develop (lighter - no PR required, no status checks)
        let protection_body = json!({
            "required_status_checks": null,
            "required_pull_request_reviews": null,
            "required_conversation_resolution": true,
            "required_linear_history": true,
            "enforce_admins": true,
            "restrictions": null
        });

        let client = reqwest::Client::new();
        let response = client
            .put(&format!(
                "https://api.github.com/repos/{}/{}/branches/develop/protection",
                org_name, repo_name
            ))
            .headers(headers)
            .json(&protection_body)
            .send()
            .await
            .map_err(|e| format!("Failed to set develop branch protection: {}", e))?;

        if !response.status().is_success() {
            let error = response.text().await
                .map_err(|e| format!("Failed to read error response: {}", e))?;
            return Err(format!("GitHub API error setting develop branch protection: {}", error).into());
        }

        println!("🔒 Successfully set up branch protection for develop (light: direct push allowed, conversation resolution, linear history, enforce admins)");
        Ok(())
    }

    pub async fn setup_repository_branches_and_protection(
        &self,
        repo_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔧 Setting up repository branches and protection rules...");
        
        // Wait a bit for the repository to be fully initialized after push
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // Step 1: Create develop branch from main
        match self.create_develop_branch(repo_name).await {
            Ok(_) => println!("✅ Develop branch setup completed"),
            Err(e) => {
                eprintln!("⚠️  Warning: Failed to create develop branch: {}", e);
                eprintln!("   Continuing with branch protection setup...");
            }
        }

        // Wait between API calls to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Step 2: Set up branch protection for main (strict)
        match self.setup_branch_protection_main(repo_name).await {
            Ok(_) => println!("✅ Main branch protection setup completed"),
            Err(e) => {
                eprintln!("⚠️  Warning: Failed to set up main branch protection: {}", e);
                eprintln!("   You may need to configure branch protection manually in GitHub settings");
            }
        }

        // Wait longer for develop branch to be fully available
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // Step 3: Set up branch protection for develop (light)
        match self.setup_branch_protection_develop(repo_name).await {
            Ok(_) => println!("✅ Develop branch protection setup completed"),
            Err(e) => {
                eprintln!("⚠️  Warning: Failed to set up develop branch protection: {}", e);
                eprintln!("   You may need to configure branch protection manually in GitHub settings");
            }
        }

        println!("🎉 Repository branch setup completed!");
        println!("📋 Summary:");
        println!("   • main branch: Protected (PR required, quality-checks, conversation resolution, linear history)");
        println!("   • develop branch: Protected (direct push allowed, conversation resolution, linear history)");
        
        Ok(())
    }
}
