//! Routing: decide where a captured artifact is delivered (which SSH host or
//! local directory) based on the foreground window title and the user's
//! config. Implemented as a chain of `ContextResolver`s tried in priority
//! order — the first to return a candidate wins. Extracted out of
//! `process_job` (plan step 8) so routing can be unit-tested in isolation.

use std::path::PathBuf;

use crate::config::{AppConfig, SshConfig};

/// Everything a resolver needs to make a decision.
pub struct ResolutionInput<'a> {
    /// Foreground window title at capture time. `None` where it can't be read
    /// (e.g. Wayland) — resolvers that rely on the title simply won't match.
    pub window_title: Option<&'a str>,
    pub config: &'a AppConfig,
}

/// A resolved delivery target.
pub enum ResolutionCandidate {
    /// Upload to this SSH host (SFTP for password, SCP for key auth).
    Ssh(SshConfig),
    /// Copy into this local directory.
    Local(PathBuf),
}

/// Which resolver in the chain produced a candidate (kept for diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteSource {
    ManualRule,
    SshConfig,
    DefaultSsh,
}

/// One step of the routing chain. Resolvers are tried in order; the first to
/// return `Some(candidate)` wins.
pub trait ContextResolver {
    fn resolve(&self, input: &ResolutionInput<'_>) -> Option<ResolutionCandidate>;
}

/// Run the standard chain (manual rules → ssh-config auto-detect → default
/// SSH). First match wins; `None` means "no remote target — use the local
/// temp file".
pub fn resolve_target(input: &ResolutionInput<'_>) -> Option<(ResolutionCandidate, RouteSource)> {
    let chain: [(&dyn ContextResolver, RouteSource); 3] = [
        (&ManualRuleResolver, RouteSource::ManualRule),
        (&SshConfigResolver, RouteSource::SshConfig),
        (&DefaultSshResolver, RouteSource::DefaultSsh),
    ];
    for (resolver, source) in chain {
        if let Some(candidate) = resolver.resolve(input) {
            return Some((candidate, source));
        }
    }
    None
}

// ── Resolvers (priority order) ───────────────────────────────────────────

/// (a) Highest priority: explicit manual rules in `config.targets`, matched by
/// window-title substring.
pub struct ManualRuleResolver;

impl ContextResolver for ManualRuleResolver {
    fn resolve(&self, input: &ResolutionInput<'_>) -> Option<ResolutionCandidate> {
        let title = input.window_title?.to_lowercase();
        let targets = input.config.targets.as_ref()?;
        for target in targets {
            if !target.enabled || target.match_pattern.is_empty() {
                continue;
            }
            if !title.contains(&target.match_pattern.to_lowercase()) {
                continue;
            }
            // matched this rule
            match target.r#type.as_str() {
                "ssh" => {
                    return Some(ResolutionCandidate::Ssh(SshConfig {
                        enabled: true,
                        host: target.host.clone().unwrap_or_default(),
                        port: target.port,
                        username: target.username.clone(),
                        remote_dir: target
                            .remote_dir
                            .clone()
                            .unwrap_or_else(|| "/tmp/img2cli".to_string()),
                        match_pattern: Some(target.match_pattern.clone()),
                        remember_password: target.remember_password.unwrap_or(true),
                    }));
                }
                "local" => {
                    if let Some(dir) = target.local_dir.clone() {
                        return Some(ResolutionCandidate::Local(PathBuf::from(dir)));
                    }
                    // matched a local rule but no dir configured — keep searching
                }
                _ => {}
            }
        }
        None
    }
}

/// (b) Auto-detect: match the active window title against hosts parsed from
/// `~/.ssh/config`. The most specific match (longest alias/hostname present
/// in the title) wins.
pub struct SshConfigResolver;

impl ContextResolver for SshConfigResolver {
    fn resolve(&self, input: &ResolutionInput<'_>) -> Option<ResolutionCandidate> {
        let title = input.window_title?.to_lowercase();
        let cfg_path = crate::ssh_config::ssh_config_path()?;
        let content = std::fs::read_to_string(&cfg_path).ok()?;
        let hosts = crate::ssh_config::parse_ssh_config(&content);
        let best = hosts
            .into_iter()
            .filter(|h| {
                (!h.alias.is_empty() && title.contains(&h.alias.to_lowercase()))
                    || (!h.host.is_empty() && title.contains(&h.host.to_lowercase()))
            })
            .max_by_key(|h| h.alias.len().max(h.host.len()));
        let h = best?;
        // Inherit the configured default remote dir if set, else the standard temp.
        let default_remote = input
            .config
            .ssh
            .as_ref()
            .map(|s| s.remote_dir.clone())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| "/tmp/img2cli".to_string());
        Some(ResolutionCandidate::Ssh(SshConfig {
            enabled: true,
            host: h.host,
            port: Some(h.port),
            username: Some(h.username),
            remote_dir: default_remote,
            match_pattern: Some(h.alias),
            remember_password: true,
        }))
    }
}

/// (c) Fallback: the default SSH host from config, if enabled.
pub struct DefaultSshResolver;

impl ContextResolver for DefaultSshResolver {
    fn resolve(&self, input: &ResolutionInput<'_>) -> Option<ResolutionCandidate> {
        let ssh = input.config.ssh.as_ref()?;
        if ssh.enabled {
            Some(ResolutionCandidate::Ssh(ssh.clone()))
        } else {
            None
        }
    }
}
