import sys
import json
import os

os.environ["OPENCV_LOG_LEVEL"] = "SILENT"
import cv2

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WEBCAM_SELECTION_FILE = os.path.join(SCRIPT_DIR, "engine", ".webcam_selection.json")


def get_camera_name(index: int) -> str:
    """Try to get a human-readable camera name on Linux via v4l2."""
    try:
        name_path = f"/sys/class/video4linux/video{index}/name"
        with open(name_path, "r") as f:
            return f.read().strip()
    except (FileNotFoundError, PermissionError):
        return f"Camera {index}"


def list_webcams() -> list[dict]:
    """Probe camera indices 0-9 and return available webcams."""
    webcams = []
    for i in range(10):
        cap = cv2.VideoCapture(i)
        if cap.isOpened():
            name = get_camera_name(i)
            webcams.append({"index": i, "name": name})
            cap.release()
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
