#!/usr/bin/env python3
from http.server import HTTPServer, SimpleHTTPRequestHandler
import sys
import argparse

parser = argparse.ArgumentParser(description='run a simple host')
parser.add_argument('port', type=int, help='port in server')

args = parser.parse_args()
PORT = args.port if int(args.port) > 0 else 8083


class RequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        SimpleHTTPRequestHandler.end_headers(self)


if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else PORT
    httpd = HTTPServer(('localhost', port), RequestHandler)
    print("👀 serving at http://localhost:{}".format(PORT))
    httpd.serve_forever()
