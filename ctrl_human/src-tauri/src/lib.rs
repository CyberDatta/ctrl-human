use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

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

#[derive(Debug, Serialize, Clone)]
struct PoseSummary {
    pose_id: String,
    title: String,
}

#[tauri::command]
fn load_poses(app: tauri::AppHandle) -> Result<Vec<PoseSummary>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let poses_file = app_data_dir.join("poses.json");

    let content = std::fs::read_to_string(&poses_file).unwrap_or_else(|_| "[]".to_string());
    let poses: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse poses.json: {}", e))?;

    let summaries = poses
        .as_array()
        .ok_or_else(|| "poses.json is not an array".to_string())?
        .iter()
        .map(|p| PoseSummary {
            pose_id: p.get("pose_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            title: p.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Pose").to_string(),
        })
        .collect();

    Ok(summaries)
}

#[tauri::command]
fn create_pose(app: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let poses_file = app_data_dir.join("poses.json");

    let content = std::fs::read_to_string(&poses_file).unwrap_or_else(|_| "[]".to_string());
    let mut poses: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse poses.json: {}", e))?;
    let poses_arr = poses
        .as_array_mut()
        .ok_or_else(|| "poses.json is not an array".to_string())?;

    // Generate a UUID that doesn't collide with any existing pose_id
    let pose_id = loop {
        let id = uuid::Uuid::new_v4().to_string();
        let duplicate = poses_arr
            .iter()
            .any(|p| p.get("pose_id").and_then(|v| v.as_str()) == Some(id.as_str()));
        if !duplicate {
            break id;
        }
    };

    let active_landmarks: Vec<bool> = vec![true; 33];
    let new_pose = serde_json::json!({
        "pose_id": pose_id,
        "title": "Unknown Pose",
        "description": "This is space to put a description.",
        "detection": {
            "method": "relative",
            "confidence": 0.5
        },
        "active_landmarks": active_landmarks,
        "images": []
    });

    poses_arr.push(new_pose);

    let serialized = serde_json::to_string_pretty(&poses)
        .map_err(|e| format!("Failed to serialize poses: {}", e))?;
    std::fs::write(&poses_file, serialized)
        .map_err(|e| format!("Failed to write poses.json: {}", e))?;

    Ok(pose_id)
}

#[derive(Debug, Serialize)]
struct PoseDetection {
    method: String,
    confidence: f64,
}

#[derive(Debug, Serialize)]
struct PoseFull {
    pose_id: String,
    title: String,
    description: String,
    detection: PoseDetection,
    active_landmarks: Vec<bool>,
}

#[tauri::command]
fn load_pose(app: tauri::AppHandle, pose_id: String) -> Result<PoseFull, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let poses_file = app_data_dir.join("poses.json");

    let content = std::fs::read_to_string(&poses_file).unwrap_or_else(|_| "[]".to_string());
    let poses: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse poses.json: {}", e))?;

    let pose = poses
        .as_array()
        .ok_or_else(|| "poses.json is not an array".to_string())?
        .iter()
        .find(|p| p.get("pose_id").and_then(|v| v.as_str()) == Some(pose_id.as_str()))
        .ok_or_else(|| format!("Pose {} not found", pose_id))?;

    let active_landmarks: Vec<bool> = pose
        .get("active_landmarks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(|v| v.as_bool().unwrap_or(true)).collect())
        .unwrap_or_else(|| vec![true; 33]);

    let detection = pose.get("detection").unwrap_or(&serde_json::Value::Null);

    Ok(PoseFull {
        pose_id: pose.get("pose_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        title: pose.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Pose").to_string(),
        description: pose.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        detection: PoseDetection {
            method: detection.get("method").and_then(|v| v.as_str()).unwrap_or("relative").to_string(),
            confidence: detection.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5),
        },
        active_landmarks,
    })
}

#[tauri::command]
fn save_pose(
    app: tauri::AppHandle,
    pose_id: String,
    title: String,
    description: String,
    detection_method: String,
    confidence: f64,
    active_landmarks: Vec<bool>,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let poses_file = app_data_dir.join("poses.json");

    let content = std::fs::read_to_string(&poses_file).unwrap_or_else(|_| "[]".to_string());
    let mut poses: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse poses.json: {}", e))?;

    let pose = poses
        .as_array_mut()
        .ok_or_else(|| "poses.json is not an array".to_string())?
        .iter_mut()
        .find(|p| p.get("pose_id").and_then(|v| v.as_str()) == Some(pose_id.as_str()))
        .ok_or_else(|| format!("Pose {} not found", pose_id))?;

    pose["title"] = serde_json::json!(title);
    pose["description"] = serde_json::json!(description);
    pose["detection"]["method"] = serde_json::json!(detection_method);
    pose["detection"]["confidence"] = serde_json::json!(confidence);
    pose["active_landmarks"] = serde_json::json!(active_landmarks);

    let serialized = serde_json::to_string_pretty(&poses)
        .map_err(|e| format!("Failed to serialize poses: {}", e))?;
    std::fs::write(&poses_file, serialized)
        .map_err(|e| format!("Failed to write poses.json: {}", e))?;

    Ok(())
}

#[tauri::command]
fn delete_pose(app: tauri::AppHandle, pose_id: String) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let poses_file = app_data_dir.join("poses.json");

    let content = std::fs::read_to_string(&poses_file).unwrap_or_else(|_| "[]".to_string());
    let mut poses: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse poses.json: {}", e))?;
    let poses_arr = poses
        .as_array_mut()
        .ok_or_else(|| "poses.json is not an array".to_string())?;

    poses_arr.retain(|p| p.get("pose_id").and_then(|v| v.as_str()) != Some(pose_id.as_str()));

    let serialized = serde_json::to_string_pretty(&poses)
        .map_err(|e| format!("Failed to serialize poses: {}", e))?;
    std::fs::write(&poses_file, serialized)
        .map_err(|e| format!("Failed to write poses.json: {}", e))?;

    Ok(())
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
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir)?;
            let poses_file = app_data_dir.join("poses.json");
            if !poses_file.exists() {
                std::fs::write(&poses_file, "[]")?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_webcams, select_webcam, load_poses, create_pose, delete_pose, load_pose, save_pose])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
