import unittest
from unittest.mock import MagicMock
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__))))

from opensentinel.client import OpenSentinelClient
from opensentinel.middleware import OpenSentinelMiddleware


class TestPythonSDK(unittest.TestCase):
    def test_client_endpoint_normalization(self):
        client1 = OpenSentinelClient("http://localhost:8080/verify")
        self.assertEqual(client1.endpoint, "http://localhost:8080")
        self.assertEqual(client1.verify_url, "http://localhost:8080/verify")

        client2 = OpenSentinelClient("http://localhost:8080/verify/")
        self.assertEqual(client2.endpoint, "http://localhost:8080")

        client3 = OpenSentinelClient("http://localhost:8080")
        self.assertEqual(client3.endpoint, "http://localhost:8080")

    def test_client_siteverify_args(self):
        client = OpenSentinelClient("http://localhost:8080")

        # Test single positional argument (token) without error when connection fails
        res = client.siteverify("my-token")
        self.assertIn("success", res)

        # Test keyword arguments
        res2 = client.siteverify(secret="my-secret", response_token="my-token", remoteip="127.0.0.1")
        self.assertIn("success", res2)

    def test_middleware_init_and_secret(self):
        mock_client = MagicMock()
        mock_client.siteverify.return_value = {"success": True}

        middleware = OpenSentinelMiddleware(mock_client, secret="secret_123")
        self.assertEqual(middleware.secret, "secret_123")


if __name__ == "__main__":
    unittest.main()
