# Tako

Tako: take container image.

Tako securely downloads and updates binary files such as container images. It
is intended as a lightweight delivery mechanism for signed versioned images.
Through version bounds Tako enables automatic security updates while avoiding
breaking changes. Ed25519 signatures ensure that images come from a trusted
source.

Tako is a short-lived process that downloads images specified in its
configuration and then exits. Optionally Tako restarts configured systemd units
when it downloads a newer version of an image.

Tako can be used in conjunction with systemd as a more flexible alternative
to container runtimes. [Systemd can take care of the sandboxing
part][containers-systemd]. Tako takes care of versioned image acquisition and
automatic updates.

## Documentation

 * [Overview](docs/index.md)
 * [Downloading Images](docs/downloading-images.md)
 * [Distributing Images](docs/distributing-images.md)
 * [Configuration](docs/configuration.md)
 * [Versions](docs/versions.md)
 * [`tako fetch`](docs/tako-fetch.md)
 * [`tako store`](docs/tako-store.md)
 * [`tako gen-key`](docs/tako-gen-key.md)
 * [Manifest Format](docs/manifest-format.md)

## Goals

Goals:

 * Securely downloading signed images.
 * Implement a versioning policy, to be able to download the latest compatible
   version of an image. Automatic security updates, but not new versions with
   breaking changes without manual intervention.

Non-goals:

 * Reinvent apt packaging. In particular: no scriptable install steps or
   extensive metadata. Just a signed filesystem image. Not even systemd unit
   files.
 * Delivering multiple files. Tako can download a tar archive, but it will not
   extract it for you.
 * Be a container runtime. Systemd is a decent container runtime.

## Building

    cargo build --release
    target/release/tako --help

If you want to build with statically linked libsodium, build a static libsodium
from source:

    autogen.sh
    ./configure --disable-shared
    make

Then build Tako, passing the path to the static libsodium:

    export SODIUM_STATIC=true
    export SODIUM_LIB_DIR=/path/to/libsodium/checkout/src/libsodium/.libs
    export SODIUM_INC_DIR=/path/to/libsodium/checkout/src/libsodium/include

## Future work

 * GC'ing the local store.
 * Differential updates. (Bsdiff, Casync?)

[containers-systemd]: https://media.ccc.de/v/ASG2017-101-containers_without_a_container_manager_with_systemd
