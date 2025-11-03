//! Unix platform utilities

use crate::error::{InspectraError, Result};
use crate::platform::PlatformInfo;

pub fn get_info() -> Result<PlatformInfo> {
    Ok(PlatformInfo {
        os: std::env::consts::OS.to_string(),
        version: "Unknown".to_string(), // Would use uname
        architecture: std::env::consts::ARCH.to_string(),
    })
}

pub fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn request_elevation() -> Result<()> {
    if is_elevated() {
        Ok(())
    } else {
        Err(InspectraError::permission_denied(
            "Root privileges required. Please run with sudo.",
        ))
    }
}
