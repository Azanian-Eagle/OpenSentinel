# OpenSentinel Python SDK

Official Python SDK for **OpenSentinel**, engineered by **Azanian Eagle** to provide server-side validation and client-telemetry verification for non-invasive, privacy-preserving CAPTCHA-free bot detection.

## About OpenSentinel & Azanian Eagle

Engineered by [Azanian Eagle](https://github.com/Azanian-Eagle), OpenSentinel is built on the core principle of **digital sovereignty** and **user privacy**. Legacy CAPTCHA tools require users to decode image puzzles, exploiting human attention and harvesting personal tracking data to train third-party artificial intelligence models.

OpenSentinel replaces intrusive visual puzzles with frictionless telemetry analysis. By evaluating abstract human interaction parameters—such as mouse trajectory curvature, speed variance, and keystroke flight dynamics—OpenSentinel detects automated software bots without collecting personally identifiable information (PII).

### Key Features
- **Zero Visual Friction:** Eliminates image selection puzzles and visual challenges for end-users.
- **Lightweight Standard Library Dependencies:** Built using Python standard libraries (`urllib`, `json`), requiring zero external pip dependencies.
- **Privacy-First Architecture:** Zero tracking cookies, zero fingerprinting, and full compliance with South Africa's **Protection of Personal Information Act (POPIA)**, GDPR, and CCPA.
- **Server-Side Payload Validation:** Securely transmits client telemetry payloads to your sovereign OpenSentinel server for verification.
- **Framework Agnostic:** Easily integrates into Django, FastAPI, Flask, Pyramid, and Tornado backend applications.

---

## Installation

Install the official package from PyPI using `pip`:

```bash
pip install azanian-eagle-opensentinel
```

Or using `poetry` or `pipenv`:

```bash
poetry add azanian-eagle-opensentinel
# or
pipenv install azanian-eagle-opensentinel
```

---

## Implementation & Usage Guide

### 1. Basic Client Usage

Initialize `OpenSentinelClient` with your self-hosted or managed OpenSentinel endpoint, then verify the payload received from your frontend client:

```python
from opensentinel import OpenSentinelClient

# Initialize client with OpenSentinel server endpoint URL
client = OpenSentinelClient(endpoint="https://api.yourdomain.com")

# Payload received from frontend sensor (contains encrypted telemetry and IV)
incoming_payload = {
    "data": "A1b2C3d4E5f6...",  # AES-256-GCM ciphertext
    "iv": "X1y2Z3a4b5c6=="     # Base64 IV
}

try:
    # Submit payload for verification
    result = client.verify(incoming_payload)

    if result.get("passed"):
        print(f"Humanity verified! Score: {result.get('score')}")
        # Process user request (e.g., allow login, comment submission)
    else:
        print(f"Bot detected or verification failed: {result.get('message')}")
        # Block request or trigger secondary security challenge

except Exception as e:
    print(f"Verification error: {str(e)}")
```

---

## Web Framework Integration Examples

### FastAPI Integration

```python
from fastapi import FastAPI, HTTPException, Request
from opensentinel import OpenSentinelClient

app = FastAPI()
sentinel = OpenSentinelClient(endpoint="http://localhost:8080")

@app.post("/api/login")
async def login(request: Request):
    data = await request.json()
    payload = data.get("sentinel_payload")

    if not payload:
        raise HTTPException(status_code=400, detail="Missing OpenSentinel telemetry payload")

    try:
        verification = sentinel.verify(payload)
        if not verification.get("passed"):
            raise HTTPException(status_code=403, detail="Security check failed: Bot activity detected")
    except Exception as err:
        raise HTTPException(status_code=500, detail=f"Sentinel service error: {str(err)}")

    return {"status": "authenticated"}
```

### Flask Integration

```python
from flask import Flask, request, jsonify
from opensentinel import OpenSentinelClient

app = Flask(__name__)
sentinel = OpenSentinelClient(endpoint="http://localhost:8080")

@app.route("/submit-form", methods=["POST"])
def submit_form():
    payload = request.json.get("opensentinel")

    if not payload:
        return jsonify({"error": "Missing verification payload"}), 400

    result = sentinel.verify(payload)
    if not result.get("passed"):
        return jsonify({"error": "Human verification failed"}), 403

    return jsonify({"success": True, "message": "Form submitted successfully"})
```

---

## Class Reference & Methods

### `OpenSentinelClient(endpoint: str)`

- **`endpoint`** (`str`): Base URL of your OpenSentinel server instance (e.g. `'https://api.yourdomain.com'`).

#### Method: `verify(payload: dict) -> dict`

Submits client telemetry payload dictionary (`{'data': str, 'iv': str}`) to the backend `/verify` endpoint.

- **Parameters:**
  - `payload` (`dict`): Payload dictionary containing `'data'` (ciphertext base64) and `'iv'` (initialization vector base64).
- **Returns:**
  - `dict`: Response dictionary containing:
    - `passed` (`bool`): `True` if verification passed humanity threshold, `False` otherwise.
    - `score` (`float`): Calculated probability score (0.0 to 1.0).
    - `message` (`str`): Verification status description or error details.

---

## Regulatory Compliance & Privacy

OpenSentinel is engineered by **Azanian Eagle** to respect digital sovereignty and global privacy standards:
- **POPIA (South Africa):** Fully compliant as no personal identifiable information (PII) is captured or stored.
- **GDPR & CCPA:** Compliant due to complete absence of tracking cookies, device fingerprinting, and third-party data selling.

---

## Licence

Distributed under the **MIT Licence**. Engineered with pride by **Azanian Eagle**.
