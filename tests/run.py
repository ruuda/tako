#!/usr/bin/env python3

import http.server
import os
import os.path
import shutil
import socketserver
import subprocess
import threading

is_repo_root = os.path.exists(os.path.join(os.getcwd(), 'README.md'))
assert is_repo_root, 'This script must be run from the root of the repository.'


def exec(args):
    subprocess.run(args).check_returncode()


def run_server():
    port = 8117
    Handler = http.server.SimpleHTTPRequestHandler
    httpd = socketserver.TCPServer(('', port), Handler)
    httpd.serve_forever()


# Run the http server in a background thread. Don't wait for the
# thread to finish if the main thread exits.
httpd_thread = threading.Thread(target=run_server, daemon=True)
httpd_thread.start()

# Clean up results from a previous test run, if there are any.
shutil.rmtree('tests/scratch')
os.mkdir('tests/scratch')

# Print a backtrace if the Rust program crashes.
os.environ['RUST_BACKTRACE'] = '1'

print('tako fetch')

print(' * fetches the manifest')
exec(['target/debug/tako', 'fetch', 'tests/config/foo.tako'])
