mono_title: true

# tako store

Add a new image version to a server directory.

## Synopsis

    tako store [-k <key> | -f <file>] --output <dir> [--] <image> <version>
    tako store -h | --help

## Description

This commands adds a new file to the store and updates the manifest. It computes
the SHA256 of the `<image>` file and copies it into `<dir>/store` using the hash
as filename. Tako then adds an entry to the manifest in `<dir>` that specifies
that `<version>` corresponds to the computed hash. Tako signs the updated
manifest with the provided secret key.

See [Versions](versions.md) for information on how Tako treats `<version>`. For
proper ordering, versions should start with a digit: use `1.0.0` rather than
`v1.0.0`. The version must not contain spaces.
<!-- TODO: This is not actually verified and would corrupt the manifest. -->

The secret key which is used to sign the manifest can be provided in thee ways:

 * By setting the `TAKO_SECRET_KEY` environment variable.
 * By passing the secret key directly on the command line with `--key`.
 * By reading the secret key from a file with `--key-file`.

Command-line options take precedence over the environment variable. A key pair
can be generated with [`tako gen-key`](tako-gen-key.md).

## Options

### `-k` `--key <key>`

Provide the secret key `<key>` directly.

### `-f` `--key-file <file>`

Read the secret key from `<file>`.

### `-o` `--output <dir>`

Specifies the server directory. `<dir>` must exist. This must be a path on the
file system, uris are not supported. Tako will create `<dir>/manifest` and
`<dir>/store` if they do not exist yet.
