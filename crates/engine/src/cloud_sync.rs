//! Cloud synchronization services (Google Drive, YouTube Squad folders, and real-time project sharing).
//!
//! Provides Google Drive v3 REST API bindings, HTTP Range-request media streaming helpers,
//! YouTube squad collab folders, and 1-click shareable project URLs (`?room=...&drive=...`).

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
    pub active_folder_name: Option<String>,
    pub auto_sync_on_edit: bool,
    /// The YouTube squad / collaboration room ID for live peer syncing.
    pub squad_room_id: Option<String>,
}

impl Default for GoogleDriveConfig {
    fn default() -> Self {
        Self {
            client_id: "filmcraft-public-client.apps.googleusercontent.com".into(),
            access_token: None,
            refresh_token: None,
            root_folder_name: "FilmCraft Projects".into(),
            active_folder_id: None,
            active_folder_name: None,
            auto_sync_on_edit: true,
            squad_room_id: None,
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

/// YouTube squad shared channel folder metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveSquadFolder {
    pub folder_id: String,
    pub name: String,
    pub project_files: Vec<CloudSyncFile>,
    pub collaborators: Vec<String>,
    pub is_synced: bool,
}

/// Parameters decoded from a 1-click share URL.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareParams {
    pub room_id: String,
    pub drive_id: String,
    pub project_name: String,
    pub host_name: String,
}

/// Simple percent-encoding for URI query parameters.
pub fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Simple percent-decoding for URI query parameters.
pub fn url_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'+' {
            bytes.push(b' ');
        } else if b == b'%' {
            let h1 = chars.next().unwrap_or(b'0');
            let h2 = chars.next().unwrap_or(b'0');
            let hex_str = [h1, h2];
            if let Ok(s) = std::str::from_utf8(&hex_str) {
                if let Ok(val) = u8::from_str_radix(s, 16) {
                    bytes.push(val);
                }
            }
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Builds the standard HTTP Range header tuple for chunked media streaming.
pub fn build_range_header(start: u64, end: Option<u64>) -> (&'static str, String) {
    let header_val = match end {
        Some(e) => format!("bytes={start}-{e}"),
        None => format!("bytes={start}-"),
    };
    ("Range", header_val)
}

/// Returns the Google Drive API v3 endpoint for retrieving file metadata or media content.
pub fn drive_file_media_url(file_id: &str) -> String {
    format!("https://www.googleapis.com/drive/v3/files/{file_id}?alt=media")
}

/// Returns the Google Drive API v3 endpoint for listing files inside a folder.
pub fn drive_list_files_url(folder_id: &str) -> String {
    let query = format!("'{}' in parents and trashed = false", folder_id);
    format!("https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,name,mimeType,size,modifiedTime)", url_encode(&query))
}

/// Returns the Google Drive API v3 endpoint for creating files or folders.
pub fn drive_create_file_url() -> String {
    "https://www.googleapis.com/drive/v3/files".to_string()
}

/// Generates a 1-click shareable collaboration link for friends.
pub fn generate_share_url(base_url: &str, params: &ShareParams) -> String {
    let base = base_url.trim_end_matches('/');
    format!(
        "{}/?room={}&drive={}&project={}&host={}",
        base,
        url_encode(&params.room_id),
        url_encode(&params.drive_id),
        url_encode(&params.project_name),
        url_encode(&params.host_name)
    )
}

/// Parses share parameters from a URL query string.
pub fn parse_share_url(url_or_query: &str) -> Option<ShareParams> {
    let query_str = if let Some(idx) = url_or_query.find('?') {
        &url_or_query[idx + 1..]
    } else {
        url_or_query
    };

    let mut room_id = None;
    let mut drive_id = None;
    let mut project_name = None;
    let mut host_name = None;

    for part in query_str.split('&') {
        let mut kv = part.splitn(2, '=');
        let k = kv.next()?;
        let v = kv.next().unwrap_or("");
        let decoded = url_decode(v);

        match k {
            "room" => room_id = Some(decoded),
            "drive" => drive_id = Some(decoded),
            "project" => project_name = Some(decoded),
            "host" => host_name = Some(decoded),
            _ => {}
        }
    }

    Some(ShareParams {
        room_id: room_id?,
        drive_id: drive_id.unwrap_or_else(|| "default-drive-folder".into()),
        project_name: project_name.unwrap_or_else(|| "YouTube Collab".into()),
        host_name: host_name.unwrap_or_else(|| "Host".into()),
    })
}

/// Returns a default mock YouTube Squad folder with sample collaboration projects.
pub fn default_squad_folder() -> DriveSquadFolder {
    DriveSquadFolder {
        folder_id: "squad_yt_collab_2026".into(),
        name: "YouTube Squad Collab".into(),
        project_files: vec![
            CloudSyncFile {
                file_id: "file_ep4_montage".into(),
                name: "Squad_Montage_Ep4.filmcraft".into(),
                mime_type: "application/json".into(),
                size_bytes: 2_450_000,
                md5_checksum: Some("9a8b7c6d5e".into()),
                modified_time: "2026-10-27T16:30:00Z".into(),
            },
            CloudSyncFile {
                file_id: "file_shorts_v2".into(),
                name: "Viral_Shorts_Pack_V2.filmcraft".into(),
                mime_type: "application/json".into(),
                size_bytes: 1_120_000,
                md5_checksum: Some("4f3e2d1c0b".into()),
                modified_time: "2026-10-27T14:15:00Z".into(),
            },
            CloudSyncFile {
                file_id: "file_gaming_raw".into(),
                name: "Clutch_Clip_Raw.mp4".into(),
                mime_type: "video/mp4".into(),
                size_bytes: 480_000_000,
                md5_checksum: Some("7c8d9e0f1a".into()),
                modified_time: "2026-10-27T12:00:00Z".into(),
            },
        ],
        collaborators: vec!["Sebastian (You)".into(), "Alex".into(), "Marcus".into()],
        is_synced: true,
    }
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

    #[test]
    fn test_share_url_generation_and_parsing() {
        let params = ShareParams {
            room_id: "yt-squad-404".into(),
            drive_id: "gdrive-collab-folder".into(),
            project_name: "Summer Highlights 4K".into(),
            host_name: "Sebastian".into(),
        };

        let url = generate_share_url("https://tuntitledgeek.github.io/filmcraft", &params);
        assert!(url.contains("room=yt-squad-404"));
        assert!(url.contains("drive=gdrive-collab-folder"));

        let parsed = parse_share_url(&url).expect("must parse url");
        assert_eq!(parsed.room_id, "yt-squad-404");
        assert_eq!(parsed.drive_id, "gdrive-collab-folder");
        assert_eq!(parsed.project_name, "Summer Highlights 4K");
        assert_eq!(parsed.host_name, "Sebastian");
    }

    #[test]
    fn test_build_range_header() {
        let (hdr, val) = build_range_header(0, Some(1048575));
        assert_eq!(hdr, "Range");
        assert_eq!(val, "bytes=0-1048575");

        let (_, open_val) = build_range_header(2048, None);
        assert_eq!(open_val, "bytes=2048-");
    }
}
