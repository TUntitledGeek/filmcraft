//! Cloud synchronization services (Google Drive, cloud hubs, and remote project storage).

use serde::{Deserialize, Serialize};

/// Supported cloud storage hub providers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudProviderKind {
    GoogleDrive,
    CustomWebDav,
}

/// Google Drive OAuth and sync configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleDriveConfig {
    pub client_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub root_folder_name: String,
    pub active_folder_id: Option<String>,
    pub auto_sync_on_edit: bool,
}

impl Default for GoogleDriveConfig {
    fn default() -> Self {
        Self {
            client_id: "filmcraft-public-client.apps.googleusercontent.com".into(),
            access_token: None,
            refresh_token: None,
            root_folder_name: "FilmCraft Projects".into(),
            active_folder_id: None,
            auto_sync_on_edit: true,
        }
    }
}

impl GoogleDriveConfig {
    /// Generates the standard Google OAuth 2.0 authorization URL for user sign-in.
    pub fn authorization_url(&self, redirect_uri: &str, state_nonce: &str) -> String {
        let scope = "https://www.googleapis.com/auth/drive.file";
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}",
            self.client_id,
            redirect_uri,
            scope,
            state_nonce
        )
    }

    /// Whether an authenticated session is active.
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some() || self.refresh_token.is_some()
    }
}

/// Metadata for a project or asset file synchronized with the cloud.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudSyncFile {
    pub file_id: String,
    pub name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub md5_checksum: Option<String>,
    pub modified_time: String,
}

/// Progress state of an in-flight cloud upload or range stream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CloudTransferProgress {
    pub file_id: String,
    pub filename: String,
    pub transferred_bytes: u64,
    pub total_bytes: u64,
    pub is_upload: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn google_drive_oauth_url_generates_correctly() {
        let cfg = GoogleDriveConfig::default();
        let url = cfg.authorization_url("http://127.0.0.1:8765/oauth/callback", "nonce123");
        assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
        assert!(url.contains("client_id=filmcraft-public-client"));
        assert!(url.contains("drive.file"));
        assert!(url.contains("state=nonce123"));
    }

    #[test]
    fn auth_state_reflects_tokens() {
        let mut cfg = GoogleDriveConfig::default();
        assert!(!cfg.is_authenticated());

        cfg.access_token = Some("ya29.test_token".into());
        assert!(cfg.is_authenticated());
    }
}
