import subprocess
import time
import urllib.request
import urllib.error
import random
import math
import sys
from playwright.sync_api import sync_playwright

SERVER_URL = "http://localhost:8080"
SERVER_CMD = ["cargo", "run"]

def start_server():
    print("Starting server...")
    # Run from server directory
    process = subprocess.Popen(
        SERVER_CMD,
        cwd="./server",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    # Wait for server to start
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
    stdout, stderr = process.communicate()
    print("STDOUT:", stdout)
    print("STDERR:", stderr)
    return None

def test_bot_behavior(page):
    print("\nRunning Bot Behavior Test...")
    page.goto(SERVER_URL)
    page.wait_for_selector("#verifyBtn")

    # Simulate Bot: Perfectly linear mouse movement
    print("Simulating bot mouse movement...")
    start_x, start_y = 100, 100
    end_x, end_y = 500, 500
    steps = 20

    for i in range(steps + 1):
        x = start_x + (end_x - start_x) * (i / steps)
        y = start_y + (end_y - start_y) * (i / steps)
        page.mouse.move(x, y)
        time.sleep(0.01) # constant speed

    # Simulate Bot: Instant typing
    print("Simulating bot typing...")
    # page.fill might be too perfect/instant and not trigger keydown
    # Let's try type with 0 delay
    page.focus("#testInput")
    page.keyboard.type("I am a robot", delay=0)

    # Click verify
    page.click("#verifyBtn")

    # Wait for result
    # We expect failure
    try:
        page.wait_for_selector(".failure.visible", timeout=5000)
    except:
        print("Timeout waiting for failure message. Checking content...")

    result_text = page.text_content("#result")
    print(f"Result: {result_text}")

    if "Verification failed" not in result_text:
        raise Exception(f"Expected verification failure, got: {result_text}")
    print("Bot test PASSED (Verification correctly failed)")

def test_human_behavior(page):
    print("\nRunning Human Behavior Test...")
    page.goto(SERVER_URL)
    page.wait_for_selector("#verifyBtn")

    # Simulate Human: Noisy mouse movement
    print("Simulating human mouse movement...")
    start_x, start_y = 100, 100
    end_x, end_y = 500, 500
    steps = 50

    for i in range(steps + 1):
        # Add noise
        noise_x = random.randint(-5, 5)
        noise_y = random.randint(-5, 5)

        t = i / steps
        # Simple curve
        x = start_x + (end_x - start_x) * t + noise_x + math.sin(t * math.pi) * 50
        y = start_y + (end_y - start_y) * t + noise_y

        page.mouse.move(x, y)
        time.sleep(random.uniform(0.01, 0.05)) # variable speed

    # Simulate Human: Typing with delays
    print("Simulating human typing...")
    page.focus("#testInput")
    text = "I am a human"
    for char in text:
        page.keyboard.type(char)
        time.sleep(random.uniform(0.05, 0.15))

    # Click verify
    page.click("#verifyBtn")

    # Wait for result
    try:
        page.wait_for_selector(".success.visible", timeout=5000)
    except:
        print("Timeout waiting for success message. Checking content...")

    result_text = page.text_content("#result")
    print(f"Result: {result_text}")

    if "Verification successful" not in result_text:
        raise Exception(f"Expected verification success, got: {result_text}")
    print("Human test PASSED (Verification correctly passed)")

def main():
    # Build first to avoid timeout
    print("Building server...")
    subprocess.run(["cargo", "build"], cwd="./server", check=True)

    server_process = start_server()
    if not server_process:
        sys.exit(1)

    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            page = browser.new_page()

            try:
                test_bot_behavior(page)
                test_human_behavior(page)
                print("\nAll integration tests PASSED!")
            except Exception as e:
                print(f"\nTest FAILED: {e}")
                # Kill server to read logs
                server_process.terminate()
                stdout, stderr = server_process.communicate()
                print("SERVER OUTPUT:")
                print(stdout)
                print(stderr)
                sys.exit(1)
            finally:
                browser.close()
    finally:
        if server_process.poll() is None:
            print("Stopping server...")
            server_process.terminate()
            server_process.wait()

if __name__ == "__main__":
    main()
