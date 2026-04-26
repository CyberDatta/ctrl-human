# ctrl-human

**Control video games with your body — no controller required.**

ctrl-human uses your webcam and MediaPipe pose detection to turn body positions into keyboard and mouse inputs. You define poses (a raised arm, a lean left, a crouch), wire each one to a key or mouse button or a combination of keys or mouse buttons, and run the controller while you play. The app watches your camera in real time and fires the right inputs whenever your body matches a saved pose.

---

## How it works

1. **Create poses** — open the Pose Library, snap a photo of yourself in a position, and the app extracts your body landmark data from the image using MediaPipe.
2. **Build a controller** — open the Controller Library, add your poses, and assign each one a keyboard key or mouse button combination.
3. **Play** — hit *Start Playing* on the home screen, pick your controller, and start moving. The app detects your current pose every frame and fires the mapped input.

---

## Prerequisites

Install these before anything else.

| Tool | Version | Notes |
|------|---------|-------|
| [Rust](https://rustup.rs) | stable | includes `cargo` and `rustc` |
| [Node.js](https://nodejs.org) | 18+ | |
| [pnpm](https://pnpm.io/installation) | 8+ | `npm install -g pnpm` |
| [Python](https://www.python.org/downloads/) | 3.10+ | must be on PATH |

**Linux only** — install Tauri's system dependencies:

```bash
# Ubuntu / Debian
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel

# Arch
sudo pacman -S webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg
```

**Windows only** — make sure [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) is installed. It ships with Windows 11 by default; on Windows 10 you may need to install it manually.

---

## Setup

### 1. Clone and enter the repo

```bash
git clone https://github.com/CyberDatta/ctrl-human.git
cd ctrl-human
```

### 2. Install frontend dependencies

```bash
cd ctrl_human
pnpm install
```

### 3. Set up the Python virtual environment

The Python sidecars (webcam listing, pose extraction, camera streaming) run inside a local venv called `pygmy`.

**Linux / macOS**

```bash
cd ctrl_human/src-python
python3 -m venv pygmy
source pygmy/bin/activate
pip install -r requirements.txt
deactivate
```

**Windows (PowerShell)**

```powershell
cd ctrl_human\src-python
python -m venv pygmy
pygmy\Scripts\Activate.ps1
pip install -r requirements.txt
deactivate
```

> The MediaPipe pose models are already included in `src-python/models/` — no separate download needed.

### 4. Build the Python sidecars

This compiles the three Python scripts into standalone binaries that Tauri can spawn at runtime. Run it once per machine; it takes around 5–10 minutes.

**Linux**

```bash
cd ctrl_human/src-python
bash build_sidecars.sh
```

**Windows (PowerShell)**

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
cd ctrl_human\src-python
.\build_sidecars.ps1
```

Binaries are written to `ctrl_human/src-tauri/binaries/`.

---

## Running in development

```bash
cd ctrl_human
pnpm tauri dev
```

This starts the SvelteKit dev server and the Tauri window together with hot reload.

---

## Building for distribution

```bash
cd ctrl_human
pnpm tauri build
```

The packaged app is placed in `ctrl_human/src-tauri/target/release/bundle/`.

---

## Tech stack

- **Frontend** — SvelteKit (Svelte 5) + TypeScript, Vite 6
- **Backend** — Tauri 2 (Rust)
- **Pose detection** — MediaPipe via Python sidecars bundled with PyInstaller
- **Input simulation** — [enigo](https://github.com/enigo-rs/enigo) (keyboard + mouse)
