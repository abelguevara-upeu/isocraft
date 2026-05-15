/// Returns the OS key compatible with Mojang manifests (osx, windows, linux).
pub fn get_os_key() -> &'static str {
    match std::env::consts::OS {
        "macos" => "osx",
        "windows" => "windows",
        _ => "linux",
    }
}

/// Returns the architecture key used in modern Minecraft manifests (x64, arm64).
pub fn get_arch_key() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x64",
    }
}
