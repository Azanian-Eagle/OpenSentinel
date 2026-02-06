import sys
import threading
import time
from http.server import HTTPServer, SimpleHTTPRequestHandler
from playwright.sync_api import sync_playwright
import os

# Configuration
PORT = 8000
DIST_DIR = "dist"

def print_structure():
    print(f"--- Structure of {DIST_DIR} ---")
    if not os.path.exists(DIST_DIR):
        print("dist directory does not exist!")
        return

    for root, dirs, files in os.walk(DIST_DIR):
        level = root.replace(DIST_DIR, '').count(os.sep)
        indent = ' ' * 4 * (level)
        print(f"{indent}{os.path.basename(root)}/")
        subindent = ' ' * 4 * (level + 1)
        # Limit file listing to avoid spam
        for f in files[:5]:
            print(f"{subindent}{f}")
        if len(files) > 5:
            print(f"{subindent}... ({len(files)-5} more)")

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

        # Context 1: Desktop
        print("--- Verifying Desktop ---")
        page = browser.new_page()

        # Listen for console errors
        console_errors = []
        page.on("console", lambda msg: console_errors.append(f"Console {msg.type}: {msg.text}") if msg.type == "error" else None)
        page.on("pageerror", lambda exc: console_errors.append(f"Page Error: {exc}"))

        # Listen for network failures (404s, failed loads)
        network_errors = []
        page.on("requestfailed", lambda req: network_errors.append(f"Request failed: {req.url} - {req.failure}"))
        page.on("response", lambda res: network_errors.append(f"HTTP Error: {res.url} - {res.status}") if res.status >= 400 else None)

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
            if page.is_visible("nav.navbar"):
                 print("Navbar found.")
            else:
                 print("ERROR: Navbar not visible.")
                 sys.exit(1)

            if page.is_visible("footer"):
                 print("Footer found.")
            else:
                 print("ERROR: Footer not visible.")
                 sys.exit(1)
        except Exception as e:
            print(f"ERROR: Element verification failed: {e}")
            sys.exit(1)

        # Check for errors (Desktop)
        if console_errors or network_errors:
            print("ERROR: Errors detected on Desktop:")
            for err in console_errors: print(f"  - {err}")
            for err in network_errors: print(f"  - {err}")
            sys.exit(1)
        else:
            print("No errors on Desktop.")

        page.close()

        # Context 2: Mobile
        print("--- Verifying Mobile ---")
        context_mobile = browser.new_context(viewport={"width": 375, "height": 812}, user_agent="Mozilla/5.0 (iPhone; CPU iPhone OS 12_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/12.0 Mobile/15E148 Safari/604.1")
        page_mobile = context_mobile.new_page()

        console_errors_mobile = []
        network_errors_mobile = []
        page_mobile.on("console", lambda msg: console_errors_mobile.append(f"Console {msg.type}: {msg.text}") if msg.type == "error" else None)
        page_mobile.on("pageerror", lambda exc: console_errors_mobile.append(f"Page Error: {exc}"))
        page_mobile.on("requestfailed", lambda req: network_errors_mobile.append(f"Request failed: {req.url} - {req.failure}"))
        page_mobile.on("response", lambda res: network_errors_mobile.append(f"HTTP Error: {res.url} - {res.status}") if res.status >= 400 else None)

        print(f"Navigating to {url} (Mobile)")
        page_mobile.goto(url)

        # Check Hamburger existence and visibility
        if page_mobile.is_visible(".hamburger"):
             print("Hamburger menu found on Mobile.")
        else:
             print("ERROR: Hamburger menu not visible on Mobile.")
             sys.exit(1)

        # Test Interaction
        print("Clicking hamburger menu...")
        page_mobile.click(".hamburger")
        # Wait for animation/class toggle
        page_mobile.wait_for_timeout(500)

        # Check if nav-links are active
        if "active" in page_mobile.get_attribute(".nav-links", "class"):
            print("Mobile menu opened successfully.")
        else:
             print("ERROR: Mobile menu did not open.")
             sys.exit(1)

        if console_errors_mobile or network_errors_mobile:
            print("ERROR: Errors detected on Mobile:")
            for err in console_errors_mobile: print(f"  - {err}")
            for err in network_errors_mobile: print(f"  - {err}")
            sys.exit(1)
        else:
            print("No errors on Mobile.")

        page_mobile.close()

        # Verify Demo Page
        print("--- Verifying Demo Page ---")
        page_demo = browser.new_page()
        demo_url = f"http://localhost:{PORT}/docs/demo.html"
        print(f"Navigating to {demo_url}")

        # Listen for errors on Demo page
        demo_errors = []
        page_demo.on("console", lambda msg: demo_errors.append(f"Console {msg.type}: {msg.text}") if msg.type == "error" else None)
        page_demo.on("requestfailed", lambda req: demo_errors.append(f"Request failed: {req.url} - {req.failure}"))
        page_demo.on("response", lambda res: demo_errors.append(f"HTTP Error: {res.url} - {res.status}") if res.status >= 400 else None)

        page_demo.goto(demo_url)

        if page_demo.title() != "OpenSentinel Live Demo | Azanian Eagle":
             print(f"ERROR: Demo page title mismatch. Got '{page_demo.title()}'")
             sys.exit(1)
        print("Demo page loaded.")

        if demo_errors:
             # Filter out 404 for favicon if it's missing (optional, but good to know)
             filtered_errors = [e for e in demo_errors if "favicon.ico" not in e]
             if filtered_errors:
                 print("ERROR: Errors detected on Demo Page:")
                 for err in filtered_errors: print(f"  - {err}")
                 sys.exit(1)

        page_demo.close()

        browser.close()
        print("Verification successful!")

if __name__ == "__main__":
    if not os.path.exists(DIST_DIR):
        print(f"ERROR: {DIST_DIR} directory not found. Run ./scripts/prepare_deploy.sh first.")
        sys.exit(1)

    print_structure()

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
