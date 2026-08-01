use std::path::Path;

/// Create a symlink from `dst` -> `src`, replacing any existing `dst`.
/// On Windows (where symlinks need privileges) this falls back to a file copy.
#[cfg(unix)]
pub fn symlink_or_copy(src: &Path, dst: &Path) -> std::io::Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if dst.exists() || std::fs::symlink_metadata(dst).is_ok() {
        let _ = std::fs::remove_file(dst);
    }
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
pub fn symlink_or_copy(src: &Path, dst: &Path) -> std::io::Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if dst.exists() {
        let _ = std::fs::remove_file(dst);
    }
    match std::os::windows::fs::symlink_file(src, dst) {
        Ok(()) => Ok(()),
        Err(_) => std::fs::copy(src, dst).map(|_| ()),
    }
}
