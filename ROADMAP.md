# Roadmap

The version numbers in this list should not be interpreted literally. Rather,
they indicate near-term versus long-term goals.

## Done

 * Rename `RestartUnit=` to `Restart=`.
 * Allow space-separated units in `Restart=`.
 * Replace [`ring`][ring] with [`ed25519-dalek`][dalek] and [`sha2`][sha2].
   Ring’s versioning and stability policy is problematic; Ring cannot securely
   be used with Rust 1.24 which we aim to support. RustCrypto at least aims to
   support Rust 1.21 at the time of writing. Hindsight note: actually, we later
   opted for [`libsodium`][sodium] with Rust bindings.
 * Implement `tako fetch --init`.
 * Implement `x <= v < y` version pattern.

## 0.1

 * Ensure it runs on CoreOS.
 * Check for spaces in versions.
 * Keep a changelog.

## 0.2

 * Thoroughly test failure modes, not only success path.
 * Print friendly errors, have right exit codes.
 * Fuzz the manifest parser and config file parser.
 * Implement restarting units.

## 0.3

 * Add `--[no-]verify` switch to `tako fetch` to control whether to verify
   integrity of files in the store.
 * Implement size constraints on the store.
 * Handle 410 Gone responses from the server.
 * Support for blue/geen deployments (keep an A and B symlink, restart one of
   two services).

## 0.4

 * Reduce binary size.
 * Investigate using etag headers to reduce bandwidth.

[ring]:   https://github.com/briansmith/ring
[dalek]:  https://github.com/dalek-cryptography/ed25519-dalek
[sha2]:   https://github.com/RustCrypto/hashes
[sodium]: https://libsodium.org
