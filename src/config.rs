use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(default)]
    pub ssh: Option<SshConfig>,
    #[serde(default)]
    pub ssh_targets: Option<Vec<SshConfig>>,
}

fn default_output_format() -> String {
    "markdown".to_string()
}
fn default_compress_quality() -> u8 {
    80
}
fn default_max_dimension() -> Option<u32> {
    Some(1024)
}
fn default_wrap_single_quotes() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_dir: None,
            output_format: "markdown".to_string(),
            compress_quality: 80,
            max_dimension: Some(1024),
            workspace_aware: false,
            wrap_single_quotes: true,
            ssh: Some(SshConfig {
                enabled: false,
                host: "S91".to_string(),
                port: Some(22),
                username: Some("mengzijun".to_string()),
                remote_dir: "/tmp/img2cli".to_string(),
                match_pattern: None,
            }),
            ssh_targets: None,
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
