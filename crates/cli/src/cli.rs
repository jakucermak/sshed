use std::{fs::canonicalize, io::Error, path::PathBuf};

use clap::Parser;
use log::{debug, error, warn};

/// Command line arguments for the application
///
/// This struct defines the command line interface using clap.
/// It currently supports specifying an optional configuration file path.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to configuration file.
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Run in cli
    #[arg(short, long)]
    tui: bool,
}

/// Gets the default platform-specific configuration file path
///
/// Returns a String containing the default path where the configuration file
/// should be located based on the current operating system:
/// - Windows: %USERPROFILE%/.sshed/config.toml
/// - Other: $HOME/.config/sshed/config.toml
///
/// # Examples
///
/// ```
/// use cli::get_default_config_path;
///
/// let config_path = get_default_config_path();
/// assert!(config_path.contains("config.toml"));
/// ```
pub fn get_default_config_path() -> String {
    if cfg!(target_os = "windows") {
        format!(
            "{}/.sshed/config.toml",
            std::env::var("USERPROFILE").unwrap_or_default()
        )
    } else {
        format!(
            "{}/.config/sshed/config.toml",
            std::env::var("HOME").unwrap_or_default()
        )
    }
}

/// Checks if a file exists at the given path
///
/// # Arguments
///
/// * `path` - A string slice containing the path to check
///
/// # Returns
///
/// Returns Ok(()) if the file exists, otherwise returns the OS error
///
/// # Examples
///
/// ```
/// use cli::try_check_file;
///
/// match try_check_file("/path/to/file") {
///     Ok(_) => println!("File exists!"),
///     Err(e) => println!("File not found: {}", e)
/// }
/// ```
pub fn try_check_file(path: &str) -> Result<(), Error> {
    debug!("Checking if file exists at: {}", path);
    match std::fs::metadata(path) {
        Ok(_) => {
            debug!("Found file at: {}", path);
            Ok(())
        }
        Err(e) => {
            warn!("No file found at: {}", path);
            Err(e)
        }
    }
}

/// Attempts to open a configuration file from the provided path or default location
///
/// First tries to open the file at the user-provided path if one is given.
/// If that fails or no path is provided, tries the default platform-specific path.
///
/// # Arguments
///
/// * `path` - Optional String containing user-provided config file path
///
/// # Returns
///
/// Returns Result containing either the opened File or an Error
fn check_config_path(path: Option<String>) -> Result<PathBuf, Error> {
    debug!("Checking configuration paths");

    // Try user-provided path first
    if let Some(user_path) = path {
        if let Ok(_) = try_check_file(&user_path) {
            return canonicalize(&user_path);
        }
    }

    // Try default platform-specific path
    let default_path = get_default_config_path();
    match try_check_file(&default_path) {
        Ok(_) => canonicalize(&default_path),
        Err(e) => {
            error!("No valid configuration file found");
            Err(e)
        }
    }
}

/// Parses command line arguments and returns opened configuration file
///
/// # Returns
///
/// Returns Result containing either opened config File or Error
///
/// # Examples
///
/// ```
/// match cli::parse_args() {
///     Ok(file) => println!("Successfully opened config file"),
///     Err(e) => eprintln!("Failed to open config file: {}", e)
/// }
/// ```
pub fn parse_args() -> Result<PathBuf, Error> {
    let args = Args::parse();
    match check_config_path(args.config) {
        Ok(f) => Ok(f),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_default_config_path() {
        // Create config file and directories
        let path = get_default_config_path();
        std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
        std::fs::File::create(&path).unwrap();

        // Run tests
        assert!(!path.is_empty());
        assert!(path.contains("config.toml"));

        // Clean up
        std::fs::remove_file(&path).unwrap();
        let parent = std::path::Path::new(&path).parent().unwrap();
        std::fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn test_try_check_file() {
        // Test non-existent file
        let result = try_check_file("non_existent_file.txt");
        assert!(result.is_err());

        // Test existing file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        let result = try_check_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_config_path() {
        // Test non-existent path
        let result = check_config_path(Some("non_existent_config.toml".to_string()));
        assert!(result.is_err());

        // Test with temp file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test config").unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        let result = check_config_path(Some(path));
        assert!(result.is_ok());
    }
}
