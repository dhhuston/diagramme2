//! Global project mutex registered with Tauri via `.manage()`.

use diagramme_schema::ProjectState;
use std::sync::Mutex;

pub struct AppState(pub Mutex<ProjectState>);
