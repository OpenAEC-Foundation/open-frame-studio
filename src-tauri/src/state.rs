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

/// Find the correct Python executable.
/// On Windows, Inkscape or other apps may shadow `python` in PATH.
/// We try `py -3` first (Windows launcher), then `python3`, then `python`.
pub fn python_command() -> tokio::process::Command {
    #[cfg(target_os = "windows")]
    {
        // Try the Windows Python Launcher first
        let mut cmd = tokio::process::Command::new("py");
        cmd.arg("-3");
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        tokio::process::Command::new("python3")
    }
}
