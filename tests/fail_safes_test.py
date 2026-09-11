import urllib.request
import urllib.error
import json
import base64
import os
import time
import hashlib
import sys
import subprocess
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

SERVER_URL = "http://localhost:8080"
SERVER_CMD = ["cargo", "run"]
DEFAULT_ENV = {
    "PAYLOAD_SECRET_KEY": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
    "OPEN_SENTINEL_MODEL_PATH": "model.onnx",
}
key_bytes = bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32])

def start_server():
    print("Starting server...")
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

def make_request(payload, raw=False):
    if raw:
        req_data = payload
    else:
        req_data = json.dumps(payload).encode('utf-8')
    req = urllib.request.Request(f"{SERVER_URL}/verify", data=req_data, method="POST")
    req.add_header('Content-Type', 'application/json')
    try:
        with urllib.request.urlopen(req) as response:
            return response.status, json.loads(response.read().decode('utf-8'))
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read().decode('utf-8'))

def test_missing_pow():
    print("Testing missing PoW...")
    timestamp = int(time.time() * 1000)
    raw_payload = {
        "mouse_events": [[10, 10, timestamp]],
        "key_events": [],
        "user_agent": "test",
        "timestamp": timestamp
        # Missing 'pow' field
    }

    aesgcm = AESGCM(key_bytes)
    iv = os.urandom(12)
    pt = json.dumps(raw_payload).encode('utf-8')
    ct = aesgcm.encrypt(iv, pt, None)

    req_payload = {
        "data": base64.b64encode(ct).decode('utf-8'),
        "iv": base64.b64encode(iv).decode('utf-8')
    }

    status, response = make_request(req_payload)
    assert status == 400
    assert "Missing Proof of Work" in response["message"]
    print("Missing PoW test PASSED")

def test_invalid_pow():
    print("Testing invalid PoW...")
    timestamp = int(time.time() * 1000)
    raw_payload = {
        "mouse_events": [[10, 10, timestamp]],
        "key_events": [],
        "user_agent": "test",
        "timestamp": timestamp,
        "pow": {
            "prefix": "invalid",
            "nonce": 123,
            "hash": "000invalidhash"
        }
    }

    aesgcm = AESGCM(key_bytes)
    iv = os.urandom(12)
    pt = json.dumps(raw_payload).encode('utf-8')
    ct = aesgcm.encrypt(iv, pt, None)

    req_payload = {
        "data": base64.b64encode(ct).decode('utf-8'),
        "iv": base64.b64encode(iv).decode('utf-8')
    }

    status, response = make_request(req_payload)
    assert status == 400
    assert "Invalid Proof of Work" in response["message"]
    print("Invalid PoW test PASSED")

def test_replay_attack():
    print("Testing replay attack...")
    timestamp = int(time.time() * 1000)
    prefix = f"{timestamp}rand"
    nonce = 0
    while True:
        msg = f"{prefix}{nonce}".encode('utf-8')
        h = hashlib.sha256(msg).hexdigest()
        if h.startswith("000"):
            break
        nonce += 1

    raw_payload = {
        "mouse_events": [[10, 10, timestamp]],
        "key_events": [],
        "user_agent": "test",
        "timestamp": timestamp,
        "pow": {
            "prefix": prefix,
            "nonce": nonce,
            "hash": h
        }
    }

    aesgcm = AESGCM(key_bytes)
    iv = os.urandom(12)
    pt = json.dumps(raw_payload).encode('utf-8')
    ct = aesgcm.encrypt(iv, pt, None)

    req_payload = {
        "data": base64.b64encode(ct).decode('utf-8'),
        "iv": base64.b64encode(iv).decode('utf-8')
    }

    status, response = make_request(req_payload)
    assert status == 200 # First time succeeds

    status, response = make_request(req_payload)
    assert status == 400
    assert "Replay attack detected" in response["message"]
    print("Replay attack test PASSED")

def test_invalid_decryption():
    print("Testing invalid decryption...")
    timestamp = int(time.time() * 1000)
    raw_payload = {"some": "data"}

    wrong_key = bytes([2] * 32)
    aesgcm = AESGCM(wrong_key)
    iv = os.urandom(12)
    pt = json.dumps(raw_payload).encode('utf-8')
    ct = aesgcm.encrypt(iv, pt, None)

    req_payload = {
        "data": base64.b64encode(ct).decode('utf-8'),
        "iv": base64.b64encode(iv).decode('utf-8')
    }

    status, response = make_request(req_payload)
    assert status == 400
    assert "Failed to decrypt payload" in response["message"]
    print("Invalid decryption test PASSED")

def test_invalid_iv_length():
    print("Testing invalid IV length...")
    req_payload = {
        "data": base64.b64encode(b"somedata").decode('utf-8'),
        "iv": base64.b64encode(b"shortiv").decode('utf-8')
    }
    status, response = make_request(req_payload)
    assert status == 400
    assert "Invalid IV length" in response["message"]
    print("Invalid IV length test PASSED")

def main():
    subprocess.run(["cargo", "build"], cwd="./server", check=True)
    server_process = start_server()
    if not server_process:
        sys.exit(1)

    try:
        test_missing_pow()
        test_invalid_pow()
        test_replay_attack()
        test_invalid_decryption()
        test_invalid_iv_length()
        print("\nAll fail safes tests PASSED!")
    except Exception as e:
        print(f"\nTest FAILED: {e}")
        sys.exit(1)
    finally:
        server_process.terminate()
        server_process.wait()

if __name__ == "__main__":
    main()
