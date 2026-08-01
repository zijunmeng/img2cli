//! Delivery routing: decide *where* a captured artifact goes (which SSH host
//! or local directory), based on the foreground window and the user's config.
//!
//! A chain of `RouteResolver`s is tried in priority order; the first to
//! return a candidate wins. The standard chain always ends with
//! `LocalFallbackResolver`, so it always produces a target.
//!
//! Naming note: this is *delivery* routing, not execution-context resolution.
//! A future `ContextResolver` (shell integration / WSL / VS Code Remote /
//! container detection) will produce a richer `ForegroundContext` that feeds
//! into `RouteRequest`. Keeping these names distinct avoids a later collision.
//!
//! Resolver contract: select a target only. Do NOT read the keyring and do
//! NOT upload — auth choice and transport belong to `ArtifactTransport`
//! (Step 9). `~/.ssh/config` is parsed once by the caller and passed in via
//! `RouteRequest.ssh_hosts`, so resolvers stay pure and unit-testable.

use std::path::PathBuf;

use crate::config::{AppConfig, SshConfig};
use crate::ssh_config::SshHostEntry;

/// What we know about the foreground window/process at capture time. Minimal
/// now; grows once execution-context detection (shell/WSL/remote) lands.
#[derive(Debug, Clone, Default)]
pub struct ForegroundContext {
    pub window_title: Option<String>,
}

/// Inputs to routing. `ssh_hosts` is pre-parsed by the caller so resolvers do
/// no per-call file I/O.
pub struct RouteRequest<'a> {
    pub foreground: &'a ForegroundContext,
    pub config: &'a AppConfig,
    pub ssh_hosts: &'a [SshHostEntry],
}

/// A resolved destination plus provenance for diagnostics.
#[derive(Debug, Clone)]
pub struct RouteCandidate {
    pub target: DeliveryTarget,
    pub source: RouteSource,
    /// Human-readable reason, e.g. "window title matched SSH alias 'dev'".
    pub reason: String,
    /// 0–100 confidence hint (diagnostics only for now).
    pub confidence: u8,
}

#[derive(Debug, Clone)]
pub enum DeliveryTarget {
    /// Copy into this local directory.
    Local(PathBuf),
    /// Upload to this SSH host. Auth (password/keyring) is resolved later by
    /// `ArtifactTransport` (Step 9); for now this carries the config-domain
    /// SshConfig. A routing-domain `SshTarget` + `auth_profile` replaces it
    /// once transport needs to choose an auth strategy.
    Ssh(SshConfig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteSource {
    ManualRule,
    SshConfig,
    DefaultSsh,
    LocalFallback,
}

/// Routing failure. Deliberately distinct from "no match" (`Ok(None)`): a
/// resolver returns `Ok(None)` when it simply has no candidate, and `Err(_)`
/// when it could not decide (reserved for future resolvers that hit real
/// errors). The standard chain's `LocalFallback` always matches, so the
/// pipeline never sees `NoRoute` in practice.
#[derive(Debug, Clone)]
pub enum RouteError {
    NoRoute,
}

/// One step of the routing chain.
pub trait RouteResolver: Send + Sync {
    fn name(&self) -> &'static str;
    fn resolve(&self, request: &RouteRequest<'_>) -> Result<Option<RouteCandidate>, RouteError>;
}

/// An ordered set of resolvers; the first `Some(candidate)` wins.
pub struct ResolverChain {
    resolvers: Vec<Box<dyn RouteResolver>>,
}

impl ResolverChain {
    pub fn new(resolvers: Vec<Box<dyn RouteResolver>>) -> Self {
        Self { resolvers }
    }

    pub fn resolve(&self, request: &RouteRequest<'_>) -> Result<RouteCandidate, RouteError> {
        for resolver in &self.resolvers {
            if let Some(candidate) = resolver.resolve(request)? {
                return Ok(candidate);
            }
        }
        Err(RouteError::NoRoute)
    }
}

/// The standard chain used by the pipeline: manual rules → ssh-config
/// auto-detect → default SSH → local fallback. Always resolves.
pub fn default_chain() -> ResolverChain {
    ResolverChain::new(vec![
        Box::new(ManualRuleResolver),
        Box::new(SshConfigResolver),
        Box::new(DefaultSshResolver),
        Box::new(LocalFallbackResolver),
    ])
}

// ── Resolvers (priority order) ───────────────────────────────────────────

/// (1) Explicit manual rules in `config.targets`, matched by window-title
/// substring. Highest priority. An empty `match_pattern` never matches (guards
/// against the `"title".contains("")` trap that would match every window).
pub struct ManualRuleResolver;

impl RouteResolver for ManualRuleResolver {
    fn name(&self) -> &'static str {
        "manual_rule"
    }

    fn resolve(&self, request: &RouteRequest<'_>) -> Result<Option<RouteCandidate>, RouteError> {
        let title = match &request.foreground.window_title {
            Some(t) => t.to_lowercase(),
            None => return Ok(None),
        };
        let targets = match &request.config.targets {
            Some(t) if !t.is_empty() => t,
            _ => return Ok(None),
        };
        for target in targets {
            if !target.enabled || target.match_pattern.is_empty() {
                continue;
            }
            let pat = target.match_pattern.to_lowercase();
            if !title.contains(&pat) {
                continue;
            }
            match target.r#type.as_str() {
                "ssh" => {
                    return Ok(Some(RouteCandidate {
                        target: DeliveryTarget::Ssh(SshConfig {
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
                        }),
                        source: RouteSource::ManualRule,
                        reason: format!("manual rule matched {:?}", target.match_pattern),
                        confidence: 100,
                    }));
                }
                "local" => {
                    if let Some(dir) = target.local_dir.clone() {
                        return Ok(Some(RouteCandidate {
                            target: DeliveryTarget::Local(PathBuf::from(dir)),
                            source: RouteSource::ManualRule,
                            reason: format!("manual local rule matched {:?}", target.match_pattern),
                            confidence: 100,
                        }));
                    }
                    // matched a local rule but no dir configured — keep searching
                }
                _ => {}
            }
        }
        Ok(None)
    }
}

/// (2) Auto-detect: match the window title against hosts parsed from
/// `~/.ssh/config`. The most specific match (longest alias/hostname present
/// in the title) wins.
pub struct SshConfigResolver;

impl RouteResolver for SshConfigResolver {
    fn name(&self) -> &'static str {
        "ssh_config"
    }

    fn resolve(&self, request: &RouteRequest<'_>) -> Result<Option<RouteCandidate>, RouteError> {
        let title = match &request.foreground.window_title {
            Some(t) => t.to_lowercase(),
            None => return Ok(None),
        };
        let best = request
            .ssh_hosts
            .iter()
            .filter(|h| {
                (!h.alias.is_empty() && title.contains(&h.alias.to_lowercase()))
                    || (!h.host.is_empty() && title.contains(&h.host.to_lowercase()))
            })
            .max_by_key(|h| h.alias.len().max(h.host.len()));
        let h = match best {
            Some(h) => h,
            None => return Ok(None),
        };
        let default_remote = request
            .config
            .ssh
            .as_ref()
            .map(|s| s.remote_dir.clone())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| "/tmp/img2cli".to_string());
        Ok(Some(RouteCandidate {
            target: DeliveryTarget::Ssh(SshConfig {
                enabled: true,
                host: h.host.clone(),
                port: Some(h.port),
                username: Some(h.username.clone()),
                remote_dir: default_remote,
                match_pattern: Some(h.alias.clone()),
                remember_password: true,
            }),
            source: RouteSource::SshConfig,
            reason: format!("window title matched ssh-config host {:?}", h.alias),
            confidence: 80,
        }))
    }
}

/// (3) Default SSH host from config, if enabled.
pub struct DefaultSshResolver;

impl RouteResolver for DefaultSshResolver {
    fn name(&self) -> &'static str {
        "default_ssh"
    }

    fn resolve(&self, request: &RouteRequest<'_>) -> Result<Option<RouteCandidate>, RouteError> {
        match request.config.ssh.as_ref() {
            Some(ssh) if ssh.enabled => Ok(Some(RouteCandidate {
                target: DeliveryTarget::Ssh(ssh.clone()),
                source: RouteSource::DefaultSsh,
                reason: format!("default SSH host {:?}", ssh.host),
                confidence: 30,
            })),
            _ => Ok(None),
        }
    }
}

/// (4) Always matches: deliver locally. Uses the configured save dir (or the
/// temp default) — the pipeline's temp file already lives there, so the
/// resulting copy is a no-op (the delivery handler guards same-path copies).
pub struct LocalFallbackResolver;

impl RouteResolver for LocalFallbackResolver {
    fn name(&self) -> &'static str {
        "local_fallback"
    }

    fn resolve(&self, request: &RouteRequest<'_>) -> Result<Option<RouteCandidate>, RouteError> {
        let dir = request
            .config
            .save_dir
            .clone()
            .unwrap_or_else(|| std::env::temp_dir().join("img2cli"));
        Ok(Some(RouteCandidate {
            target: DeliveryTarget::Local(dir),
            source: RouteSource::LocalFallback,
            reason: "no remote target matched — using local file".to_string(),
            confidence: 0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, SshConfig, TargetConfig};
    use crate::ssh_config::SshHostEntry;

    fn fg(title: Option<&str>) -> ForegroundContext {
        ForegroundContext {
            window_title: title.map(|s| s.to_string()),
        }
    }

    fn config(targets: Vec<TargetConfig>, ssh: Option<SshConfig>) -> AppConfig {
        let mut c = AppConfig::default();
        c.targets = if targets.is_empty() { None } else { Some(targets) };
        c.ssh = ssh;
        c
    }

    fn manual_ssh(pattern: &str, host: &str, enabled: bool) -> TargetConfig {
        TargetConfig {
            enabled,
            r#type: "ssh".to_string(),
            match_pattern: pattern.to_string(),
            host: Some(host.to_string()),
            port: Some(22),
            username: Some("u".to_string()),
            remote_dir: Some("/srv/img".to_string()),
            local_dir: None,
            remember_password: Some(true),
        }
    }

    fn manual_local(pattern: &str, dir: &str) -> TargetConfig {
        TargetConfig {
            enabled: true,
            r#type: "local".to_string(),
            match_pattern: pattern.to_string(),
            host: None,
            port: None,
            username: None,
            remote_dir: None,
            local_dir: Some(dir.to_string()),
            remember_password: None,
        }
    }

    fn ssh_entry(alias: &str, host: &str) -> SshHostEntry {
        SshHostEntry {
            alias: alias.to_string(),
            host: host.to_string(),
            port: 22,
            username: "u".to_string(),
        }
    }

    fn resolve<'a>(fg: &'a ForegroundContext, cfg: &'a AppConfig, hosts: &'a [SshHostEntry]) -> RouteCandidate {
        let req = RouteRequest { foreground: fg, config: cfg, ssh_hosts: hosts };
        default_chain().resolve(&req).expect("chain should always resolve (LocalFallback)")
    }

    // 1. manual rule beats ssh-config and default ssh
    #[test]
    fn manual_rule_wins_over_ssh_config_and_default() {
        let cfg = config(
            vec![manual_ssh("dev", "manual-host", true)],
            Some(SshConfig { enabled: true, host: "default-host".to_string(), port: Some(22),
                username: None, remote_dir: "/tmp/img2cli".to_string(), match_pattern: None,
                remember_password: true }),
        );
        let hosts = vec![ssh_entry("dev", "autodetect-host")];
        let f = fg(Some("ssh: dev — terminal"));
        let c = resolve(&f, &cfg, &hosts);
        assert_eq!(c.source, RouteSource::ManualRule);
        match c.target {
            DeliveryTarget::Ssh(s) => assert_eq!(s.host, "manual-host"),
            _ => panic!("expected Ssh"),
        }
    }

    // 2. disabled manual rule is skipped → falls through to ssh-config
    #[test]
    fn disabled_manual_rule_is_skipped() {
        let cfg = config(vec![manual_ssh("dev", "manual-host", false)], None);
        let hosts = vec![ssh_entry("dev", "autodetect-host")];
        let c = resolve(&fg(Some("dev terminal")), &cfg, &hosts);
        assert_eq!(c.source, RouteSource::SshConfig);
    }

    // 3 & 4. ssh-config matches both alias and hostname; longest wins
    #[test]
    fn ssh_config_longest_alias_wins() {
        let hosts = vec![ssh_entry("dev", "10.0.0.1"), ssh_entry("dev-server", "10.0.0.2")];
        let c = resolve(&fg(Some("dev-server session")), &config(vec![], None), &hosts);
        assert_eq!(c.source, RouteSource::SshConfig);
        match c.target {
            DeliveryTarget::Ssh(s) => assert_eq!(s.host, "10.0.0.2"),
            _ => panic!("expected Ssh"),
        }
    }

    // 5. default ssh only when manual + ssh-config both miss
    #[test]
    fn default_ssh_when_no_match() {
        let cfg = config(
            vec![],
            Some(SshConfig { enabled: true, host: "default-host".to_string(), port: Some(22),
                username: None, remote_dir: "/tmp/img2cli".to_string(), match_pattern: None,
                remember_password: true }),
        );
        let c = resolve(&fg(Some("some unrelated window")), &cfg, &[]);
        assert_eq!(c.source, RouteSource::DefaultSsh);
    }

    // 6. default ssh disabled → local fallback
    #[test]
    fn local_fallback_when_default_ssh_disabled() {
        let cfg = config(
            vec![],
            Some(SshConfig { enabled: false, host: "x".to_string(), port: None,
                username: None, remote_dir: "/tmp".to_string(), match_pattern: None,
                remember_password: false }),
        );
        let c = resolve(&fg(Some("anything")), &cfg, &[]);
        assert_eq!(c.source, RouteSource::LocalFallback);
    }

    // 7. no window title → local fallback (can't match title-based rules)
    #[test]
    fn no_window_title_falls_back_locally() {
        let c = resolve(&fg(None), &config(vec![manual_ssh("dev", "h", true)], None), &[]);
        assert_eq!(c.source, RouteSource::LocalFallback);
    }

    // 8. empty match_pattern must NOT match all windows
    #[test]
    fn empty_pattern_does_not_match_all() {
        let cfg = config(vec![manual_ssh("", "should-not-win", true)], None);
        let c = resolve(&fg(Some("totally unrelated window")), &cfg, &[]);
        // empty-pattern rule is skipped → no match → local fallback
        assert_eq!(c.source, RouteSource::LocalFallback);
    }

    // 9. local target reachable via manual rule
    #[test]
    fn manual_local_rule_resolves() {
        let cfg = config(vec![manual_local("notes", "/home/u/notes")], None);
        let c = resolve(&fg(Some("notes app")), &cfg, &[]);
        assert_eq!(c.source, RouteSource::ManualRule);
        match c.target {
            DeliveryTarget::Local(p) => assert_eq!(p, PathBuf::from("/home/u/notes")),
            _ => panic!("expected Local"),
        }
    }

    // 10. first matching manual rule wins (order stability)
    #[test]
    fn first_matching_manual_rule_wins() {
        let cfg = config(
            vec![manual_ssh("dev", "first", true), manual_ssh("dev", "second", true)],
            None,
        );
        let c = resolve(&fg(Some("dev")), &cfg, &[]);
        match c.target {
            DeliveryTarget::Ssh(s) => assert_eq!(s.host, "first"),
            _ => panic!("expected Ssh"),
        }
    }

    // 11. manual resolver returns Ok(None) when nothing matches (not an error)
    #[test]
    fn manual_resolver_no_match_is_none_not_error() {
        let cfg = config(vec![manual_ssh("dev", "h", true)], None);
        let req = RouteRequest { foreground: &fg(Some("other")), config: &cfg, ssh_hosts: &[] };
        assert_eq!(ManualRuleResolver.resolve(&req).unwrap(), None);
    }

    // 12. ssh-config resolver: no hosts → no match
    #[test]
    fn ssh_config_no_hosts_no_match() {
        let req = RouteRequest {
            foreground: &fg(Some("dev")),
            config: &config(vec![], None),
            ssh_hosts: &[],
        };
        assert_eq!(SshConfigResolver.resolve(&req).unwrap(), None);
    }
}
