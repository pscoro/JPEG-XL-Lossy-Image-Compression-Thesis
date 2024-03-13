use std::fs;
use std::path::Path;

pub fn exists_or_create_dir(path: &str) -> Result<String, std::io::Error> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)?;
    }
    Ok(path.to_string())
}
