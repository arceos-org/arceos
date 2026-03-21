#!/usr/bin/env python3
"""
Simple HTTP-to-HTTPS reverse proxy for ArceAgent testing.

Listens on 0.0.0.0:8080 (plain HTTP) and forwards requests to the
Tsinghua AI Platform HTTPS API. This allows ArceOS (which lacks TLS)
to reach the API via QEMU user-mode networking (10.0.2.2:8080).

The proxy also injects the Authorization header, so the API key does NOT
need to be stored in the Rust binary or transmitted over the unencrypted
QEMU virtual network link.

Usage:
    python3 llm_proxy.py
    # Then ArceAgent connects to http://10.0.2.2:8080/v1/chat/completions
"""

import http.server
import urllib.request
import ssl
import json
import sys

TARGET_BASE = "https://lab.cs.tsinghua.edu.cn/ai-platform/api/v1"
LISTEN_PORT = 8080

# API key is stored here (server-side only) instead of in the Rust binary.
API_KEY = # Paste your API key here


class ProxyHandler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        self._proxy()

    def do_GET(self):
        self._proxy()

    def _proxy(self):
        # Read request body if present
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length) if content_length > 0 else None

        # Build target URL: /v1/chat/completions -> TARGET_BASE/chat/completions
        # Strip /v1 prefix if present since TARGET_BASE already includes /v1
        path = self.path
        if path.startswith("/v1"):
            path = path[3:]  # Remove /v1 prefix
        target_url = TARGET_BASE + path

        # Forward selected headers from the client
        headers = {}
        for key in ("Content-Type", "Accept"):
            val = self.headers.get(key)
            if val:
                headers[key] = val

        # Always inject the API key (overrides any client-provided key)
        headers["Authorization"] = f"Bearer {API_KEY}"

        try:
            req = urllib.request.Request(
                target_url,
                data=body,
                headers=headers,
                method=self.command,
            )
            # Create SSL context that validates certs
            ctx = ssl.create_default_context()
            with urllib.request.urlopen(req, context=ctx, timeout=120) as resp:
                resp_body = resp.read()
                self.send_response(resp.status)
                for key, val in resp.getheaders():
                    if key.lower() not in ("transfer-encoding", "connection"):
                        self.send_header(key, val)
                self.send_header("Content-Length", str(len(resp_body)))
                self.end_headers()
                self.wfile.write(resp_body)
        except urllib.error.HTTPError as e:
            error_body = e.read()
            self.send_response(e.code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(error_body)))
            self.end_headers()
            self.wfile.write(error_body)
        except Exception as e:
            error_msg = json.dumps({"error": str(e)}).encode()
            self.send_response(502)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(error_msg)))
            self.end_headers()
            self.wfile.write(error_msg)

    def log_message(self, format, *args):
        print(f"[proxy] {args[0]}", flush=True)


if __name__ == "__main__":
    server = http.server.HTTPServer(("0.0.0.0", LISTEN_PORT), ProxyHandler)
    print(f"LLM proxy listening on 0.0.0.0:{LISTEN_PORT}", flush=True)
    print(f"Forwarding to {TARGET_BASE}", flush=True)
    print(f"API key injected by proxy (not sent from client)", flush=True)
    print(f"ArceAgent should connect to http://10.0.2.2:{LISTEN_PORT}/v1/...", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nProxy stopped.")
        server.server_close()
