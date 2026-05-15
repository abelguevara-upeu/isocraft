/// Known version issues, indexed by (version_id, platform).
/// Platform: "macos", "windows", "linux", or "*" for all platforms.
/// Add entries here when a version has a confirmed reproducible issue.
struct KnownIssue {
    version: &'static str,
    platform: &'static str,
    message: &'static str,
}

const KNOWN_ISSUES: &[KnownIssue] = &[
    KnownIssue {
        version: "1.19.3",
        platform: "macos",
        message: "1.19.3 crashes on macOS (SIGABRT) due to a LWJGL 3.3.1 bug with libdispatch. \
                  This affects all third-party launchers. \
                  Use 1.19.2, 1.18.X, or 1.20+ as a workaround.",
    },
    KnownIssue {
        version: "1.19.4",
        platform: "macos",
        message: "1.19.4 crashes on macOS (SIGABRT) due to a LWJGL 3.3.1 bug with libdispatch. \
                  This affects all third-party launchers. \
                  Use 1.19.2, 1.18.X, or 1.20+ as a workaround.",
    },
];

/// Returns a warning message if the given version has known issues on the current platform.
/// Returns None if the version is expected to work correctly.
pub fn get_launch_warning(version_id: &str) -> Option<String> {
    let current_platform = std::env::consts::OS; // "macos", "windows", "linux"

    for issue in KNOWN_ISSUES {
        let version_matches = issue.version == version_id;
        let platform_matches = issue.platform == "*" || issue.platform == current_platform;

        if version_matches && platform_matches {
            return Some(issue.message.to_string());
        }
    }

    None
}
