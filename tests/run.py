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

# Secret key of the test key pair that is used in all the tests.
secret_key = ('MFMCAQEwBQYDK2VwBCIEIOOLlPLpuO1GOp+ZrvzKNXf8NpA8OtcRcpW2/clbJYyq'
              'oSMDIQCocvTcH9bTcmbVaZ//EpXqD+0hwb56WG/3JWjyIc6tdg==')

# Clean up results from a previous test run, if there are any.
shutil.rmtree('tests/scratch', ignore_errors=True)
assert not os.path.exists('tests/scratch')
os.mkdir('tests/scratch')
os.mkdir('tests/scratch/foo')
os.mkdir('tests/scratch/bar')
os.mkdir('tests/scratch/bar-origin')

# Print a backtrace if the Rust program crashes.
os.environ['RUST_BACKTRACE'] = '1'

print('tako fetch')

print(' * fetches the manifest into an empty destination')
# TODO: Fix tests.
# exec(['target/debug/tako', 'fetch', 'tests/config/foo.tako'])
# assert os.path.exists('tests/scratch/foo/manifest')

print(' * fetches the manifest into an non-empty destination')
# TODO: Fix tests.
# exec(['target/debug/tako', 'fetch', 'tests/config/foo.tako'])
# assert os.path.exists('tests/scratch/foo/manifest')

print('tako store')

print(' * stores into an empty server directory')
exec(['target/debug/tako', 'store',
      '--key', secret_key,
      '--output', 'tests/scratch/bar-origin',
      'tests/images/1.0.0.img', '1.0.0'])
assert os.path.exists('tests/scratch/bar-origin/manifest')
assert os.path.exists('tests/scratch/bar-origin/store/'
                      'a18339e497c231154b9d06c809ef7e03'
                      'a44cd59eb74217c64886b00696ce7062')
