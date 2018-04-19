#!/usr/bin/env python3

import http.server
import os
import os.path
import shutil
import socketserver
import subprocess
import sys
import threading

is_repo_root = os.path.exists(os.path.join(os.getcwd(), 'README.md'))
assert is_repo_root, 'This script must be run from the root of the repository.'


def exec(args):
    p = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if p.returncode != 0:
        print('Process {} exited with nonzero exit code.'.format(args))
        print('\nSTDOUT\n------')
        sys.stdout.buffer.write(p.stdout)
        print('\nSTDERR\n------')
        sys.stdout.buffer.write(p.stderr)
        sys.exit(-1)


def run_server():
    port = 8117
    Handler = http.server.SimpleHTTPRequestHandler
    with socketserver.TCPServer(('', port), Handler) as httpd:
        httpd.serve_forever()


# Run the http server in a background thread. Don't wait for the
# thread to finish if the main thread exits.
httpd_thread = threading.Thread(target=run_server, daemon=True)
httpd_thread.start()

# Clean up results from a previous test run, if there are any.
shutil.rmtree('tests/scratch')
os.mkdir('tests/scratch')
os.mkdir('tests/scratch/foo')

# Print a backtrace if the Rust program crashes.
os.environ['RUST_BACKTRACE'] = '1'

print('tako fetch')

print(' * fetches the manifest into an empty destination')
exec(['target/debug/tako', 'fetch', 'tests/config/foo.tako'])
assert os.path.exists('tests/scratch/foo/manifest')

print(' * fetches the manifest into an non-empty destination')
exec(['target/debug/tako', 'fetch', 'tests/config/foo.tako'])
assert os.path.exists('tests/scratch/foo/manifest')
