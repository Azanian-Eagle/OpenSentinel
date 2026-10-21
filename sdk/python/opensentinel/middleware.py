import functools

class OpenSentinelMiddleware:
    """
    Framework middleware helpers for OpenSentinel token verification.
    """
    def __init__(self, client, secret: str = ""):
        self.client = client
        self.secret = secret

    def flask_verify(self, param_name="opensentinel_token"):
        """
        Flask view decorator for verifying OpenSentinel tokens.
        """
        def decorator(f):
            @functools.wraps(f)
            def wrapper(*args, **kwargs):
                try:
                    from flask import request, jsonify
                except ImportError:
                    raise RuntimeError("Flask package is required for flask_verify decorator")

                token = request.headers.get("X-OpenSentinel-Token") or request.values.get(param_name)
                if not token and request.is_json:
                    token = (request.get_json() or {}).get(param_name)

                if not token:
                    return jsonify({"error": "Missing verification token"}), 400

                remoteip = request.remote_addr
                res = self.client.siteverify(secret=self.secret, response_token=token, remoteip=remoteip)
                if not res.get("success"):
                    return jsonify({"error": "Verification failed"}), 403

                return f(*args, **kwargs)
            return wrapper
        return decorator

    def fastapi_dependency(self, param_name="opensentinel_token"):
        """
        FastAPI / Starlette dependency for verifying OpenSentinel tokens.
        """
        async def dependency(request):
            token = request.headers.get("X-OpenSentinel-Token") or request.query_params.get(param_name)
            if not token:
                from fastapi import HTTPException
                raise HTTPException(status_code=400, detail="Missing verification token")

            remoteip = getattr(getattr(request, "client", None), "host", None)
            res = self.client.siteverify(secret=self.secret, response_token=token, remoteip=remoteip)
            if not res.get("success"):
                from fastapi import HTTPException
                raise HTTPException(status_code=403, detail="Verification failed")

            return res

        return dependency
