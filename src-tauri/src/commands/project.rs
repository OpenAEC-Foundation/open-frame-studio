use crate::state::AppState;
use ofs_core::kozijn::Project;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::State;

#[tauri::command]
pub fn new_project(
    state: State<'_, AppState>,
    name: String,
    number: String,
) -> Result<Project, String> {
    let project = Project::new(&name, &number);
    let mut current = state.project.lock().map_err(|e| e.to_string())?;
    *current = project.clone();
    let mut path = state.project_path.lock().map_err(|e| e.to_string())?;
    *path = None;
    Ok(project)
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>) -> Result<Project, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    Ok(project.clone())
}

#[tauri::command]
pub fn open_project(state: State<'_, AppState>, file_path: String) -> Result<Project, String> {
    let contents = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let project: Project = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
    let mut current = state.project.lock().map_err(|e| e.to_string())?;
    *current = project.clone();
    let mut path = state.project_path.lock().map_err(|e| e.to_string())?;
    *path = Some(file_path);
    Ok(project)
}

#[tauri::command]
pub fn save_project(state: State<'_, AppState>, file_path: String) -> Result<(), String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&*project).map_err(|e| e.to_string())?;
    write_project_file(Path::new(&file_path), json.as_bytes())?;
    drop(project);
    let mut path = state.project_path.lock().map_err(|e| e.to_string())?;
    *path = Some(file_path);
    Ok(())
}

/// Write beside the target, then replace it. A failed write leaves the
/// previously saved .ofs file intact instead of truncating it first.
fn write_project_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if path.file_name().is_none() || !parent.is_dir() {
        return Err("Kies een geldig projectpad in een bestaande map".into());
    }
    let staging: PathBuf = parent.join(format!(".ofs-save-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&staging, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staging);
    }
    result.map_err(|e| format!("Project opslaan mislukt: {e}"))
}

#[cfg(test)]
mod tests {
    use super::write_project_file;

    #[test]
    fn replaces_existing_project_and_cleans_failed_staging() {
        let dir = std::env::temp_dir().join(format!("ofs-save-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let project = dir.join("project.ofs");
        std::fs::write(&project, b"old").unwrap();

        write_project_file(&project, b"new").unwrap();
        assert_eq!(std::fs::read(&project).unwrap(), b"new");

        let occupied_target = dir.join("occupied.ofs");
        std::fs::create_dir(&occupied_target).unwrap();
        assert!(write_project_file(&occupied_target, b"should fail").is_err());
        assert!(occupied_target.is_dir());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);

        std::fs::remove_file(&project).unwrap();
        std::fs::remove_dir(&occupied_target).unwrap();
        std::fs::remove_dir(&dir).unwrap();
    }
}
