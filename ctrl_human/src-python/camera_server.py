#!/usr/bin/env python3
"""
MJPEG camera server.
Usage: camera_server.py <camera_index>
Prints the bound port to stdout immediately, then opens the camera and serves
individual JPEG frames at GET /frame until killed.
"""
import sys
import threading
import time
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

os.environ["OPENCV_LOG_LEVEL"] = "SILENT"
os.environ["QT_QPA_PLATFORM"] = "offscreen"

import cv2


camera_index = int(sys.argv[1]) if len(sys.argv) > 1 else 0

# cap starts as None; get_frame() returns None until the camera is ready
cap = None


def get_frame() -> bytes | None:
    if cap is None:
        return None
    ret, frame = cap.read()
    if not ret:
        return None
    _, jpeg = cv2.imencode(".jpg", frame, [cv2.IMWRITE_JPEG_QUALITY, 80])
    return jpeg.tobytes()


class FrameHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def do_GET(self):
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


# Bind the server first so the port is live before we print it
server = ThreadingHTTPServer(("127.0.0.1", 0), FrameHandler)
port = server.server_address[1]

# Tell the Rust parent which port we're on — server is already listening
print(port, flush=True)


# Open the camera in a background thread so serve_forever() starts immediately
def open_camera():
    global cap
    cap = cv2.VideoCapture(camera_index)


threading.Thread(target=open_camera, daemon=True).start()

try:
    server.serve_forever()
finally:
    if cap is not None:
        cap.release()
