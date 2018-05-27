# Roadmap

The version numbers in this list should not be interpreted literally. Rather,
they indicate near-term versus long-term goals.

## Done

 * Rename `RestartUnit=` to `Restart=`.
 * Allow space-separated units in `Restart=`.

## 0.1

 * Implement `tako fetch --init`.
 * Ensure it runs on CoreOS.
 * Check for spaces in versions.
 * Keep a changelog.

## 0.2

 * Thoroughly test failure modes, not only success path.
 * Print friendly errors, have right exit codes.
 * Implement `x <= v < y` version pattern.
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
