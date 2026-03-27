use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::Manager;

// ── Camera MJPEG server state ──
struct CameraState(Mutex<Option<Child>>);

impl Drop for CameraState {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.0.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait(); // reap the zombie so the OS releases the device
            }
        }
    }
}

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

fn camera_server_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("src-python")
        .join("camera_server.py")
}

fn extract_pose_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("src-python")
        .join("extract_pose.py")
}

fn pose_model_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("src-python")
        .join("models")
        .join("pose_landmarker_full.task")
}

fn pose_model_lite_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("src-python")
        .join("models")
        .join("pose_landmarker_lite.task")
}

fn default_pose_values() -> Vec<serde_json::Value> {
    (0..33).map(|_| serde_json::json!({ "x": 0.0, "y": 0.0 })).collect()
}

fn python_ld_library_path() -> String {
    let existing = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
    let sys = "/usr/lib/x86_64-linux-gnu:/usr/lib";
    if existing.is_empty() { sys.to_string() } else { format!("{}:{}", sys, existing) }
}

#[tauri::command]
fn start_camera_stream(
    state: tauri::State<CameraState>,
    camera_index: u32,
    with_inference: bool,
) -> Result<u16, String> {
    let mut guard = state.0.lock().unwrap();

    // Stop any existing camera server
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
    }

    let mut cmd = Command::new(python_path());
    cmd.arg(camera_server_path())
        .arg(camera_index.to_string())
        .env("LD_LIBRARY_PATH", python_ld_library_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if with_inference {
        cmd.arg("--model").arg(pose_model_lite_path());
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start camera server: {}", e))?;

    // The script prints the bound port on its first stdout line
    let stdout = child.stdout.take().ok_or("No stdout from camera server")?;
    use std::io::{BufRead, BufReader};
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("Failed to read port: {}", e))?;
    let port: u16 = line
        .trim()
        .parse()
        .map_err(|e| format!("Invalid port '{}': {}", line.trim(), e))?;

    *guard = Some(child);
    Ok(port)
}

#[tauri::command]
fn stop_camera_stream(state: tauri::State<CameraState>) {
    let mut guard = state.0.lock().unwrap();
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
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
    thumbnail: Option<String>,
}

#[tauri::command]
fn load_poses(app: tauri::AppHandle) -> Result<Vec<PoseSummary>, String> {
    use base64::Engine;

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
        .map(|p| {
            let pose_id = p.get("pose_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let title = p.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Pose").to_string();
            let thumbnail = p
                .get("images")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|img| img.get("image_id")?.as_str().map(|s| s.to_string()))
                .and_then(|image_id| {
                    let path = app_data_dir.join("images").join(&pose_id).join(format!("{}.png", image_id));
                    std::fs::read(&path).ok().map(|bytes| {
                        format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&bytes))
                    })
                });
            PoseSummary { pose_id, title, thumbnail }
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

    // Create the images directory for this pose
    let images_dir = app_data_dir.join("images").join(&pose_id);
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create image dir: {}", e))?;

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

    // Remove the images directory for this pose
    let images_dir = app_data_dir.join("images").join(&pose_id);
    if images_dir.exists() {
        std::fs::remove_dir_all(&images_dir)
            .map_err(|e| format!("Failed to delete image dir: {}", e))?;
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct PoseImageInfo {
    image_id: String,
    active: bool,
    data_b64: String,
}

#[tauri::command]
fn load_pose_images(app: tauri::AppHandle, pose_id: String) -> Result<Vec<PoseImageInfo>, String> {
    use base64::Engine;

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

    let images = pose
        .get("images")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|img| {
                    let image_id = img.get("image_id")?.as_str()?.to_string();
                    let active = img.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                    let file_path = app_data_dir
                        .join("images")
                        .join(&pose_id)
                        .join(format!("{}.png", &image_id));
                    let bytes = std::fs::read(&file_path).ok()?;
                    let data_b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    Some(PoseImageInfo { image_id, active, data_b64 })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(images)
}

#[tauri::command]
fn delete_pose_image(
    app: tauri::AppHandle,
    pose_id: String,
    image_id: String,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

    // Delete the image file
    let image_path = app_data_dir
        .join("images")
        .join(&pose_id)
        .join(format!("{}.png", image_id));
    if image_path.exists() {
        std::fs::remove_file(&image_path)
            .map_err(|e| format!("Failed to delete image file: {}", e))?;
    }

    // Remove entry from poses.json
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

    if let Some(images) = pose.get_mut("images").and_then(|v| v.as_array_mut()) {
        images.retain(|img| img.get("image_id").and_then(|v| v.as_str()) != Some(image_id.as_str()));
    }

    let serialized = serde_json::to_string_pretty(&poses)
        .map_err(|e| format!("Failed to serialize poses: {}", e))?;
    std::fs::write(&poses_file, serialized)
        .map_err(|e| format!("Failed to write poses.json: {}", e))?;

    Ok(())
}

#[tauri::command]
fn set_image_active(
    app: tauri::AppHandle,
    pose_id: String,
    image_id: String,
    active: bool,
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

    let image = pose
        .get_mut("images")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "No images array".to_string())?
        .iter_mut()
        .find(|img| img.get("image_id").and_then(|v| v.as_str()) == Some(image_id.as_str()))
        .ok_or_else(|| format!("Image {} not found", image_id))?;

    image["active"] = serde_json::json!(active);

    let serialized = serde_json::to_string_pretty(&poses)
        .map_err(|e| format!("Failed to serialize poses: {}", e))?;
    std::fs::write(&poses_file, serialized)
        .map_err(|e| format!("Failed to write poses.json: {}", e))?;

    Ok(())
}

#[tauri::command]
fn save_pose_image(
    app: tauri::AppHandle,
    pose_id: String,
    image_data: String, // base64-encoded PNG
) -> Result<String, String> {
    use base64::Engine;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&image_data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

    let image_id = uuid::Uuid::new_v4().to_string();
    let image_path = app_data_dir
        .join("images")
        .join(&pose_id)
        .join(format!("{}.png", image_id));
    std::fs::write(&image_path, &bytes)
        .map_err(|e| format!("Failed to write image: {}", e))?;

    // Add image entry to poses.json
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

    // Run MediaPipe pose extraction on the saved image
    let pose_values: Vec<serde_json::Value> = {
        let output = Command::new(python_path())
            .arg(extract_pose_path())
            .arg(&image_path)
            .arg(pose_model_path())
            .env("LD_LIBRARY_PATH", python_ld_library_path())
            .stderr(Stdio::inherit())
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                match serde_json::from_str::<serde_json::Value>(stdout.trim()) {
                    Ok(serde_json::Value::Array(arr)) if arr.len() == 33 => arr,
                    _ => {
                        // null or unexpected output — no pose detected, discard the image
                        let _ = std::fs::remove_file(&image_path);
                        return Err("No pose detected in the captured image".to_string());
                    }
                }
            }
            _ => {
                let _ = std::fs::remove_file(&image_path);
                return Err("Pose extraction failed".to_string());
            }
        }
    };

    let new_image = serde_json::json!({
        "image_id": image_id,
        "active": false,
        "pose_values": pose_values
    });

    if let Some(images) = pose.get_mut("images").and_then(|v| v.as_array_mut()) {
        images.push(new_image);
    } else {
        pose["images"] = serde_json::json!([new_image]);
    }

    let serialized = serde_json::to_string_pretty(&poses)
        .map_err(|e| format!("Failed to serialize poses: {}", e))?;
    std::fs::write(&poses_file, serialized)
        .map_err(|e| format!("Failed to write poses.json: {}", e))?;

    Ok(image_id)
}

#[derive(Debug, Serialize)]
struct PosePointF {
    x: f64,
    y: f64,
}

#[derive(Debug, Serialize)]
struct PoseTestingData {
    detection_method: String,
    confidence: f64,
    active_landmarks: Vec<bool>,
    /// pose_values arrays from active images only (no image bytes)
    reference_values: Vec<Vec<PosePointF>>,
}

#[tauri::command]
fn load_pose_for_testing(app: tauri::AppHandle, pose_id: String) -> Result<PoseTestingData, String> {
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

    let detection = pose.get("detection").unwrap_or(&serde_json::Value::Null);
    let detection_method = detection
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("relative")
        .to_string();
    let confidence = detection
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5);

    let active_landmarks: Vec<bool> = pose
        .get("active_landmarks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(|v| v.as_bool().unwrap_or(true)).collect())
        .unwrap_or_else(|| vec![true; 33]);

    let reference_values: Vec<Vec<PosePointF>> = pose
        .get("images")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|img| {
                    img.get("active")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                })
                .filter_map(|img| {
                    let vals = img.get("pose_values")?.as_array()?;
                    let points: Vec<PosePointF> = vals
                        .iter()
                        .filter_map(|pt| {
                            let x = pt.get("x")?.as_f64()?;
                            let y = pt.get("y")?.as_f64()?;
                            Some(PosePointF { x, y })
                        })
                        .collect();
                    if points.is_empty() { None } else { Some(points) }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(PoseTestingData { detection_method, confidence, active_landmarks, reference_values })
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
    // Force WebKitGTK to use the software renderer instead of DMABuf.
    // Without this, camera streams via getUserMedia render as a blank white
    // video element even though the MediaStream is active.
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    tauri::Builder::default()
        .manage(CameraState(Mutex::new(None)))
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let state = window.app_handle().state::<CameraState>();
                let mut guard = state.0.lock().unwrap();
                if let Some(mut child) = guard.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        })
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir)?;

            let poses_file = app_data_dir.join("poses.json");
            if !poses_file.exists() {
                std::fs::write(&poses_file, "[]")?;
            }

            // Ensure images/ dir exists and every existing pose has a subdirectory
            let images_dir = app_data_dir.join("images");
            std::fs::create_dir_all(&images_dir)?;

            let content = std::fs::read_to_string(&poses_file).unwrap_or_else(|_| "[]".to_string());
            if let Ok(poses) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(arr) = poses.as_array() {
                    for pose in arr {
                        if let Some(id) = pose.get("pose_id").and_then(|v| v.as_str()) {
                            let _ = std::fs::create_dir_all(images_dir.join(id));
                        }
                    }
                }
            }

            // Grant camera/media permissions in the WebKitGTK webview on Linux
            #[cfg(target_os = "linux")]
            {
                let win = app.get_webview_window("main").expect("no main window");
                win.with_webview(|wv| {
                    use webkit2gtk::{WebViewExt, PermissionRequestExt};
                    wv.inner().connect_permission_request(|_, req| {
                        req.allow();
                        true
                    });
                })?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_webcams, select_webcam, load_poses, create_pose, delete_pose, load_pose, save_pose, save_pose_image, load_pose_images, set_image_active, delete_pose_image, start_camera_stream, stop_camera_stream, load_pose_for_testing])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
