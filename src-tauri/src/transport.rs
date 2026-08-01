//! Delivery transports: take a processed artifact and a routing target, put
//! the bytes where they need to go, and return the resulting path. Routing
//! (Step 8) decides the target; this layer only executes delivery.
//!
//! Auth selection stays HERE, not in routing: `SshTransport` asks the secret
//! store whether a password exists for the target's identity, then dispatches
//! to the russh SFTP transport (password) or the system-ssh SCP transport
//! (key). A keyring backend failure is a structured `TransportError::Keyring`
//! — it is NEVER silently treated as "no password", because that would mask a
//! broken keyring and silently downgrade to key auth.
//!
//! Trait and domain types avoid Tokio types on purpose: the trait is sync
//! (the job worker is a plain OS thread), and the SFTP implementation bridges
//! to the shared tokio runtime internally via `block_on`. If the pipeline ever
//! moves fully async, only this module changes.

use std::path::PathBuf;
use std::sync::Arc;

use crate::daemon;
use crate::routing::{DeliveryTarget, LocalTarget, SshTarget};
use crate::ssh;

/// A processed image ready to be delivered.
#[derive(Debug, Clone)]
pub struct ProcessedArtifact {
    pub local_path: PathBuf,
    pub filename: String,
}

/// The result of a delivery — the path to reference in the injected text
/// (remote path for SSH, local path for local copy).
#[derive(Debug, Clone)]
pub struct DeliveredArtifact {
    pub delivered_path: String,
}

#[derive(Debug, Clone)]
pub enum TransportError {
    /// The keyring backend could not be queried (distinct from "no password").
    Keyring(String),
    /// An upload (SFTP or SCP) failed.
    Upload(String),
    /// A local filesystem operation failed.
    LocalIo(String),
    /// A transport received a target type it doesn't handle.
    InvalidTarget(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Keyring(m) => write!(f, "keyring: {}", m),
            TransportError::Upload(m) => write!(f, "upload: {}", m),
            TransportError::LocalIo(m) => write!(f, "local I/O: {}", m),
            TransportError::InvalidTarget(m) => write!(f, "invalid target: {}", m),
        }
    }
}

// ── Secret store (so auth lookup is mockable + the 3 states are explicit) ─

#[derive(Debug, Clone)]
pub enum SecretError {
    /// The keyring backend exists but could not be read (e.g. D-Bus / Secret
    /// Service unavailable). Distinct from "no entry".
    Backend(String),
}

pub trait SecretStore: Send + Sync {
    /// `Ok(None)` = no password stored; `Ok(Some)` = the password;
    /// `Err(_)` = the keyring backend could not be queried.
    fn get_password(&self, identity: &str) -> Result<Option<String>, SecretError>;
}

/// OS-keyring-backed secret store (wraps `ssh::lookup_password`, which
/// distinguishes "no entry" from a backend error).
pub struct KeyringSecretStore;

impl SecretStore for KeyringSecretStore {
    fn get_password(&self, identity: &str) -> Result<Option<String>, SecretError> {
        ssh::lookup_password(identity).map_err(|e| SecretError::Backend(format!("{:?}", e)))
    }
}

// ── Transport trait ──────────────────────────────────────────────────────

pub trait ArtifactTransport: Send + Sync {
    fn deliver(
        &self,
        artifact: &ProcessedArtifact,
        target: &DeliveryTarget,
    ) -> Result<DeliveredArtifact, TransportError>;
}

// ── Local transport ──────────────────────────────────────────────────────

pub struct LocalTransport;

impl ArtifactTransport for LocalTransport {
    fn deliver(
        &self,
        artifact: &ProcessedArtifact,
        target: &DeliveryTarget,
    ) -> Result<DeliveredArtifact, TransportError> {
        let dir = match target {
            DeliveryTarget::Local(LocalTarget { dir }) => dir,
            _ => {
                return Err(TransportError::InvalidTarget(
                    "LocalTransport received a non-Local target".to_string(),
                ))
            }
        };
        let _ = std::fs::create_dir_all(dir);
        let final_path = dir.join(&artifact.filename);
        // Self-copy guard: when the target dir is where the temp file already
        // lives (the LocalFallback case), reuse it instead of copying a file
        // onto itself (which would truncate it to zero bytes).
        let local_path = if final_path == artifact.local_path {
            artifact.local_path.clone()
        } else if std::fs::copy(&artifact.local_path, &final_path).is_ok() {
            final_path
        } else {
            // Copy failed — fall back to the original temp path so the
            // injected link still points at a valid file.
            artifact.local_path.clone()
        };
        Ok(DeliveredArtifact {
            delivered_path: local_path.to_string_lossy().into_owned(),
        })
    }
}

// ── SSH transport (dispatcher: password → SFTP, else → SCP) ──────────────

pub struct SshTransport {
    secret_store: Arc<dyn SecretStore>,
}

impl SshTransport {
    pub fn new(secret_store: Arc<dyn SecretStore>) -> Self {
        Self { secret_store }
    }
}

impl ArtifactTransport for SshTransport {
    fn deliver(
        &self,
        artifact: &ProcessedArtifact,
        target: &DeliveryTarget,
    ) -> Result<DeliveredArtifact, TransportError> {
        let t: &SshTarget = match target {
            DeliveryTarget::Ssh(t) => t,
            _ => {
                return Err(TransportError::InvalidTarget(
                    "SshTransport received a non-Ssh target".to_string(),
                ))
            }
        };
        let remote_path = match self.secret_store.get_password(&t.identity()) {
            Ok(Some(pw)) => ssh::upload_via_sftp(
                &t.host, t.port, &t.username, &pw, &t.remote_dir, &artifact.local_path,
            )
            .map_err(|e| TransportError::Upload(e))?,
            Ok(None) => daemon::upload_via_scp(
                &artifact.local_path, &t.host, t.port, &t.username, &t.remote_dir,
            )
            .map_err(|e| TransportError::Upload(e))?,
            Err(se) => return Err(TransportError::Keyring(format!("{:?}", se))),
        };
        Ok(DeliveredArtifact {
            delivered_path: remote_path,
        })
    }
}

// ── Top-level dispatcher ─────────────────────────────────────────────────

pub struct DefaultTransport {
    local: LocalTransport,
    ssh: SshTransport,
}

impl ArtifactTransport for DefaultTransport {
    fn deliver(
        &self,
        artifact: &ProcessedArtifact,
        target: &DeliveryTarget,
    ) -> Result<DeliveredArtifact, TransportError> {
        match target {
            DeliveryTarget::Local(_) => self.local.deliver(artifact, target),
            DeliveryTarget::Ssh(_) => self.ssh.deliver(artifact, target),
        }
    }
}

/// The transports the pipeline uses. Auth resolution happens inside
/// `SshTransport` via the OS keyring.
pub fn default_transport() -> DefaultTransport {
    DefaultTransport {
        local: LocalTransport,
        ssh: SshTransport::new(Arc::new(KeyringSecretStore)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::{DeliveryTarget, LocalTarget, SshTarget};

    fn artifact() -> ProcessedArtifact {
        let dir = std::env::temp_dir().join("img2cli_transport_test");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("src.jpg");
        std::fs::write(&p, b"x").unwrap();
        ProcessedArtifact { local_path: p, filename: "src.jpg".into() }
    }

    fn ssh_target() -> SshTarget {
        SshTarget {
            host: "h".into(), port: 22, username: "u".into(),
            remote_dir: "/r".into(), source_alias: None,
        }
    }

    struct MockSecrets { has: bool, fail: bool }
    impl SecretStore for MockSecrets {
        fn get_password(&self, _id: &str) -> Result<Option<String>, SecretError> {
            if self.fail {
                return Err(SecretError::Backend("unavailable".into()));
            }
            Ok(if self.has { Some("pw".into()) } else { None })
        }
    }

    fn ssh_transport(has: bool, fail: bool) -> SshTransport {
        SshTransport::new(Arc::new(MockSecrets { has, fail }))
    }

    // local copy to a different dir → file lands at dir/filename
    #[test]
    fn local_transport_copies_to_target_dir() {
        let a = artifact();
        let out = std::env::temp_dir().join("img2cli_transport_out");
        let _ = std::fs::remove_dir_all(&out);
        let t = DeliveryTarget::Local(LocalTarget { dir: out.clone() });
        let d = LocalTransport.deliver(&a, &t).expect("local deliver");
        assert!(d.delivered_path.ends_with("src.jpg"));
        assert!(out.join("src.jpg").exists());
    }

    // self-copy guard: delivering into the file's own dir reuses it (no truncation)
    #[test]
    fn local_transport_self_copy_guard() {
        let a = artifact();
        let same_dir = a.local_path.parent().unwrap().to_path_buf();
        let t = DeliveryTarget::Local(LocalTarget { dir: same_dir });
        let d = LocalTransport.deliver(&a, &t).expect("local deliver");
        // source file must still be intact (not truncated by a self-copy)
        assert_eq!(std::fs::read(&a.local_path).unwrap(), b"x");
        assert!(d.delivered_path.ends_with("src.jpg"));
    }

    // wrong target type → InvalidTarget (defensive)
    #[test]
    fn local_transport_rejects_ssh_target() {
        let a = artifact();
        let t = DeliveryTarget::Ssh(ssh_target());
        assert!(matches!(
            LocalTransport.deliver(&a, &t),
            Err(TransportError::InvalidTarget(_))
        ));
    }

    // keyring backend failure → structured Keyring error, NOT a silent SCP
    // fallback. (No real network: get_password errors before any upload.)
    #[test]
    fn ssh_transport_keyring_failure_is_structured_error() {
        let a = artifact();
        let t = DeliveryTarget::Ssh(ssh_target());
        match ssh_transport(false, true).deliver(&a, &t) {
            Err(TransportError::Keyring(_)) => {}
            other => panic!("expected Keyring error, got {:?}", other),
        }
    }

    // wrong target type for SshTransport → InvalidTarget (defensive).
    // No real network: the target-type check fires before get_password.
    #[test]
    fn ssh_transport_rejects_local_target() {
        let a = artifact();
        let t = DeliveryTarget::Local(LocalTarget { dir: std::env::temp_dir() });
        assert!(matches!(
            ssh_transport(false, false).deliver(&a, &t),
            Err(TransportError::InvalidTarget(_))
        ));
    }

    // DefaultTransport is Send + Sync (compile-time assertion)
    #[test]
    fn default_transport_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>(_: T) {}
        assert_send_sync(default_transport());
    }
}
