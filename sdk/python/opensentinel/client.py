import json
import urllib.request
import urllib.error

class OpenSentinelClient:
    def __init__(self, endpoint: str):
        """
        Initialize the OpenSentinel client.

        :param endpoint: The URL of the OpenSentinel server (e.g. 'http://localhost:8080')
        """
        self.endpoint = endpoint.rstrip('/')
        self.verify_url = f"{self.endpoint}/verify"

    def verify(self, payload: dict) -> dict:
        """
        Verify an encrypted payload using the OpenSentinel backend.

        :param payload: The payload dictionary containing 'data' and 'iv'
        :return: A dictionary containing the verification result ('passed', 'score', etc.)
        """
        if not isinstance(payload, dict):
            raise ValueError("Payload must be a dictionary")

        data = json.dumps(payload).encode('utf-8')
        req = urllib.request.Request(
            self.verify_url,
            data=data,
            headers={'Content-Type': 'application/json'}
        )

        try:
            with urllib.request.urlopen(req) as response:
                result_data = response.read()
                return json.loads(result_data.decode('utf-8'))
        except urllib.error.HTTPError as e:
            try:
                error_data = e.read()
                return json.loads(error_data.decode('utf-8'))
            except json.JSONDecodeError:
                raise Exception(f"HTTPError {e.code}: {e.reason}")
        except Exception as e:
            raise Exception(f"Failed to verify payload: {str(e)}")
