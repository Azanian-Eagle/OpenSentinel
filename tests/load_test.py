import urllib.request
import urllib.error
import threading
import time
import json
import random
import os
import subprocess
import sys

SERVER_URL = "http://localhost:8080"
SERVER_CMD = ["cargo", "run"]
DEFAULT_ENV = {
    "PAYLOAD_SECRET_KEY": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
    "OPEN_SENTINEL_MODEL_PATH": "model.onnx",
}
NUM_THREADS = 20
REQUESTS_PER_THREAD = 50

success_count = 0
failure_count = 0
total_time = 0.0
lock = threading.Lock()

def make_health_request():
    global success_count, failure_count, total_time
    for _ in range(REQUESTS_PER_THREAD):
        start_time = time.time()
        try:
            with urllib.request.urlopen(f"{SERVER_URL}/healthz") as response:
                if response.status == 200:
                    with lock:
                        success_count += 1
                        total_time += (time.time() - start_time)
                else:
                    with lock:
                        failure_count += 1
        except Exception:
            with lock:
                failure_count += 1

def start_server():
    print("Starting server for load testing...")
    process = subprocess.Popen(
        SERVER_CMD,
        cwd="./server",
        env={**os.environ, **DEFAULT_ENV},
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    for _ in range(60):
        try:
            with urllib.request.urlopen(SERVER_URL) as response:
                if response.status == 200:
                    print("Server started!")
                    return process
        except (urllib.error.URLError, ConnectionResetError):
            time.sleep(1)

    print("Failed to start server.")
    process.terminate()
    return None

def main():
    print("Building server...")
    subprocess.run(["cargo", "build"], cwd="./server", check=True)

    server_process = start_server()
    if not server_process:
        sys.exit(1)

    try:
        print(f"Running load test with {NUM_THREADS} threads, {REQUESTS_PER_THREAD} requests each...")
        threads = []
        start_time = time.time()

        for _ in range(NUM_THREADS):
            t = threading.Thread(target=make_health_request)
            threads.append(t)
            t.start()

        for t in threads:
            t.join()

        end_time = time.time()

        print(f"\n--- Load Test Results ---")
        print(f"Total Requests: {NUM_THREADS * REQUESTS_PER_THREAD}")
        print(f"Successful Requests: {success_count}")
        print(f"Failed Requests: {failure_count}")
        print(f"Total Time Taken: {end_time - start_time:.2f} seconds")
        if success_count > 0:
            print(f"Average Request Time: {(total_time / success_count) * 1000:.2f} ms")

        if failure_count > 0:
            print("Load test failed due to failed requests.")
            sys.exit(1)
        else:
            print("Load test PASSED!")

    finally:
        server_process.terminate()
        server_process.wait()

if __name__ == "__main__":
    main()
