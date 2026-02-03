// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{
    sync::{Mutex, Arc, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
    process::{Command, Stdio},
    io::{BufReader, BufRead},
    env
};
use tauri::{State, Emitter};

struct StreamCtrl {
    running: AtomicBool,
}

#[tauri::command]
fn start_stream(app: tauri::AppHandle, ctrl: State<Arc<StreamCtrl>>) -> Result<(), String> {
    if ctrl.running.swap(true, Ordering::SeqCst) {
        return Ok(()); // already running
    }

    // Get the current working directory (Tauri project root)
    let current_dir = env::current_dir().map_err(|e| format!("Failed to get current dir: {}", e))?;
    let python_script_path = current_dir.join("../engine/pose_tracking.py");

    println!("Python script path: {}", python_script_path.display());

    // Spawn the Python process for pose tracking
    let mut child = Command::new("python3")
        .arg(python_script_path)  // Path to Python script
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Python process: {}", e))?;

    // Capture Python output (stdout) asynchronously
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Parse JSON payload from Python
                    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&line) {
                        // Emit event immediately after parsing
                        let _ = app.emit("stream:data", payload);
                    }
                }
                Err(_) => break, // End the loop if there's an error
            }

            // Sleep to ensure 30 frames per second
            thread::sleep(Duration::from_millis(33)); // 33ms for 30 FPS
        }

        // Notify when the stream has stopped
        let _ = app.emit("stream:stopped", serde_json::json!({}));
    });

    Ok(())
}
#[tauri::command]
fn stop_stream(ctrl: State<Arc<StreamCtrl>>) -> Result<(), String> {
    let ctrl = ctrl.inner().clone();
    ctrl.running.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct Counter(Mutex<i32>);

#[tauri::command]
fn increment(counter: State<Counter>) -> i32 {
    let mut value = counter.0.lock().unwrap();
    *value += 1;
    *value
}

#[tauri::command]
fn reset(counter: State<Counter>) -> i32 {
    let mut value = counter.0.lock().unwrap();
    *value = 0;
    *value
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Counter(Mutex::new(0)))
        .manage(Arc::new(StreamCtrl { running: AtomicBool::new(false) }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, increment, reset, start_stream, stop_stream])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
