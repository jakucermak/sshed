#[cfg(target_os = "windows")]
pub(crate) fn ssh_config_path() -> String {
    format!(
        "{}/.ssh/config",
        std::env::var("USERPROFILE").unwrap_or_default()
    )
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn ssh_config_path() -> String {
    format!("{}/.ssh/config", std::env::var("HOME").unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_ssh_config_path() {
        let userprofile = std::env::var("USERPROFILE").unwrap_or_default();
        assert_eq!(ssh_config_path(), format!("{}/.ssh/config", userprofile));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_unix_ssh_config_path() {
        let home = std::env::var("HOME").unwrap_or_default();
        assert_eq!(ssh_config_path(), format!("{}/.ssh/config", home));
    }
}
