use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

fn python_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("src-python")
        .join("pygmy")
        .join("bin")
        .join("python")
}

fn bridge_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("src-python")
        .join("bridge.py")
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Webcam {
    pub index: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct ListWebcamsResponse {
    webcams: Vec<Webcam>,
}

#[tauri::command]
fn list_webcams() -> Result<Vec<Webcam>, String> {
    let output = Command::new(python_path())
        .arg(bridge_path())
        .arg("list_webcams")
        .output()
        .map_err(|e| format!("Failed to spawn Python: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python bridge error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: ListWebcamsResponse =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(response.webcams)
}

#[tauri::command]
fn select_webcam(index: u32) -> Result<String, String> {
    let output = Command::new(python_path())
        .arg(bridge_path())
        .arg("select_webcam")
        .arg(index.to_string())
        .output()
        .map_err(|e| format!("Failed to spawn Python: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python bridge error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_webcams, select_webcam])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
