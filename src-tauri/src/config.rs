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
    pub injection_mode: String,  // "direct" or "swap"
    #[serde(default = "default_clean_keep_days")]
    pub clean_keep_days: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    
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
fn default_wrap_single_quotes() -> bool { true }
fn default_launch_on_boot() -> bool { true }
fn default_enable_notifications() -> bool { true }
fn default_global_hotkey() -> String { "Alt+V".to_string() }
fn default_screenshot_hotkey() -> String { "Alt+Shift+S".to_string() }
fn default_upload_strategy() -> String { "eager".to_string() }
fn default_injection_mode() -> String { "direct".to_string() }
fn default_clean_keep_days() -> u32 { 1 }
fn default_theme() -> String { "apple-dark".to_string() }

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
