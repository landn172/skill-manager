use std::path::Path;
use tokio::fs;

pub async fn copy_dir_recursive(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), String> {
    let dst = dst.as_ref();
    // Validate source exists before copying
    if !src.as_ref().exists() {
        return Err(format!(
            "Source directory does not exist: {:?}",
            src.as_ref()
        ));
    }

    // Clear destination if it exists to avoid permission/conflict issues
    if dst.exists() {
        let _ = fs::remove_dir_all(dst).await;
    }

    let mut stack = vec![(src.as_ref().to_path_buf(), dst.to_path_buf())];

    while let Some((current_src, current_dst)) = stack.pop() {
        fs::create_dir_all(&current_dst)
            .await
            .map_err(|e| e.to_string())?;
        let mut entries = fs::read_dir(&current_src)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip common metadata/heavy directories
            if file_name_str == ".git"
                || file_name_str == "node_modules"
                || file_name_str == "__pycache__"
            {
                continue;
            }

            let ty = entry.file_type().await.map_err(|e| e.to_string())?;
            if ty.is_dir() {
                stack.push((entry.path(), current_dst.join(file_name)));
            } else {
                fs::copy(entry.path(), current_dst.join(file_name))
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// Create hard links from src to dst (recursively)
/// If hard link fails (e.g. cross-filesystem), falls back to copy
pub async fn link_dir_recursive(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), String> {
    let dst = dst.as_ref();
    // Validate source exists before linking
    if !src.as_ref().exists() {
        return Err(format!(
            "Source directory does not exist: {:?}",
            src.as_ref()
        ));
    }

    // Clear destination if it exists
    if dst.exists() {
        let _ = fs::remove_dir_all(dst).await;
    }

    let mut stack = vec![(src.as_ref().to_path_buf(), dst.to_path_buf())];

    while let Some((current_src, current_dst)) = stack.pop() {
        fs::create_dir_all(&current_dst)
            .await
            .map_err(|e| e.to_string())?;
        let mut entries = fs::read_dir(&current_src)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip common metadata/heavy directories
            if file_name_str == ".git"
                || file_name_str == "node_modules"
                || file_name_str == "__pycache__"
            {
                continue;
            }

            let ty = entry.file_type().await.map_err(|e| e.to_string())?;
            let dst_path = current_dst.join(file_name);

            if ty.is_dir() {
                stack.push((entry.path(), dst_path));
            } else {
                // Try hard link first
                if let Err(_) = std::fs::hard_link(entry.path(), &dst_path) {
                    // Fallback to copy if hard link fails
                    fs::copy(entry.path(), &dst_path)
                        .await
                        .map_err(|e| format!("Failed to link or copy {:?}: {}", entry.path(), e))?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub async fn remove_quarantine(path: impl AsRef<Path>) -> Result<(), String> {
    use tokio::process::Command;
    let path_str = path.as_ref().to_string_lossy();

    // xattr -d -r com.apple.quarantine <path>
    // We ignore errors because the attribute might not exist.
    let _ = Command::new("xattr")
        .args(&["-d", "-r", "com.apple.quarantine", &path_str])
        .output()
        .await;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub async fn remove_quarantine(_path: impl AsRef<Path>) -> Result<(), String> {
    Ok(())
}
