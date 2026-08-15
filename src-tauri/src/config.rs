use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SshConfig {
    #[serde(default)]
    pub enabled: bool,
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub remote_dir: String,
    #[serde(default)]
    pub match_pattern: Option<String>,
    #[serde(default)]
    pub remember_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TargetConfig {
    #[serde(default)]
    pub enabled: bool,
    pub r#type: String, // "ssh" or "local"
    pub match_pattern: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub remote_dir: Option<String>,
    pub local_dir: Option<String>,
    #[serde(default)]
    pub remember_password: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub save_dir: Option<PathBuf>,
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default = "default_compress_quality")]
    pub compress_quality: u8,
    #[serde(default = "default_max_dimension")]
    pub max_dimension: Option<u32>,
    #[serde(default)]
    pub workspace_aware: bool,
    #[serde(default = "default_wrap_single_quotes")]
    pub wrap_single_quotes: bool,
    
    // GUI / Daemon settings
    #[serde(default = "default_launch_on_boot")]
    pub launch_on_boot: bool,
    #[serde(default = "default_enable_notifications")]
    pub enable_notifications: bool,
    #[serde(default = "default_global_hotkey")]
    pub global_hotkey: String,
    #[serde(default = "default_screenshot_hotkey")]
    pub screenshot_hotkey: String,
    #[serde(default = "default_upload_strategy")]
    pub upload_strategy: String, // "eager" or "lazy"
    #[serde(default = "default_injection_mode")]
    pub injection_mode: InjectionMode,
    #[serde(default = "default_clean_keep_days")]
    pub clean_keep_days: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,

    // When Direct injection can't be verified as delivered, also copy the
    // path to the clipboard as insurance (P0, docs/ISSUES_20260809.md §2).
    // The former post_paste_wait_ms / input_release_timeout_ms fields were
    // never read anywhere and were removed in v0.3.12 along with the
    // swap/paste_keep machinery they were meant to tune.
    #[serde(default = "default_fallback_to_copy")]
    pub fallback_to_copy: bool,

    #[serde(default)]
    pub ssh: Option<SshConfig>,
    #[serde(default)]
    pub ssh_targets: Option<Vec<SshConfig>>,
    #[serde(default)]
    pub targets: Option<Vec<TargetConfig>>,
}

fn default_output_format() -> String { "markdown".to_string() }
fn default_compress_quality() -> u8 { 80 }
fn default_max_dimension() -> Option<u32> { Some(1024) }
fn default_wrap_single_quotes() -> bool { false }
fn default_launch_on_boot() -> bool { true }
fn default_enable_notifications() -> bool { true }
fn default_global_hotkey() -> String { "Alt+V".to_string() }
fn default_screenshot_hotkey() -> String { "Alt+Shift+S".to_string() }
fn default_upload_strategy() -> String { "eager".to_string() }
fn default_clean_keep_days() -> u32 { 1 }
fn default_theme() -> String { "dracula".to_string() }
fn default_language() -> String { "zh-CN".to_string() }
fn default_fallback_to_copy() -> bool { true }

/// How the generated paste-text is delivered to the AI CLI.
/// v0.3.12 consolidated five modes down to three:
/// - `Auto`   — the host policy decides per focused app (`host_policy`):
///   Direct where typing works, Copy for hosts that reject all synthetic
///   input.
/// - `Direct` — Enigo Unicode keystrokes, no clipboard (best-effort; the job
///   layer adds a clipboard insurance copy when `fallback_to_copy` is on).
/// - `Copy`   — clipboard only; the user pastes manually (Ctrl+V).
///
/// serde `rename_all = "snake_case"`. The removed v0.3.9–v0.3.11 values are
/// accepted as aliases (`"swap"` → Auto, `"paste_keep"` → Copy) so old
/// config.toml files load cleanly and normalize on the next save. Missing →
/// `Auto`; any other value → config-load error.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InjectionMode {
    #[serde(alias = "swap")]
    Auto,
    Direct,
    #[serde(alias = "paste_keep")]
    Copy,
}

fn default_injection_mode() -> InjectionMode {
    InjectionMode::Auto
}

impl InjectionMode {
    /// snake_case string used by the injector's match arms.
    pub fn as_str(&self) -> &'static str {
        match self {
            InjectionMode::Auto => "auto",
            InjectionMode::Direct => "direct",
            InjectionMode::Copy => "copy",
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            save_dir: None,
            output_format: default_output_format(),
            compress_quality: default_compress_quality(),
            max_dimension: default_max_dimension(),
            workspace_aware: false,
            wrap_single_quotes: default_wrap_single_quotes(),
            launch_on_boot: default_launch_on_boot(),
            enable_notifications: default_enable_notifications(),
            global_hotkey: default_global_hotkey(),
            screenshot_hotkey: default_screenshot_hotkey(),
            upload_strategy: default_upload_strategy(),
            injection_mode: default_injection_mode(),
            clean_keep_days: default_clean_keep_days(),
            theme: default_theme(),
            language: default_language(),
            fallback_to_copy: default_fallback_to_copy(),
            ssh: Some(SshConfig {
                enabled: false,
                host: "your_ssh_alias_or_ip".to_string(),
                port: Some(22),
                username: Some("your_username".to_string()),
                remote_dir: "/tmp/img2cli".to_string(),
                match_pattern: None,
                remember_password: true,
            }),
            ssh_targets: None,
            targets: None,
        }
    }
}

impl AppConfig {
    pub fn config_file_path() -> PathBuf {
        if cfg!(windows) {
            let appdata = std::env::var("APPDATA")
                .unwrap_or_else(|_| {
                    std::env::var("USERPROFILE")
                        .map(|h| format!("{}\\AppData\\Roaming", h))
                        .unwrap_or_else(|_| "C:\\".to_string())
                });
            PathBuf::from(appdata).join("img2cli").join("config.toml")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config").join("img2cli").join("config.toml")
        }
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn load() -> Result<Self, String> {
        Self::load_from_path(&Self::config_file_path())
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to_path(&Self::config_file_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_mode_migration_aliases() {
        // v0.3.11-and-older configs stored swap / paste_keep — they must keep
        // loading, normalizing onto the consolidated 3-mode set (v0.3.12).
        let cfg: AppConfig = toml::from_str("injection_mode = 'swap'").unwrap();
        assert_eq!(cfg.injection_mode, InjectionMode::Auto);
        let cfg: AppConfig = toml::from_str("injection_mode = 'paste_keep'").unwrap();
        assert_eq!(cfg.injection_mode, InjectionMode::Copy);

        // Canonical values parse and round-trip serialization normalizes to
        // them, so the next save rewrites old values permanently.
        let cfg: AppConfig = toml::from_str("injection_mode = 'copy'").unwrap();
        assert_eq!(cfg.injection_mode, InjectionMode::Copy);
        let out = toml::to_string(&cfg).unwrap();
        assert!(out.contains("injection_mode = \"copy\""));
        assert!(!out.contains("paste_keep"));
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.output_format, "markdown");
        assert_eq!(config.compress_quality, 80);
        assert_eq!(config.global_hotkey, "Alt+V");
    }

    #[test]
    fn test_config_save_load() {
        let config = AppConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.output_format, "markdown");
        assert_eq!(deserialized.compress_quality, 80);
    }

    #[test]
    fn test_config_disk_io() {
        let mut config = AppConfig::default();
        config.output_format = "html".to_string();
        config.compress_quality = 95;
        
        let temp_dir = std::env::temp_dir().join("img2cli_test");
        let test_path = temp_dir.join("config.toml");
        
        // Save using real I/O methods
        config.save_to_path(&test_path).unwrap();
        
        // Load using real I/O methods
        let loaded = AppConfig::load_from_path(&test_path).unwrap();
        
        assert_eq!(loaded.output_format, "html");
        assert_eq!(loaded.compress_quality, 95);
        
        let _ = std::fs::remove_file(test_path);
        let _ = std::fs::remove_dir(temp_dir);
    }
}
