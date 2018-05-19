# Configuration

Tako determines what to fetch and where to store images from a config file, one
per image. Config files follow the same syntax as systemd unit files.

## Example

    Origin=https://images.example.com/app-foo
    PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=
    Destination=/var/lib/images/app-foo
    Version=1.*

    # Restart app-foo after a new image has been fetched.
    RestartUnit=app-foo.service

## Options

The following options are available. Unless noted otherwise, all options must be
specified exactly once.

### `Origin=`

Remote uri to fetch the manifest and images from. Usually an https url, but
anything supported by Curl will do, such as a `file://` or `ssh://` uri. The uri
must point to a directory that contains a manifest file. A trailing slash is
allowed, but not required.

### `PublicKey=`

Public key used to verify image interity and authenthicity. The public key
should be announced by the distributor of the image. Distributors can generate
a key pair with [`tako gen-key`](tako-gen-key.md).

### `Destination=`

Directory where images will be stored. This directory must exist. Tako will
create a `store` subdirectory to hold images, a `manifest` file which is a copy
of the remote manifest, and a `latest` symlink that points to the latest
compatible image in the store. A trailing slash is allowed, but not required.

### `Version=`

A version pattern that indicates which version range is compatible. Tako will
fetch the latest (highest numbered) compatible version. The version pattern
can be a fixed version, a wildcard pattern, or a bounds pattern. See
[Versions](versions.md) for more information.

### `Restart=`

A systemd unit to restart in case a newer image has been fetched. The format is
the same as that of [`Requires=`][systemd-requires] in systemd units. This
option may be specified more than once, or multiple space-separated units may be
specified in one option. This option is not required: if it is not set, no unit
will be restarted.

## Comments

Like systemd unit files, lines starting with `#` or `;` are ignored. Empty lines
are ignored as well.

[systemd-requires]: https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Requires=
