# Tako

Tako securely downloads and updates binary files such as container images. It
is intended as a lightweight delivery mechanism for signed versioned images.
Through version bounds Tako enables automatic security updates while avoiding
breaking changes. Ed25519 signatures ensure that images come from a trusted
source.

Tako is a short-lived process that downloads images specified in its
configuration and then exits. Optionally Tako restarts configured systemd units
when it downloads a newer version of an image. Tako runs on two occasions:

 * Periodically, triggered by a systemd timer. Tako checks for new compatible
   versions of a configured image. If one exists, Tako downloads it and restarts
   the systemd unit that uses the image.
 * As a dependency of the systemd unit that uses the image, to provision a clean
   system with an initial image.

Tako is a single binary with minimal dependencies (libc and libcurl only),
because Tako is used to bootstrap more complex applications. Installing Tako is
as easy as copying over the binary.

## User Guide

 * [Downloading Images](downloading-images.md)
 * [Distributing Images](distributing-images.md)
 * [Configuration](configuration.md)
 * [Versions](versions.md)

## Reference

 * [tako fetch](tako-fetch.md)
 * [tako store](tako-store.md)
 * [tako gen-key](tako-gen-key.md)

## Internals

 * [Manifest Format](manifest-format.md)
