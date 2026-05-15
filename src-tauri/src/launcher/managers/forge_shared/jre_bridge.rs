use crate::launcher::context::LauncherContext;
use crate::launcher::jre;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeEra {
    Legacy, // <= 1.12.2
    Modern, // 1.13 - 1.20.1
    NeoForge, // 1.20.1+
}

pub struct JreBridge;

impl JreBridge {
    /// Returns the ideal JRE version for the game runtime based on MC version and loader era.
    pub fn get_runtime_java_version(mc_version: &str, era: ForgeEra) -> u32 {
        if era == ForgeEra::NeoForge {
            return 21; // NeoForge is modern and prefers 21
        }

        // Logic based on Minecraft versions
        if mc_version.starts_with("1.21") || mc_version.starts_with("1.20.5") || mc_version.starts_with("1.20.6") {
            21
        } else if mc_version.starts_with("1.17") || mc_version.starts_with("1.18") || mc_version.starts_with("1.19") || mc_version.starts_with("1.20") {
            17 // Java 17 is the "Golden Version" for 1.17 - 1.20.4 Forge
        } else if mc_version.starts_with("1.13") || mc_version.starts_with("1.14") || mc_version.starts_with("1.15") || mc_version.starts_with("1.16") {
            11 // Java 11 is safer for modern-early forge than Java 8
        } else {
            8 // Strict Java 8 for Legacy Forge
        }
    }

    /// Returns the ideal JRE version for the INSTALLER phase.
    /// Modern installers often fail on old Java even if the game needs it.
    pub fn get_installer_java_version(mc_version: &str, era: ForgeEra) -> u32 {
        match era {
            ForgeEra::Legacy => 8, // Legacy installer is also old
            ForgeEra::Modern | ForgeEra::NeoForge => {
                if mc_version.starts_with("1.17") || mc_version.starts_with("1.18") || mc_version.starts_with("1.19") || mc_version.starts_with("1.20") || mc_version.starts_with("1.21") {
                    21
                } else {
                    17 // For 1.13 - 1.16 installers, Java 17 is robust enough to run the processors
                }
            }
        }
    }

    /// Ensures the correct JRE is downloaded and returns its path.
    pub async fn ensure_correct_jre(ctx: &LauncherContext, mc_version: &str, era: ForgeEra, for_installer: bool) -> Result<String, String> {
        let version = if for_installer {
            Self::get_installer_java_version(mc_version, era)
        } else {
            Self::get_runtime_java_version(mc_version, era)
        };
        
        println!("[ForgeShared] Ensuring JRE {} for {} phase...", version, if for_installer { "installer" } else { "runtime" });
        jre::ensure_jre(ctx, version).await
    }
}
