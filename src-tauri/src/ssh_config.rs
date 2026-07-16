use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// One connectable host parsed from an OpenSSH config file.
#[derive(Debug, Clone, Serialize)]
pub struct SshHostEntry {
    pub alias: String,
    pub host: String,
    pub port: u16,
    pub username: String,
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Location of the user's OpenSSH config (`~/.ssh/config`).
pub fn ssh_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".ssh").join("config"))
}

fn build_entry(alias: &str, props: &HashMap<String, String>, default_user: &str) -> SshHostEntry {
    SshHostEntry {
        host: props
            .get("hostname")
            .cloned()
            .unwrap_or_else(|| alias.to_string()),
        port: props
            .get("port")
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(22),
        username: props
            .get("user")
            .cloned()
            .unwrap_or_else(|| default_user.to_string()),
        alias: alias.to_string(),
    }
}

/// Parse OpenSSH config text into connectable host entries.
///
/// Host patterns containing wildcards (`*` / `?`) or negations (`!`) are
/// skipped, since they are not connectable aliases. `Include` directives are
/// not followed. When multiple patterns share a block (e.g. `Host a b`) each
/// non-wildcard pattern becomes its own entry sharing the block's settings.
pub fn parse_ssh_config(content: &str) -> Vec<SshHostEntry> {
    let default_user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default();

    let mut entries: Vec<SshHostEntry> = Vec::new();
    let mut aliases: Vec<String> = Vec::new();
    let mut props: HashMap<String, String> = HashMap::new();

    let flush = |aliases: &[String], props: &HashMap<String, String>, out: &mut Vec<SshHostEntry>| {
        for a in aliases {
            out.push(build_entry(a, props, &default_user));
        }
    };

    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.splitn(2, char::is_whitespace);
        let key = it.next().unwrap_or("").to_lowercase();
        let val = it.next().unwrap_or("").trim().to_string();
        if val.is_empty() {
            continue;
        }
        if key == "host" {
            flush(&aliases, &props, &mut entries);
            aliases.clear();
            props.clear();
            for tok in val.split_whitespace() {
                if tok.contains('*') || tok.contains('?') || tok.starts_with('!') {
                    continue;
                }
                aliases.push(tok.to_string());
            }
        } else {
            props.insert(key, val);
        }
    }
    flush(&aliases, &props, &mut entries);

    // Dedupe by alias, keeping the first occurrence.
    let mut seen: HashSet<String> = HashSet::new();
    entries.retain(|e| seen.insert(e.alias.clone()));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hosts_and_skips_wildcards() {
        let cfg = "Host gpu90\n  HostName 172.16.190.90\n  User mengzijun\n  Port 22\n\
Host *\n  User fallback\n\
Host quotosky\n  HostName quotosky.vip\n  Port 17901\n";
        let entries = parse_ssh_config(cfg);
        let aliases: Vec<&str> = entries.iter().map(|e| e.alias.as_str()).collect();
        assert_eq!(aliases, vec!["gpu90", "quotosky"]);

        let g = entries.iter().find(|e| e.alias == "gpu90").unwrap();
        assert_eq!(g.host, "172.16.190.90");
        assert_eq!(g.port, 22);
        assert_eq!(g.username, "mengzijun");

        let q = entries.iter().find(|e| e.alias == "quotosky").unwrap();
        assert_eq!(q.host, "quotosky.vip");
        assert_eq!(q.port, 17901);
    }

    #[test]
    fn alias_used_as_host_when_no_hostname() {
        let cfg = "Host mybox\n  User bob\n  Port 2222\n";
        let entries = parse_ssh_config(cfg);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host, "mybox");
        assert_eq!(entries[0].port, 2222);
        assert_eq!(entries[0].username, "bob");
    }

    #[test]
    fn multiple_patterns_split_and_case_insensitive() {
        let cfg = "host A B\n  hostname 10.0.0.5\n  user root\n";
        let entries = parse_ssh_config(cfg);
        let aliases: Vec<&str> = entries.iter().map(|e| e.alias.as_str()).collect();
        assert_eq!(aliases, vec!["A", "B"]);
        assert!(entries.iter().all(|e| e.host == "10.0.0.5" && e.username == "root"));
    }
}
