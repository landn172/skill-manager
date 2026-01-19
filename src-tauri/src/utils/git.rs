use std::path::Path;
use tokio::process::Command;

pub struct ParsedSource {
    pub url: String,
    pub subpath: Option<String>,
}

pub fn parse_source(input: &str) -> ParsedSource {
    // Basic implementation of the logic in add-skill/src/git.ts
    if let Some(caps) = regex::Regex::new(r"github\.com/([^/]+)/([^/]+)/tree/([^/]+)/(.+)")
        .unwrap()
        .captures(input)
    {
        return ParsedSource {
            url: format!("https://github.com/{}/{}.git", &caps[1], &caps[2]),
            subpath: Some(caps[4].to_string()),
        };
    }

    if let Some(caps) = regex::Regex::new(r"github\.com/([^/]+)/([^/]+)")
        .unwrap()
        .captures(input)
    {
        let repo = caps[2].trim_end_matches(".git");
        return ParsedSource {
            url: format!("https://github.com/{}/{}.git", &caps[1], repo),
            subpath: None,
        };
    }

    // Shorthand owner/repo
    if let Some(caps) = regex::Regex::new(r"^([^/]+)/([^/]+)(?:/(.+))?$")
        .unwrap()
        .captures(input)
    {
        if !input.contains(':') {
            return ParsedSource {
                url: format!("https://github.com/{}/{}.git", &caps[1], &caps[2]),
                subpath: caps.get(3).map(|m| m.as_str().to_string()),
            };
        }
    }

    ParsedSource {
        url: input.to_string(),
        subpath: None,
    }
}

pub async fn clone_repo(url: &str, dest: &Path) -> Result<(), String> {
    // Try git clone first
    let git_result = try_git_clone(url, dest).await;

    if git_result.is_ok() {
        return Ok(());
    }

    // Fallback to HTTP download for GitHub repos
    if url.contains("github.com") {
        println!("Git clone failed, trying HTTP download for: {}", url);
        return download_github_repo(url, dest).await;
    }

    // Return original git error if not a GitHub repo
    git_result
}

async fn try_git_clone(url: &str, dest: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .args(["clone", "--depth", "1", url, dest.to_str().unwrap()])
        // Disable all credential prompts - fail immediately if auth required
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "")
        .env("SSH_ASKPASS", "")
        .env("GCM_INTERACTIVE", "never")
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

async fn download_github_repo(url: &str, dest: &Path) -> Result<(), String> {
    // Convert git URL to zip download URL
    // https://github.com/owner/repo.git -> https://github.com/owner/repo/archive/HEAD.zip
    let zip_url = url
        .trim_end_matches(".git")
        .replace("github.com", "github.com")
        + "/archive/HEAD.zip";

    // Download the zip file
    let response = reqwest::get(&zip_url)
        .await
        .map_err(|e| format!("HTTP download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Create a temp file for the zip
    let temp_zip = dest
        .parent()
        .unwrap_or(Path::new("/tmp"))
        .join("download.zip");
    tokio::fs::write(&temp_zip, &bytes)
        .await
        .map_err(|e| format!("Failed to write zip file: {}", e))?;

    // Extract the zip
    let zip_file =
        std::fs::File::open(&temp_zip).map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| format!("Failed to read zip: {}", e))?;

    // Ensure dest exists
    std::fs::create_dir_all(dest).map_err(|e| format!("Failed to create destination: {}", e))?;

    // Extract, stripping the first directory level (repo-HEAD/)
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        let outpath = match file.enclosed_name() {
            Some(path) => {
                // Strip the first component (e.g., "repo-HEAD/")
                let components: Vec<_> = path.components().skip(1).collect();
                if components.is_empty() {
                    continue;
                }
                let mut out = dest.to_path_buf();
                for c in components {
                    out.push(c);
                }
                out
            }
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p).ok();
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }

    // Cleanup temp zip
    let _ = std::fs::remove_file(&temp_zip);

    Ok(())
}
