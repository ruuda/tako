#!/usr/bin/env python3

"""
A basic Python script that starts a server and tests a few properties by running
Tako. Test output is TAP compliant (see https://testanything.org), but the
script can also be used standalone.
"""

import contextlib
import http.server
import os
import os.path
import shutil
import socketserver
import subprocess
import sys
import threading
import traceback

# Print the number of tests in advance. This is understood by the TAP protocol,
# and used to recognize failure when the script unexpectedly exits early.
print('1..7')

is_repo_root = os.path.exists(os.path.join(os.getcwd(), 'README.md'))
assert is_repo_root, 'This script must be run from the root of the repository.'

test_number = 0
test_failed = False

class ExecError(Exception):
    """ Raised from exec(), used to print stderr after test description. """

    def __init__(self, command, code, stdout, stderr):
        self.command = command
        self.code = code
        self.stdout = stdout
        self.stderr = stderr

    def print_details(self):
        print('# Process {} exited with unexpected '
              'exit code {}.'.format(self.command, self.code))
        print('\nSTDOUT\n------')
        sys.stdout.flush()
        sys.stdout.buffer.write(self.stdout)
        print('\nSTDERR\n------')
        sys.stdout.flush()
        sys.stdout.buffer.write(self.stderr)
        print('')


def exec(*args, expect=0):
    """ Run a program with an expected exit code, raise error on mismatch. """
    p = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if p.returncode != expect:
        raise ExecError(args, p.returncode, p.stdout, p.stderr)


@contextlib.contextmanager
def test(description):
    """ Context manager to run a TAP test. Use regular assert/raise inside. """
    global test_number, test_failed
    test_number = test_number + 1
    try:
        yield
    except:
        print('not ok {} {}'.format(test_number, description))
        exc_type, error, tb = sys.exc_info()
        _filename, line, _function, statement = traceback.extract_tb(tb)[-1]
        print('# line {}: {}'.format(line, statement))
        if exc_type is ExecError:
            error.print_details()
        test_failed = True
    else:
        print('ok {} {}'.format(test_number, description))


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

print('\n# tako store\n')

img_v1_sha = 'a18339e497c231154b9d06c809ef7e03a44cd59eb74217c64886b00696ce7062'

with test('Stores into an empty server directory'):
    exec('target/debug/tako', 'store',
         '--key', secret_key,
         '--output', 'tests/scratch/bar-origin',
         'tests/images/1.0.0.img', '1.0.0')
    assert os.path.exists('tests/scratch/bar-origin/manifest')
    assert os.path.exists('tests/scratch/bar-origin/store/' + img_v1_sha)

print('\n# tako fetch\n')

with test('Fetches the manifest into an empty destination'):
    exec('target/debug/tako', 'fetch', 'tests/config/foo-none.tako')
    assert os.path.exists('tests/scratch/foo/manifest')

with test('Fetches the manifest into an non-empty destination'):
    exec('target/debug/tako', 'fetch', 'tests/config/foo-none.tako')
    assert os.path.exists('tests/scratch/foo/manifest')

img_v2_sha = '64358f43b990c1473817773028ff27029f4d367bf06595b6948d746fece678cd'
foo_store_img_v2 = 'tests/scratch/foo/store/' + img_v2_sha
store_img_v2 = 'store/' + img_v2_sha

with test('Fetches an image into the destination store'):
    exec('target/debug/tako', 'fetch', 'tests/config/foo-any.tako')
    assert os.path.exists('tests/scratch/foo/manifest')
    assert os.path.exists(foo_store_img_v2)
    # The files in the store must be readonly.
    assert not os.access(foo_store_img_v2, os.W_OK)
    assert os.readlink('tests/scratch/foo/latest') == store_img_v2

with test('Does not download an existing image'):
    exec('target/debug/tako', 'fetch', 'tests/config/foo-any.tako')
    # TODO: Add a hook to the webserver, and verify that indeed we did not
    # get a # request for the image, only for the manifest.
    assert os.path.exists(foo_store_img_v2)
    assert os.readlink('tests/scratch/foo/latest') == store_img_v2

with test('Deletes a damaged image'):
    # Corrupt the file in the store. Running "tako fetch" again should
    # detect this, and delete the file (such that on a next run it would
    # be # redownloaded).
    os.chmod(foo_store_img_v2, int('755', 8))
    with open(foo_store_img_v2, 'w') as f:
        f.write('burrito')
    os.chmod(foo_store_img_v2, int('555', 8))
    # TODO: The expected exit code should be 1 for failure, not 101 (for panic).
    exec('target/debug/tako', 'fetch', 'tests/config/foo-any.tako', expect=101)
    assert not os.path.exists(foo_store_img_v2)

with test('Fetches a previously stored manifest and image'):
    exec('target/debug/tako', 'fetch', 'tests/config/bar.tako')
    assert os.path.exists('tests/scratch/bar/manifest')
    assert os.readlink('tests/scratch/bar/latest') == 'store/' + img_v1_sha

with test('Aborts a fetch if the remote serves a larger file than expected.'):
    # Make the origin serve a file that is larger than advertised in the
    # manifest by appending some bytes to the file in the remote store.
    # TODO: I should have a special binary to make malicious manifests, that do
    # have a good sha256sum for the larger file, but a size in the manifest that
    # does not match.
    fname = 'tests/scratch/bar-origin/store/' + img_v1_sha
    os.chmod(fname, int('755', 8))
    with open(fname, 'w') as f:
        f.write('E X T R A   B Y T E S')
    os.chmod(fname, int('555', 8))

    # Remove the local file so Tako will download it again.
    os.remove('tests/scratch/bar/store/' + img_v1_sha)
    exec('target/debug/tako', 'fetch', 'tests/config/bar.tako')
    # TODO: Expect error message, so we know it was not the sha256sum that
    # failed?
    assert not os.path.exists('tests/scratch/bar/store/' + img_v1_sha)

# TODO: Test that Tako follows redirects.
# TODO: Test that Tako handles file-not-found correctly (whatever that means).

if test_failed:
    print('Some tests failed.')
    sys.exit(1)
