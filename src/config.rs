use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub save_dir: Option<PathBuf>,
    #[serde(default = "default_output_format")]
    pub output_format: String, // "raw", "markdown", "html", "base64"
    #[serde(default = "default_compress_quality")]
    pub compress_quality: u8, // 0 - 100
    #[serde(default = "default_max_dimension")]
    pub max_dimension: Option<u32>,
    #[serde(default)]
    pub workspace_aware: bool,
    #[serde(default = "default_wrap_single_quotes")]
    pub wrap_single_quotes: bool,
    
    // GUI / Daemon fields preserved in CLI config
    #[serde(default = "default_launch_on_boot")]
    pub launch_on_boot: bool,
    #[serde(default = "default_enable_notifications")]
    pub enable_notifications: bool,
    #[serde(default = "default_global_hotkey")]
    pub global_hotkey: String,
    #[serde(default = "default_screenshot_hotkey")]
    pub screenshot_hotkey: String,
    #[serde(default = "default_upload_strategy")]
    pub upload_strategy: String,
    #[serde(default = "default_injection_mode")]
    pub injection_mode: String,
    #[serde(default = "default_clean_keep_days")]
    pub clean_keep_days: u32,
    
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

impl Default for Config {
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

impl Config {
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

    pub fn load() -> Self {
        let config_path = Self::config_file_path();
        if !config_path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Warning: Failed to parse config file: {}. Using default configuration.", e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("Warning: Failed to read config file: {}. Using default configuration.", e);
                Self::default()
            }
        }
    }

    pub fn get_save_dir(&self) -> PathBuf {
        self.save_dir.clone().unwrap_or_else(|| {
            std::env::temp_dir().join("img2cli")
        })
    }
}
