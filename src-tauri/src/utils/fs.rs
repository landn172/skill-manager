use std::path::Path;
use tokio::fs;

pub async fn copy_dir_recursive(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), String> {
    // Validate source exists before copying
    if !src.as_ref().exists() {
        return Err(format!(
            "Source directory does not exist: {:?}",
            src.as_ref()
        ));
    }

    let mut stack = vec![(src.as_ref().to_path_buf(), dst.as_ref().to_path_buf())];

    while let Some((current_src, current_dst)) = stack.pop() {
        fs::create_dir_all(&current_dst)
            .await
            .map_err(|e| e.to_string())?;
        let mut entries = fs::read_dir(&current_src)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let ty = entry.file_type().await.map_err(|e| e.to_string())?;
            if ty.is_dir() {
                stack.push((entry.path(), current_dst.join(entry.file_name())));
            } else {
                fs::copy(entry.path(), current_dst.join(entry.file_name()))
                    .await
                    .map_err(|e| e.to_string())?;
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
