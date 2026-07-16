use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default = "default_compress_quality")]
    pub compress_quality: u8,
    #[serde(default = "default_max_dimension")]
    pub max_dimension: Option<u32>,
    #[serde(default = "default_wrap_single_quotes")]
    pub wrap_single_quotes: bool,
    #[serde(default = "default_launch_on_boot")]
    pub launch_on_boot: bool,
    #[serde(default = "default_enable_notifications")]
    pub enable_notifications: bool,
    #[serde(default = "default_global_hotkey")]
    pub global_hotkey: String,
    #[serde(default = "default_upload_strategy")]
    pub upload_strategy: String, // "eager" or "lazy"
    #[serde(default = "default_injection_mode")]
    pub injection_mode: String,  // "direct" or "swap"
    #[serde(default = "default_clean_keep_days")]
    pub clean_keep_days: u32,
    pub ssh: Option<SshConfig>,
    pub targets: Option<Vec<TargetConfig>>,
}

fn default_output_format() -> String { "markdown".to_string() }
fn default_compress_quality() -> u8 { 80 }
fn default_max_dimension() -> Option<u32> { Some(1024) }
fn default_wrap_single_quotes() -> bool { true }
fn default_launch_on_boot() -> bool { true }
fn default_enable_notifications() -> bool { true }
fn default_global_hotkey() -> String { "Alt+V".to_string() }
fn default_upload_strategy() -> String { "eager".to_string() }
fn default_injection_mode() -> String { "direct".to_string() }
fn default_clean_keep_days() -> u32 { 1 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            output_format: default_output_format(),
            compress_quality: default_compress_quality(),
            max_dimension: default_max_dimension(),
            wrap_single_quotes: default_wrap_single_quotes(),
            launch_on_boot: default_launch_on_boot(),
            enable_notifications: default_enable_notifications(),
            global_hotkey: default_global_hotkey(),
            upload_strategy: default_upload_strategy(),
            injection_mode: default_injection_mode(),
            clean_keep_days: default_clean_keep_days(),
            ssh: Some(SshConfig {
                enabled: false,
                host: "your_ssh_alias_or_ip".to_string(),
                port: Some(22),
                username: Some("your_username".to_string()),
                remote_dir: "/tmp/img2cli".to_string(),
                match_pattern: None,
            }),
            targets: None,
        }
    }
}

impl AppConfig {
    pub fn config_file_path() -> PathBuf {
        let base_dir = if cfg!(windows) {
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string())
        } else {
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        };
        PathBuf::from(base_dir).join(".config").join("img2cli").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_file_path();
        if !path.exists() {
            return Self::default();
        }
        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|_| Self::default()),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
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
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_path = temp_dir.join("config.toml");
        
        // Save
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&test_path, content).unwrap();
        
        // Load
        let read_content = std::fs::read_to_string(&test_path).unwrap();
        let loaded: AppConfig = toml::from_str(&read_content).unwrap();
        
        assert_eq!(loaded.output_format, "html");
        assert_eq!(loaded.compress_quality, 95);
        
        let _ = std::fs::remove_file(test_path);
        let _ = std::fs::remove_dir(temp_dir);
    }
}
