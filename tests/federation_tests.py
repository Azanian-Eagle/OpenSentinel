import subprocess
import time
import urllib.request
import urllib.error
import json
import os
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading
import binascii
import hashlib
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

SERVER_PORT = 8081
MOCK_PEER_PORT = 8082
SERVER_URL = f"http://localhost:{SERVER_PORT}"
SERVER_CMD = ["cargo", "run"]

# Generated a random Ed25519 keypair for testing (Node)
signing_key = ed25519.Ed25519PrivateKey.generate()
verifying_key = signing_key.public_key()
NODE_PRIVATE_KEY_HEX = signing_key.private_bytes(
    encoding=serialization.Encoding.Raw,
    format=serialization.PrivateFormat.Raw,
    encryption_algorithm=serialization.NoEncryption()
).hex()
NODE_PUBLIC_KEY_HEX = verifying_key.public_bytes(
    encoding=serialization.Encoding.Raw,
    format=serialization.PublicFormat.Raw
).hex()

# Another keypair for the "peer"
peer_signing_key = ed25519.Ed25519PrivateKey.generate()
peer_verifying_key = peer_signing_key.public_key()
PEER_PRIVATE_KEY_HEX = peer_signing_key.private_bytes(
    encoding=serialization.Encoding.Raw,
    format=serialization.PrivateFormat.Raw,
    encryption_algorithm=serialization.NoEncryption()
).hex()
PEER_PUBLIC_KEY_HEX = peer_verifying_key.public_bytes(
    encoding=serialization.Encoding.Raw,
    format=serialization.PublicFormat.Raw
).hex()

received_gossip_payloads = []

class MockPeerHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        post_data = self.rfile.read(content_length)

        if self.path == '/api/federation/intel':
            data = json.loads(post_data.decode('utf-8'))
            received_gossip_payloads.append(data)
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b"OK")
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass # Suppress logging

def start_mock_peer():
    server = HTTPServer(('localhost', MOCK_PEER_PORT), MockPeerHandler)
    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    server_thread.start()
    return server

def start_server():
    print("Starting OpenSentinel server with Federation enabled...")

    env = os.environ.copy()
    env["PORT"] = str(SERVER_PORT)
    env["FEDERATION_ENABLED"] = "true"
    env["TRUSTED_PEERS"] = f"http://localhost:{MOCK_PEER_PORT}"
    env["TRUSTED_PEERS_PUBKEYS"] = PEER_PUBLIC_KEY_HEX
    env["NODE_PRIVATE_KEY"] = NODE_PRIVATE_KEY_HEX
    env["NODE_ID"] = "test_node_main"

    process = subprocess.Popen(
        SERVER_CMD,
        cwd="./server",
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    for _ in range(60):
        try:
            with urllib.request.urlopen(f"{SERVER_URL}/src/sensor.js") as response:
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

def make_request(path, data, method="POST"):
    req = urllib.request.Request(f"{SERVER_URL}{path}", data=json.dumps(data).encode('utf-8'), method=method)
    req.add_header('Content-Type', 'application/json')
    try:
        with urllib.request.urlopen(req) as response:
            return response.status, json.loads(response.read().decode('utf-8'))
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read().decode('utf-8'))

def test_receive_valid_intel():
    print("\nTesting: Receive valid threat intel...")
    if os.path.exists("server/threat_intel.log"):
        os.remove("server/threat_intel.log")

    timestamp = int(time.time() * 1000)
    anonymized_signature = "some_signature_hash_123"
    score = 0.1
    source_node = f"http://localhost:{MOCK_PEER_PORT}"

    payload_str = f"{anonymized_signature}_{score}_{timestamp}_{source_node}"
    signature = peer_signing_key.sign(payload_str.encode('utf-8'))
    signature_hex = binascii.hexlify(signature).decode('utf-8')

    payload = {
        "anonymized_signature": anonymized_signature,
        "score": score,
        "timestamp": timestamp,
        "source_node": source_node,
        "signature": signature_hex
    }

    status, response = make_request("/api/federation/intel", payload)

    if status != 200:
        raise Exception(f"Expected 200 OK, got {status}. Response: {response}")

    print("Valid intel accepted.")

    time.sleep(1)
    if not os.path.exists("server/threat_intel.log"):
        raise Exception("threat_intel.log was not created")

    with open("server/threat_intel.log", "r") as f:
        content = f.read()
        if anonymized_signature not in content:
            raise Exception("Log does not contain the signature")

    print("Log verification passed.")

def test_receive_invalid_signature():
    print("\nTesting: Reject invalid signature...")

    timestamp = int(time.time() * 1000)
    anonymized_signature = "some_signature_hash_456"
    score = 0.2
    source_node = f"http://localhost:{MOCK_PEER_PORT}"

    payload_str = f"{anonymized_signature}_{0.3}_{timestamp}_{source_node}"
    signature = peer_signing_key.sign(payload_str.encode('utf-8'))
    signature_hex = binascii.hexlify(signature).decode('utf-8')

    payload = {
        "anonymized_signature": anonymized_signature,
        "score": score,
        "timestamp": timestamp,
        "source_node": source_node,
        "signature": signature_hex
    }

    status, response = make_request("/api/federation/intel", payload)

    if status != 403:
        raise Exception(f"Expected 403 Forbidden, got {status}. Response: {response}")

    print("Invalid signature rejected.")

def test_gossip_broadcast():
    print("\nTesting: Gossip broadcast on bot detection...")

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
        "mouse_events": [[10, 10, timestamp], [20, 20, timestamp+10], [30, 30, timestamp+20], [40, 40, timestamp+30], [50, 50, timestamp+40]],
        "key_events": [["KeyA", timestamp], ["KeyB", timestamp+10]],
        "user_agent": "test-bot",
        "timestamp": timestamp,
        "pow": {
            "prefix": prefix,
            "nonce": nonce,
            "hash": h
        }
    }

    import base64

    key = bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32])
    aesgcm = AESGCM(key)
    iv = os.urandom(12)

    pt = json.dumps(raw_payload).encode('utf-8')
    ct = aesgcm.encrypt(iv, pt, None)

    req_payload = {
        "data": base64.b64encode(ct).decode('utf-8'),
        "iv": base64.b64encode(iv).decode('utf-8')
    }

    status, response = make_request("/verify", req_payload)
    if status != 200 or response.get("passed") != False:
        raise Exception(f"Expected bot detection, got status: {status}, response: {response}")

    print("Bot detected locally. Waiting for gossip broadcast...")
    time.sleep(2)

    if len(received_gossip_payloads) == 0:
        raise Exception("No gossip payload received by mock peer")

    intel = received_gossip_payloads[0]
    if intel.get("source_node") != "test_node_main":
        raise Exception(f"Unexpected source_node: {intel.get('source_node')}")

    payload_str = f"{intel['anonymized_signature']}_{intel['score']}_{intel['timestamp']}_{intel['source_node']}"

    try:
        sig_bytes = binascii.unhexlify(intel['signature'])
        verifying_key.verify(sig_bytes, payload_str.encode('utf-8'))
        print("Gossip broadcast signature verification passed.")
    except Exception as e:
        raise Exception(f"Failed to verify gossip signature: {e}")

    print("Gossip broadcast test passed.")

def main():
    mock_server = start_mock_peer()
    server_process = start_server()
    if not server_process:
        sys.exit(1)

    try:
        test_receive_valid_intel()
        test_receive_invalid_signature()
        test_gossip_broadcast()
        print("\nAll Federation integration tests PASSED!")
    except Exception as e:
        print(f"\nTest FAILED: {e}")
        server_process.terminate()
        stdout, stderr = server_process.communicate()
        print("SERVER OUTPUT:")
        print(stdout)
        print(stderr)
        sys.exit(1)
    finally:
        server_process.terminate()
        server_process.wait()
        mock_server.shutdown()

if __name__ == "__main__":
    main()
