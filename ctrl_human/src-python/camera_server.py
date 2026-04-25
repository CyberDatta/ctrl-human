#!/usr/bin/env python3
"""
MJPEG camera server with optional MediaPipe pose inference.
Usage: camera_server.py <camera_index> [--model <model_path>]
Prints the bound port to stdout immediately, then opens the camera and serves
individual JPEG frames at GET /frame and (when --model is given) pose landmarks
as JSON at GET /landmarks, until killed.
"""
import sys
import threading
import json
import os
import platform
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

os.environ["OPENCV_LOG_LEVEL"] = "SILENT"
os.environ["QT_QPA_PLATFORM"] = "offscreen"

import cv2

parser = argparse.ArgumentParser()
parser.add_argument("camera_index", type=int, nargs="?", default=0)
parser.add_argument("--model", default=None)
args = parser.parse_args()

camera_index: int = args.camera_index
model_path: str | None = args.model

# ── Shared frame state ──
# _latest_bgr is stored alongside _latest_frame so inference_loop can skip the
# JPEG decode round-trip (capture encodes BGR→JPEG; decoding it back wastes ~5-10ms).
# _frame_gen increments on every new frame so inference_loop can skip duplicates.
_frame_lock = threading.Lock()
_latest_frame: bytes | None = None
_latest_bgr = None   # numpy BGR array, same lifetime as _latest_frame
_frame_gen: int = 0  # monotonically increasing; read under _frame_lock

# ── Shared landmarks state ──
_landmarks_lock = threading.Lock()
_latest_landmarks: list | None = None  # [{x, y} × 33] or None when no pose


def capture_loop():
    global _latest_frame, _latest_bgr, _frame_gen
    if platform.system() == "Windows":
        # DirectShow is the reliable Windows backend; V4L2 is Linux-only
        cap = cv2.VideoCapture(camera_index, cv2.CAP_DSHOW)
    else:
        cap = cv2.VideoCapture(camera_index, cv2.CAP_V4L2)
        # Prefer MJPEG natively from the camera — avoids slow YUYV→BGR conversion
        # that causes colour artifacts on cameras that default to YUYV.
        cap.set(cv2.CAP_PROP_FOURCC, cv2.VideoWriter_fourcc(*"MJPG"))
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, 1280)
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 720)

    while True:
        ret, frame = cap.read()
        if not ret:
            continue
        _, jpeg = cv2.imencode(".jpg", frame, [cv2.IMWRITE_JPEG_QUALITY, 80])
        with _frame_lock:
            _latest_frame = jpeg.tobytes()
            _latest_bgr = frame   # cap.read() returns a fresh array each call
            _frame_gen += 1

    cap.release()


def inference_loop():
    """Continuously runs MediaPipe on new captured frames only."""
    global _latest_landmarks
    import time
    import mediapipe as mp

    BaseOptions = mp.tasks.BaseOptions
    PoseLandmarker = mp.tasks.vision.PoseLandmarker
    PoseLandmarkerOptions = mp.tasks.vision.PoseLandmarkerOptions
    VisionRunningMode = mp.tasks.vision.RunningMode

    # VIDEO mode enables temporal tracking between frames, which is faster and
    # smoother than IMAGE mode (which re-detects from scratch every frame).
    options = PoseLandmarkerOptions(
        base_options=BaseOptions(model_asset_path=model_path),
        running_mode=VisionRunningMode.VIDEO,
        min_pose_detection_confidence=0.3,
        min_pose_presence_confidence=0.3,
        min_tracking_confidence=0.3,
    )

    last_gen = -1
    with PoseLandmarker.create_from_options(options) as landmarker:
        while True:
            # Only process a frame if capture_loop has produced a new one
            with _frame_lock:
                gen = _frame_gen
                bgr = _latest_bgr

            if bgr is None or gen == last_gen:
                time.sleep(0.001)
                continue

            last_gen = gen
            timestamp_ms = int(time.monotonic() * 1000)

            # bgr is a fresh numpy array from cap.read() — safe to use without copying
            rgb = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB)
            mp_image = mp.Image(image_format=mp.ImageFormat.SRGB, data=rgb)

            result = landmarker.detect_for_video(mp_image, timestamp_ms)
            if result.pose_landmarks:
                lms = result.pose_landmarks[0]
                landmarks = [{"x": float(lm.x), "y": float(lm.y)} for lm in lms]
            else:
                landmarks = None

            with _landmarks_lock:
                _latest_landmarks = landmarks


def get_frame() -> bytes | None:
    with _frame_lock:
        return _latest_frame


def get_landmarks() -> list | None:
    with _landmarks_lock:
        return _latest_landmarks


class FrameHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def do_GET(self):
        if self.path == "/landmarks":
            self._serve_landmarks()
        else:
            self._serve_frame()

    def _serve_frame(self):
        data = get_frame()
        if data is None:
            self.send_response(503)
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            return
        self.send_response(200)
        self.send_header("Content-Type", "image/jpeg")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        try:
            self.wfile.write(data)
        except (BrokenPipeError, ConnectionResetError):
            pass

    def _serve_landmarks(self):
        body = json.dumps(get_landmarks()).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        try:
            self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            pass


# Bind the server first so the port is live before we print it
server = ThreadingHTTPServer(("127.0.0.1", 0), FrameHandler)
port = server.server_address[1]

# Tell the Rust parent which port we're on — server is already listening
print(port, flush=True)

# Capture runs in a dedicated thread; HTTP threads only read _latest_frame
threading.Thread(target=capture_loop, daemon=True).start()

# Inference thread only starts when a model path was supplied
if model_path:
    threading.Thread(target=inference_loop, daemon=True).start()

try:
    server.serve_forever()
finally:
    pass
