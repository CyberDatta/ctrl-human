import sys
import json
import os
import glob
import fcntl
import struct

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WEBCAM_SELECTION_FILE = os.path.join(SCRIPT_DIR, "engine", ".webcam_selection.json")

# V4L2 ioctl to query device capabilities
_VIDIOC_QUERYCAP = 0x80685600
_V4L2_CAP_VIDEO_CAPTURE = 0x00000001


def get_camera_name(index: int) -> str:
    """Try to get a human-readable camera name on Linux via v4l2."""
    try:
        with open(f"/sys/class/video4linux/video{index}/name") as f:
            return f.read().strip()
    except (FileNotFoundError, PermissionError):
        return f"Camera {index}"


def _is_capture_device(index: int) -> bool:
    """Return True only if /dev/videoN has V4L2_CAP_VIDEO_CAPTURE set.

    On Linux each physical camera exposes multiple device nodes (video, metadata,
    output…). isOpened() in OpenCV returns True for all of them, so probing with
    VideoCapture gives false positives. The ioctl check is instant and correct.
    """
    try:
        with open(f"/dev/video{index}", "rb") as f:
            buf = b"\x00" * 104  # sizeof(struct v4l2_capability)
            result = fcntl.ioctl(f.fileno(), _VIDIOC_QUERYCAP, buf)
            caps = struct.unpack_from("<I", result, 84)[0]  # .capabilities field
            return bool(caps & _V4L2_CAP_VIDEO_CAPTURE)
    except Exception:
        return False


def list_webcams() -> list[dict]:
    """Return all V4L2 capture-capable device nodes, skipping metadata/output nodes."""
    webcams = []
    for path in sorted(glob.glob("/dev/video*")):
        try:
            index = int(path.replace("/dev/video", ""))
        except ValueError:
            continue
        if _is_capture_device(index):
            name = get_camera_name(index)
            webcams.append({"index": index, "name": name})
    return webcams


def select_webcam(index: int) -> dict:
    """Save the selected webcam index so the engine can read it later."""
    os.makedirs(os.path.dirname(WEBCAM_SELECTION_FILE), exist_ok=True)
    with open(WEBCAM_SELECTION_FILE, "w") as f:
        json.dump({"selected_index": index}, f)
    return {"status": "ok", "selected": index}


def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "No action specified"}))
        sys.exit(1)

    action = sys.argv[1]

    if action == "list_webcams":
        result = {"webcams": list_webcams()}
    elif action == "select_webcam":
        if len(sys.argv) < 3:
            print(json.dumps({"error": "No webcam index specified"}))
            sys.exit(1)
        index = int(sys.argv[2])
        result = select_webcam(index)
    else:
        result = {"error": f"Unknown action: {action}"}
        print(json.dumps(result))
        sys.exit(1)

    print(json.dumps(result))


if __name__ == "__main__":
    main()
