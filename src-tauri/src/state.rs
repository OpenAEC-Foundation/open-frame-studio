use ofs_core::kozijn::Project;
use std::sync::Mutex;

pub struct AppState {
    pub project: Mutex<Project>,
    pub project_path: Mutex<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            project: Mutex::new(Project::new("Nieuw project", "")),
            project_path: Mutex::new(None),
        }
    }
}
