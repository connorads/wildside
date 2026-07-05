#!/usr/bin/env python3
"""Dev server for the web build: http.server plus the COOP/COEP headers that
make the page cross-origin isolated, which SharedArrayBuffer (the audio
worklet ring, #20) requires. Loopback only — expose via `tsp up` if wanted,
and never funnel publicly (the bundle ships derived assets, #16)."""

import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


class IsolatedHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        # Dev server: always serve the freshly built wasm/JS, never a cache.
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8321
    directory = str(Path(__file__).resolve().parent)

    class Handler(IsolatedHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=directory, **kwargs)

    with ThreadingHTTPServer(("127.0.0.1", port), Handler) as httpd:
        print(f"serving http://127.0.0.1:{port}/ (COOP/COEP: cross-origin isolated)")
        httpd.serve_forever()


if __name__ == "__main__":
    main()
