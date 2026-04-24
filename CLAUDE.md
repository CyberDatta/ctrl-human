# ctrl-human / MoveIt — Project Context

## What this is
A Tauri 2 desktop app that lets you control your computer using body poses detected via webcam. You create "poses" (reference images + landmark data), wire them to keyboard/mouse inputs in a "controller", then run the controller to play games hands-free.

**Status: Prototype fully complete and working.** The user has tested it end-to-end and plays video games with it. Current work is Windows support and packaging/distribution.

---

## Stack
- **Frontend**: SvelteKit (Svelte 5) + TypeScript, Vite 6, adapter-static (SPA mode, fallback index.html)
- **Backend**: Tauri 2 (Rust), `src-tauri/src/lib.rs`
- **Python sidecars**: MediaPipe + OpenCV for pose detection, bundled via PyInstaller
- **Dev port**: 1420
- **Font**: Public Sans
- **Active branch**: `test`

---

## Key Paths
- **SvelteKit app**: `ctrl_human/`
- **Screens**: `ctrl_human/src/lib/screens/`
- **Routes**: `ctrl_human/src/routes/`
- **Icons**: `ctrl_human/src/lib/assets/icons/`
- **CSS tokens**: `ctrl_human/src/lib/styles/tokens.css`
- **Python scripts**: `ctrl_human/src-python/`
- **Rust backend**: `ctrl_human/src-tauri/src/lib.rs`
- **Sidecar binaries**: `ctrl_human/src-tauri/binaries/`

---

## Routes → Screens
- `/` → `HomeScreen.svelte`
- `/controller-studio` → `ControllerStudioScreen.svelte`
- `/controller-studio/pose-library` → `PoseLibraryScreen.svelte`
- `/controller-studio/pose-library/edit` → `PoseEditingScreen.svelte`
- `/controller-studio/pose-library/edit/capture` → `ImageCaptureScreen.svelte`
- `/controller-studio/pose-library/edit/test` → `PoseTestingScreen.svelte`
- `/controller-studio/controller-library` → `ControllerLibraryScreen.svelte`
- `/controller-studio/controller-library/edit` → `ControllerEditingScreen.svelte`
- `/controller-studio/controller-library/edit/test` → `ControllerTestingScreen.svelte`

**Naming convention**: "MainControllerScreen" = ControllerTestingScreen launched from HomeScreen via "Start Playing" (with `back=/&backText=Home` params). Use this shorthand freely.

---

## Design System

### Colors (tokens.css)
- `primary-1` #DE896A salmon, `primary-2` #36c5f1 cyan, `primary-3` #30b57f green, `primary-4` #74ff74 lime
- `secondary-1` #F24e1e orange, `secondary-2` #fec700 yellow, `secondary-3` #541554 purple, `secondary-4` #a9a6ff lavender
- `tertiary-1` #7df9ff, `tertiary-2` #ffb2ef pink, `tertiary-3` #FCDFFF pale pink
- `background` #F5F5F0, `window` #FF00F5 magenta

### Shadows & Strokes
- `shadow-s` 0.25rem, `shadow-m` 0.5rem, `shadow-l` 0.75rem (all black, no blur)
- `stroke-xs` 0.125rem, `stroke-s` 0.25rem, `stroke-m` 0.5rem

### Typography
- Huge (6rem italic 900), H1 (4rem 900), H2 (3rem 900), H3 (2.25rem 800), H4 (1.75rem 800), H5 (1.375rem 700)

### Style rules
- **No border-radius** — brutalist design
- Cards: stroke-s border, shadow-l
- CTAs: stroke-s border, shadow-m, H3 font, hover → mouse-hover bg + white text
- Back buttons: `chunky_arrow` SVG rotated 90° + text (white with purple outline)
- Navigation: `goto()` from `$app/navigation`

### Icons (SVG in `assets/icons/`)
`checked`, `unchecked`, `camera`, `magnifying_glass`, `pencil`, `trash_can`, `chunky_arrow`, `arrow_for_upload`, `arrow_for_download`, `image`, `cross`, `lab_flask`, `switch_card`, `ecstatic`

---

## Layout
- `ctrl_human/src/routes/+layout.svelte` — imports tokens.css, wraps `<slot />`
- `app.html` body: inline `background-color: #DE896A` to prevent white flash on load

---

## Backend — Tauri Commands

### Save System
- **Persistence**: manual JSON via `std::fs` in Rust (NOT `@tauri-apps/plugin-fs`)
- **app_data_dir**: `~/.local/share/com.cyberdatta.ctrl-human/` on Linux; Windows equivalent via `app.path().app_data_dir()`
- **poses.json**: top-level array of pose objects; auto-created with `[]` on startup
- **controllers.json**: top-level array; same dir as poses.json
- **images/**: `app_data_dir/images/<pose_id>/<image_id>.png`

### Pose schema
```
{
  pose_id, title, description,
  detection: { method: "relative"|"cosine", confidence: 0–1 },
  active_landmarks: boolean[33],
  images: [{ image_id, active: boolean, pose_values: [{x,y}×33] }]
}
```

### Controller schema
```
{ controller_id, title, description, poses: [{ pose_id, priority: u32, input_type, input: string[] }] }
```

### Tauri commands
- `list_webcams` → `Vec<Webcam>`
- `load_poses` → `Vec<{pose_id, title, thumbnail: Option<String>}]` (thumbnail = base64 data URI)
- `load_pose(pose_id)` → full pose without images
- `create_pose` → `String (pose_id)`
- `save_pose(pose_id, title, description, detection_method, confidence, active_landmarks)` → `()`
- `delete_pose(pose_id)` → `()`
- `save_pose_image(pose_id, image_data: base64)` → `String (image_id)` — runs `extract_pose.py`; returns Err("No pose detected") if mediapipe finds nothing
- `load_pose_images(pose_id)` → `Vec<{image_id, active, data_b64}>`
- `set_image_active(pose_id, image_id, active)` → `()`
- `delete_pose_image(pose_id, image_id)` → `()`
- `start_camera_stream(camera_index, with_inference: bool)` → `u16 (port)`
- `stop_camera_stream()` → `()`
- `load_pose_for_testing(pose_id)` → `{detection_method, confidence, active_landmarks, reference_values}`
- `load_controllers` → `Vec<{controller_id, title}>`
- `create_controller` → `String (controller_id)`
- `delete_controller(controller_id)` → `()`
- `load_controller(controller_id)` → `{controller_id, title, description}`
- `save_controller(controller_id, title, description, poses: Vec<PoseSave>)` → `()`

### Rust helpers (lib.rs)
- `get_sidecar_path(name)` — dev: resolves to `src-tauri/binaries/{name}-{TARGET_TRIPLE}`; release: sibling of main exe
- `EnigoState(Mutex<Option<Enigo>>)` — persistent single Enigo instance; critical for Press inputs (creating per-call causes immediate key release on drop)
- `CameraState(Mutex<Option<Child>>)` + `on_window_event(Destroyed)` — kills Python child on exit
- Linux only: `WEBKIT_DISABLE_DMABUF_RENDERER=1`, WebKit permission grant hook for camera/mic

---

## Python Sidecars

### Scripts
- **`bridge.py`**: webcam listing/selection via argv; stdlib only
- **`camera_server.py`**: HTTP server; `GET /frame` → JPEG, `GET /landmarks` → JSON `[{x,y}×33]|null`; MediaPipe VIDEO mode in separate thread; sets `QT_QPA_PLATFORM=offscreen`
- **`extract_pose.py`**: single-image landmark extraction; `extract_pose.py <image_path> <model_path>`; returns JSON or `null`

### Build workflow
- `build_sidecars.sh` — PyInstaller bundler; run once per machine before `pnpm tauri build`
- Output: `src-tauri/binaries/{bridge,camera_server,extract_pose}-<target-triple>`
- Python venv lives at `src-python/pygmy/`
- requirements: `mediapipe==0.10.33`, `opencv-contrib-python==4.13.0.92`
- **On Windows**: venv activates via `pygmy\Scripts\activate`; script needs PowerShell equivalent or Git Bash with path fix

### tauri.conf.json
- `productName`: `ctrl-human`
- `identifier`: `com.cyberdatta.ctrl-human`
- `externalBin`: `["binaries/bridge", "binaries/camera_server", "binaries/extract_pose"]`
- `resources`: `pose_landmarker_full.task`, `pose_landmarker_lite.task` → `models/`

---

## Camera Polling Pattern (do not change)
All screens that need live camera (ImageCaptureScreen, PoseTestingScreen, ControllerTestingScreen) use **single-frame HTTP polling** against `camera_server.py`, NOT `getUserMedia` or `<video>`. getUserMedia + video stays blank in WebKitGTK. Fetch ReadableStream hangs. MJPEG causes onerror. Polling with `fetch` + `arrayBuffer` is the only reliable approach.

---

## Screen Details

### PoseEditingScreen
- bg: tertiary-3; layout: main panel (flex:1) + sidebar (32.5rem)
- `onMount`: sync, chains async via `.then()` to avoid TS error
- Inline title/description editing with pencil buttons; commits on blur/Enter
- Skeleton viewport: 36.9rem × 34.25rem, tertiary-1 bg; 33 joints as 50×50 checkbox rects; zoom/pan via wheel+drag
- Detection card: yellow bg; pill toggle for method; confidence slider (cosine only)
- Sidebar: reference images with cross (remove) + checkbox (active toggle) overlaid absolutely

### ImageCaptureScreen
- bg: primary-3 green; camera via polling
- `pollGeneration` counter prevents stale loops when switching cameras
- Selfie countdown: 10s → `captureFrame()` → `save_pose_image` → appends to strip
- Captures strip: horizontal scroll, negative margin for edge-to-edge

### PoseTestingScreen
- bg: primary-3 green; camera + landmarks polled in parallel
- Cosine method: center-normalize both vectors → cosine similarity → remap to [0,1] → detect if ≥ confidence
- Relative method: rank-order x-coords and y-coords separately → detect if ranks match
- Skeleton overlay: white lines for POSE_CONNECTIONS, cyan dots left-body, orange dots right-body

### ControllerEditingScreen
- bg: tertiary-3; layout: main panel + sidebar (33.125rem)
- Pose rows: secondary-4 bg; priority + inputType dropdowns (primary-1 bg); input capture popup (keyboard + mouse)
- All changes persist immediately via `save_controller`
- `PoseRow.input`: display string "Ctrl + Left Click + g", saved split on ` + ` into array

### ControllerTestingScreen / MainControllerScreen
- Live camera + skeleton overlay + per-frame detection against all controller poses
- Priority-based: highest-priority matching pose wins; fires OS input via `enigo`
- Input types: Tap (single key press), Press (hold while pose held), Crazy Tap (rapid repeat)
- Single persistent `EnigoState` — do not create new Enigo per-call

---

## Input Types
- **Tap**: single keypress on pose detection
- **Press**: key held while pose is maintained (requires persistent Enigo instance)
- **Crazy Tap**: rapid repeated keypresses
- Supported: all keyboard keys + Left Click / Right Click / Middle Click
