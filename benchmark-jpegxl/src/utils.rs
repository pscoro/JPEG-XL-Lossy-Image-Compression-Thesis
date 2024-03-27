use std::fs;
use std::path::Path;

/// Check if a direcotry exists or create it.
///
/// # Arguments
/// * `path` - A string slice that holds the path to the directory.
///
/// # Returns
/// * The path to the directory as an owned String, or an error if the directory could not be created and does not exist.
pub fn exists_or_create_dir(path: &str) -> Result<String, std::io::Error> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)?;
    }
    Ok(path.to_string())
}

/// Check if a directory exists.
///
/// # Arguments
/// * `path` - A string slice that holds the path to the directory.
///
/// # Returns
/// * The path to the directory as an owned String, or an error if the path does not exist or is not a directory.
pub fn dir_exists(path: &str) -> Result<String, std::io::Error> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path not found: {}", path.display()),
        ));
    }
    if !path.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a directory: {}", path.display()),
        ));
    }
    Ok(path.to_str().unwrap().to_string())
}
