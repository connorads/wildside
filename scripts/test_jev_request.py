"""Exercise the private Jev adapter against a local HTTP service."""

import json
import threading
import unittest
from http.server import BaseHTTPRequestHandler, HTTPServer

import jev_request


class JevRequestTests(unittest.TestCase):
    def test_posts_typed_choice_and_returns_usage(self):
        received = []

        class Handler(BaseHTTPRequestHandler):
            def do_POST(self):
                received.append((self.headers.get("Authorization"), json.loads(
                    self.rfile.read(int(self.headers["Content-Length"])))))
                body = json.dumps({"model": "jev-1.13.0", "answers": {
                    "action": {"type": "choice", "choice": "jump"}},
                    "usage": {"input_tokens": 100}}).encode()
                self.send_response(200)
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *_):
                pass

        server = HTTPServer(("127.0.0.1", 0), Handler)
        worker = threading.Thread(target=server.handle_request)
        worker.start()
        try:
            response = jev_request.evaluate({"target_hit": False}, "test-only-key",
                endpoint=f"http://127.0.0.1:{server.server_port}/v1/systemone")
        finally:
            worker.join(timeout=5)
            server.server_close()
        self.assertEqual(response["answers"]["action"]["choice"], "jump")
        self.assertEqual(response["usage"]["input_tokens"], 100)
        auth, request = received[0]
        self.assertEqual(auth, "Bearer test-only-key")
        self.assertEqual(request["model"], "jev-1.13.0")
        self.assertEqual(request["state"], {"target_hit": False})
        self.assertEqual(request["questions"]["action"]["type"], "choice")
        self.assertIn("release", request["questions"]["action"]["criteria"])

    def test_missing_key_stops_before_network(self):
        with self.assertRaises(ValueError):
            jev_request.evaluate({}, "")


if __name__ == "__main__":
    unittest.main()
