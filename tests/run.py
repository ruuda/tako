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


def exec(*args, expect=0):
    """ Run a program with an expected exit code, print stdout on mismatch. """
    p = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if p.returncode != expect:
        print('Process {} exited with unexpected '
              'exit code {}.'.format(args, p.returncode))
        print('\nSTDOUT\n------')
        sys.stdout.buffer.write(p.stdout)
        print('\nSTDERR\n------')
        sys.stdout.buffer.write(p.stderr)
        sys.exit(-1)


def run_server():
    port = 8117
    Handler = http.server.SimpleHTTPRequestHandler
    # Enabling addr reuse ensures that we can run the tests twice in a row,
    # without having to wait a few dozen seconds in between.
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(('', port), Handler) as httpd:
        # The http server logs to stderr. That is causes noise in the test
        # output that we don't care about. Therefore, redirect stderr to
        # /dev/null.
        sys.stderr = open(os.devnull, 'w')
        httpd.serve_forever()


# Run the http server in a background thread. Don't wait for the
# thread to finish if the main thread exits.
httpd_thread = threading.Thread(target=run_server, daemon=True)
httpd_thread.start()

# Secret key of the test key pair that is used in all the tests.
secret_key = ('MFMCAQEwBQYDK2VwBCIEIHRlc3Qta2V5LXZlcnktc2VjdXJpdHktc3VjaC'
              '1zYWZloSMDIQCXQPbwnZ+Ihe9Y9t5k/vCRqr50HnkaXbKyKCX2ZAfb2Q==')

# Clean up results from a previous test run, if there are any.
shutil.rmtree('tests/scratch', ignore_errors=True)
assert not os.path.exists('tests/scratch')
os.mkdir('tests/scratch')
os.mkdir('tests/scratch/foo')
os.mkdir('tests/scratch/bar')
os.mkdir('tests/scratch/bar-origin')

# Print a backtrace if the Rust program crashes.
os.environ['RUST_BACKTRACE'] = '1'

print('tako store')

print(' * stores into an empty server directory')
exec('target/debug/tako', 'store',
     '--key', secret_key,
     '--output', 'tests/scratch/bar-origin',
     'tests/images/1.0.0.img', '1.0.0')
assert os.path.exists('tests/scratch/bar-origin/manifest')
assert os.path.exists('tests/scratch/bar-origin/store/'
                      'a18339e497c231154b9d06c809ef7e03'
                      'a44cd59eb74217c64886b00696ce7062')

print('tako fetch')

print(' * fetches the manifest into an empty destination')
exec('target/debug/tako', 'fetch', 'tests/config/foo-none.tako')
assert os.path.exists('tests/scratch/foo/manifest')

print(' * fetches the manifest into an non-empty destination')
exec('target/debug/tako', 'fetch', 'tests/config/foo-none.tako')
assert os.path.exists('tests/scratch/foo/manifest')

img_v2_sha = '64358f43b990c1473817773028ff27029f4d367bf06595b6948d746fece678cd'
foo_store_img_v2 = 'tests/scratch/foo/store/' + img_v2_sha

print(' * fetches an image into the destination store')
exec('target/debug/tako', 'fetch', 'tests/config/foo-any.tako')
assert os.path.exists('tests/scratch/foo/manifest')
assert os.path.exists(foo_store_img_v2)
# The files in the store must be readonly.
assert not os.access(foo_store_img_v2, os.W_OK)

print(' * does not download an existing image')
exec('target/debug/tako', 'fetch', 'tests/config/foo-any.tako')
# TODO: Add a hook to the webserver, and verify that indeed we did not get a
# request for the image, only for the manifest.
assert os.path.exists(foo_store_img_v2)

print(' * deletes a damaged image')
# Corrupt the file in the store. Running "tako fetch" again should detect this,
# and delete the file (such that on a next run it would be redownloaded).
os.chmod(foo_store_img_v2, int('755', 8))
with open(foo_store_img_v2, 'w') as f:
    f.write('burrito')
os.chmod(foo_store_img_v2, int('555', 8))
# TODO: The expected exit code should be 1 for failure, not 101 (for panic).
exec('target/debug/tako', 'fetch', 'tests/config/foo-any.tako', expect=101)
assert not os.path.exists(foo_store_img_v2)

print(' * fetches a previously stored manifest')
exec('target/debug/tako', 'fetch', 'tests/config/bar.tako')
assert os.path.exists('tests/scratch/bar/manifest')

# TODO: Test that Tako follows redirects.
# TODO: Test that Tako handles file-not-found correctly (whatever that means).

print('All tests passed.')
