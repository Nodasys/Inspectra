//! Platform-specific utilities

use crate::error::Result;

#[cfg(windows)]
pub mod windows;
#[cfg(unix)]
pub mod unix;

/// Platform information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os: String,
    pub version: String,
    pub architecture: String,
}

/// Get current platform information
pub fn get_platform_info() -> Result<PlatformInfo> {
    #[cfg(windows)]
    return windows::get_info();

    #[cfg(unix)]
    return unix::get_info();
}

/// Check if running with elevated privileges
pub fn is_elevated() -> bool {
    #[cfg(windows)]
    return windows::is_elevated();

    #[cfg(unix)]
    return unix::is_elevated();
}

/// Request elevation (platform-specific)
pub fn request_elevation() -> Result<()> {
    #[cfg(windows)]
    return windows::request_elevation();

    #[cfg(unix)]
    return unix::request_elevation();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info() {
        let info = get_platform_info().unwrap();
        assert!(!info.os.is_empty());
    }
}
