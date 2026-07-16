//! SSH client for password-based auth + SFTP upload (russh), with passwords
//! stored in the OS keyring (Xshell-style). Key-based servers keep using the
//! system `ssh`/`scp` binaries (see `daemon::upload_via_scp`).

use std::path::Path;
use std::sync::Arc;

use russh::{client, ChannelMsg, Disconnect};
use russh_sftp::client::SftpSession;
use tokio::io::AsyncWriteExt;

const SERVICE: &str = "img2cli";

/// Stable identity used as the keyring entry name for a host.
pub fn identity_key(user: &str, host: &str, port: Option<u16>) -> String {
    format!("{}@{}:{}", user, host, port.unwrap_or(22))
}

/// Look up a stored password for an identity (None if absent / unreadable).
pub fn get_stored_password(identity: &str) -> Option<String> {
    keyring::Entry::new(SERVICE, identity)
        .ok()
        .and_then(|e| e.get_password().ok())
}

/// Whether a password is stored for an identity (for UI status display).
pub fn has_stored_password(identity: &str) -> bool {
    keyring::Entry::new(SERVICE, identity)
        .map(|e| e.get_password().is_ok())
        .unwrap_or(false)
}

/// Store (or overwrite) a password for an identity in the OS keyring.
pub fn store_password(identity: &str, password: &str) -> Result<(), String> {
    let entry =
        keyring::Entry::new(SERVICE, identity).map_err(|e| format!("Keyring error: {}", e))?;
    entry
        .set_password(password)
        .map_err(|e| format!("Failed to save password: {}", e))
}

/// Delete a stored password for an identity (no-op if absent).
pub fn clear_password(identity: &str) -> Result<(), String> {
    match keyring::Entry::new(SERVICE, identity) {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("Failed to clear password: {}", e)),
        },
        Err(e) => Err(format!("Keyring error: {}", e)),
    }
}

/// Accept any host key (equivalent to `StrictHostKeyChecking=no`).
struct Handler;

impl client::Handler for Handler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

async fn connect_and_auth(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
) -> Result<client::Handle<Handler>, String> {
    let config = Arc::new(client::Config::default());
    let mut handle = client::connect(config, (host, port), Handler)
        .await
        .map_err(|e| format!("Connection error: {}", e))?;
    let authed = handle
        .authenticate_password(user, password)
        .await
        .map_err(|e| format!("Auth error: {}", e))?;
    if !authed.success() {
        return Err("Password authentication failed".to_string());
    }
    Ok(handle)
}

/// Async: SFTP-upload `local_path` to `remote_dir/<filename>`, creating the
/// directory first. Returns the remote path. Await this from an async context
/// (e.g. a Tauri command); do NOT call block_on from within another runtime.
pub async fn upload_via_sftp_async(
    host: String,
    port: u16,
    user: String,
    password: String,
    remote_dir: String,
    local_path: std::path::PathBuf,
) -> Result<String, String> {
    let filename = local_path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| "Invalid local file name".to_string())?;

    let mut handle = connect_and_auth(&host, port, &user, &password).await?;

    // mkdir -p the remote dir.
    let mut ch = handle
        .channel_open_session()
        .await
        .map_err(|e| format!("Open channel: {}", e))?;
    ch.exec(true, format!("mkdir -p -- '{}'", remote_dir))
        .await
        .map_err(|e| format!("mkdir: {}", e))?;
    while let Some(msg) = ch.wait().await {
        if matches!(msg, ChannelMsg::ExitStatus { .. }) {
            break;
        }
    }

    // SFTP put.
    let mut sch = handle
        .channel_open_session()
        .await
        .map_err(|e| format!("Open channel: {}", e))?;
    sch.request_subsystem(true, "sftp")
        .await
        .map_err(|e| format!("SFTP subsystem: {}", e))?;
    let sftp = SftpSession::new(sch.into_stream())
        .await
        .map_err(|e| format!("SFTP init: {}", e))?;

    let data = std::fs::read(&local_path).map_err(|e| format!("Read local file: {}", e))?;
    let mut file = sftp
        .create(format!("{}/{}", remote_dir, filename))
        .await
        .map_err(|e| format!("SFTP create: {}", e))?;
    file.write_all(&data)
        .await
        .map_err(|e| format!("SFTP write: {}", e))?;
    let _ = file.flush().await;
    let _ = file.shutdown().await;
    let _ = sftp.close().await;

    let _ = handle
        .disconnect(Disconnect::ByApplication, "bye", "en")
        .await;
    Ok(format!("{}/{}", remote_dir, filename))
}

/// Async: connect + password-auth only (for "Test Connection").
pub async fn test_password_async(
    host: String,
    port: u16,
    user: String,
    password: String,
) -> Result<(), String> {
    let mut handle = connect_and_auth(&host, port, &user, &password).await?;
    let _ = handle
        .disconnect(Disconnect::ByApplication, "bye", "en")
        .await;
    Ok(())
}

/// Sync wrapper for the (non-async) daemon worker thread: spin up a
/// single-threaded runtime, run the SFTP upload, return the remote path.
pub fn upload_via_sftp(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    remote_dir: &str,
    local_path: &Path,
) -> Result<String, String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Runtime error: {}", e))?;
    rt.block_on(upload_via_sftp_async(
        host.to_string(),
        port,
        user.to_string(),
        password.to_string(),
        remote_dir.to_string(),
        local_path.to_path_buf(),
    ))
}
