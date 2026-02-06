import sys
import threading
import time
from http.server import HTTPServer, SimpleHTTPRequestHandler
from playwright.sync_api import sync_playwright
import os

# Configuration
PORT = 8000
DIST_DIR = "dist"

def start_server():
    try:
        os.chdir(DIST_DIR)
        httpd = HTTPServer(("", PORT), SimpleHTTPRequestHandler)
        print(f"Serving {DIST_DIR} at http://localhost:{PORT}")
        httpd.serve_forever()
    except Exception as e:
        print(f"Server error: {e}")

def verify():
    print("Starting verification...")
    with sync_playwright() as p:
        browser = p.chromium.launch()
        page = browser.new_page()

        # Navigate to /docs/
        url = f"http://localhost:{PORT}/docs/"
        print(f"Navigating to {url}")
        response = page.goto(url)

        if response.status != 200:
            print(f"ERROR: Failed to load page. Status: {response.status}")
            sys.exit(1)

        # Verify Title
        expected_title = "OpenSentinel | Privacy-First Bot Protection by Azanian Eagle"
        title = page.title()
        if title != expected_title:
            print(f"ERROR: Title mismatch. Expected '{expected_title}', got '{title}'")
            sys.exit(1)
        print("Title verification passed.")

        # Verify Key Elements
        try:
            # Check for Navbar
            if page.is_visible("nav.navbar"):
                 print("Navbar found.")
            else:
                 print("ERROR: Navbar not visible.")
                 sys.exit(1)

            # Check for Footer
            if page.is_visible("footer"):
                 print("Footer found.")
            else:
                 print("ERROR: Footer not visible.")
                 sys.exit(1)

        except Exception as e:
            print(f"ERROR: Element verification failed: {e}")
            sys.exit(1)

        # Take Screenshot (save in repo root, so ../ relative to dist)
        # Note: os.chdir in start_server is in a thread, but os.chdir affects the whole process in Python!
        # So we are currently in 'dist'.
        # We want to save to project root.
        page.screenshot(path="../verification_screenshot.png")
        print("Screenshot saved to verification_screenshot.png")

        browser.close()
        print("Verification successful!")

if __name__ == "__main__":
    if not os.path.exists(DIST_DIR):
        print(f"ERROR: {DIST_DIR} directory not found. Run ./scripts/prepare_deploy.sh first.")
        sys.exit(1)

    # Start server in background thread
    server_thread = threading.Thread(target=start_server, daemon=True)
    server_thread.start()

    # Give server a moment to start
    time.sleep(2)

    try:
        verify()
    except SystemExit as e:
        sys.exit(e.code)
    except Exception as e:
        print(f"An error occurred: {e}")
        sys.exit(1)
