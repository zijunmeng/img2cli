use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SshConfig {
    pub enabled: bool,
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub remote_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub save_dir: Option<PathBuf>,
    pub hotkey: String,
    pub output_format: String, // "raw", "markdown", "html", "base64"
    pub compress_quality: u8, // 0 - 100
    pub max_dimension: Option<u32>,
    pub workspace_aware: bool,
    pub ssh: Option<SshConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_dir: None,
            hotkey: "ctrl+shift+v".to_string(),
            output_format: "markdown".to_string(),
            compress_quality: 80,
            max_dimension: Some(1024),
            workspace_aware: false,
            ssh: Some(SshConfig {
                enabled: false,
                host: "S91".to_string(),
                port: Some(22),
                username: Some("mengzijun".to_string()),
                remote_dir: "/tmp/img2cli".to_string(),
            }),
        }
    }
}

impl Config {
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

    pub fn config_file_path() -> PathBuf {
        let base_dir = if cfg!(windows) {
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string())
        } else {
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        };
        PathBuf::from(base_dir).join(".config").join("img2cli").join("config.toml")
    }

    pub fn get_save_dir(&self) -> PathBuf {
        self.save_dir.clone().unwrap_or_else(|| {
            std::env::temp_dir().join("img2cli")
        })
    }
}
