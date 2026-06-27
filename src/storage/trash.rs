use anyhow::Result;
use std::path::Path;

#[cfg(target_os = "macos")]
pub fn move_to_trash(path: &Path) -> Result<()> {
    let trash_dir = directories::BaseDirs::new()
        .map(|d| d.home_dir().join(".Trash"))
        .unwrap_or_else(|| Path::new("/Users/Shared/.Trash").to_path_buf());
    let filename = path.file_name().unwrap_or_default();
    let dest = trash_dir.join(filename);
    std::fs::rename(path, dest)?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn move_to_trash(path: &Path) -> Result<()> {
    let trash_dir = directories::BaseDirs::new()
        .map(|d| d.home_dir().join(".local/share/Trash/files"))
        .unwrap_or_else(|| Path::new("/tmp").to_path_buf());
    if !trash_dir.exists() {
        std::fs::create_dir_all(&trash_dir)?;
    }
    let filename = path.file_name().unwrap_or_default();
    let dest = trash_dir.join(filename);
    std::fs::rename(path, dest)?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn move_to_trash(path: &Path) -> Result<()> {
    let trash_dir = directories::BaseDirs::new()
        .map(|d| {
            d.home_dir()
                .join("AppData/Local/Microsoft/Windows/RecycleBin")
        })
        .unwrap_or_else(|| Path::new("C:/RecycleBin").to_path_buf());
    if !trash_dir.exists() {
        std::fs::create_dir_all(&trash_dir)?;
    }
    let filename = path.file_name().unwrap_or_default();
    let dest = trash_dir.join(filename);
    std::fs::rename(path, dest)?;
    Ok(())
}
