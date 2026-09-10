#!/usr/bin/env python3
import urllib.request
import urllib.error
import sys
import os

def check_health(url):
    try:
        req = urllib.request.Request(f"{url}/healthz")
        with urllib.request.urlopen(req, timeout=5) as response:
            if response.status == 200:
                data = response.read().decode('utf-8')
                print(f"Health check passed for {url}: {data}")
                return True
            else:
                print(f"Health check failed for {url} with status {response.status}")
                return False
    except urllib.error.URLError as e:
        print(f"Connection failed for {url}: {e.reason}")
        return False
    except Exception as e:
        print(f"Unexpected error for {url}: {e}")
        return False

def check_ready(url):
    try:
        req = urllib.request.Request(f"{url}/readyz")
        with urllib.request.urlopen(req, timeout=5) as response:
            if response.status == 200:
                data = response.read().decode('utf-8')
                print(f"Ready check passed for {url}: {data}")
                return True
            else:
                print(f"Ready check failed for {url} with status {response.status}")
                return False
    except urllib.error.URLError as e:
        print(f"Connection failed for {url}: {e.reason}")
        return False
    except Exception as e:
        print(f"Unexpected error for {url}: {e}")
        return False

if __name__ == "__main__":
    url = os.environ.get("OPENSENTINEL_URL", "http://localhost:8080").rstrip('/')
    print(f"Checking OpenSentinel health at {url}...")

    health_ok = check_health(url)
    ready_ok = check_ready(url)

    if health_ok and ready_ok:
        print("All checks passed successfully.")
        sys.exit(0)
    else:
        print("Some checks failed.")
        sys.exit(1)
