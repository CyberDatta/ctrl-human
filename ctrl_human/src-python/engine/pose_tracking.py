import time
import random
import json

# Simulate pose tracking every 33ms (for 30 FPS)
while True:
    # Fake pose-like data
    t = int(time.time() * 1000)  # Time-based
    lean_x = random.uniform(-1, 1)  # Simulated lean value
    jump = (t % 300 == 0)  # Simulated jump condition

    payload = {
        "t": t,
        "lean_x": lean_x,
        "jump": jump
    }

    # Print payload as JSON
    print(json.dumps(payload))


    # Sleep to maintain 30 FPS
    time.sleep(1 / 30) 