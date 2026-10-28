//! Browser persistence for the same named workspace preferences used by the desktop host.
//! Save only when preferences change, never during ordinary playback or grading frames.

use filmcraft_ui_egui::{FilmcraftApp, dock::WorkspacePrefs};

const KEY: &str = "filmcraft.workspaces.v1";
const MAX_BYTES: usize = 4 * 1024 * 1024;

fn storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .ok_or("Browser window unavailable")?
        .local_storage()
        .map_err(|_| "Browser workspace storage is blocked".to_string())?
        .ok_or_else(|| "Browser workspace storage is unavailable".into())
}

pub fn load() -> Option<WorkspacePrefs> {
    let text = storage().ok()?.get_item(KEY).ok()??;
    if text.len() > MAX_BYTES {
        log::warn!("Ignoring oversized workspace preferences");
        return None;
    }
    match serde_json::from_str(&text) {
        Ok(prefs) => Some(prefs),
        Err(error) => {
            log::warn!("Ignoring unreadable workspace preferences: {error}");
            None
        }
    }
}

pub struct Store {
    last: WorkspacePrefs,
}

impl Store {
    pub fn new(prefs: &WorkspacePrefs) -> Self {
        Self { last: prefs.clone() }
    }

    pub fn tick(&mut self, app: &mut FilmcraftApp) {
        if self.last == app.workspaces {
            return;
        }
        // A failed write reports once for this change; retry on the next explicit change,
        // rather than attempting synchronous storage writes on every UI frame.
        self.last = app.workspaces.clone();
        let result = (|| {
            let text = serde_json::to_string(&self.last).map_err(|e| e.to_string())?;
            if text.len() > MAX_BYTES {
                return Err("Workspace preferences exceed the 4 MiB limit".into());
            }
            storage()?.set_item(KEY, &text).map_err(|_| "Unable to save browser workspaces (storage quota or permissions)".to_string())
        })();
        if let Err(error) = result {
            app.ui.status = error;
        }
    }
}
