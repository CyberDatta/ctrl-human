#!/usr/bin/env python3
"""
Extract MediaPipe Pose Landmarker normalized coordinates from an image.
Usage: extract_pose.py <image_path> <model_path>
Prints a JSON array of 33 {"x": ..., "y": ...} objects to stdout.
Falls back to all zeros if no pose is detected or an error occurs.
"""
import sys
import json
import os

os.environ["QT_QPA_PLATFORM"] = "offscreen"


def extract(image_path: str, model_path: str) -> list[dict] | None:
    import mediapipe as mp

    BaseOptions = mp.tasks.BaseOptions
    PoseLandmarker = mp.tasks.vision.PoseLandmarker
    PoseLandmarkerOptions = mp.tasks.vision.PoseLandmarkerOptions
    VisionRunningMode = mp.tasks.vision.RunningMode

    options = PoseLandmarkerOptions(
        base_options=BaseOptions(model_asset_path=model_path),
        running_mode=VisionRunningMode.IMAGE,
        min_pose_detection_confidence=0.2,
        min_pose_presence_confidence=0.2,
        min_tracking_confidence=0.2,
    )

    with PoseLandmarker.create_from_options(options) as landmarker:
        image = mp.Image.create_from_file(image_path)
        result = landmarker.detect(image)

        if not result.pose_landmarks:
            print("mediapipe: no pose landmarks returned", file=sys.stderr, flush=True)
            return None

        landmarks = result.pose_landmarks[0]
        return [{"x": float(lm.x), "y": float(lm.y)} for lm in landmarks]


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("extract_pose.py: missing arguments", file=sys.stderr, flush=True)
        print(json.dumps(None), flush=True)
        sys.exit(0)

    try:
        result = extract(sys.argv[1], sys.argv[2])
    except Exception as e:
        print(f"extract_pose.py error: {e}", file=sys.stderr, flush=True)
        result = None

    print(json.dumps(result), flush=True)
