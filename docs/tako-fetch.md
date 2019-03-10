mono_title: true

# tako fetch

Download or update an image.

## Synopsis

    tako fetch [--init] [--] <config>...
    tako fetch -h | --help

## Description

This command takes one or more [config files](configuration.md). For every
provided config file, it:

 * Downloads the latest manifest from the remote into the destination directory.
 * Downloads the latest image, if a newer compatible version exists.
 * Symlinks `latest` in the destination directory to the newly downloaded image.
 * Restarts any configured systemd units.

## Options

### `--init`

If this option is enabled, Tako checks if `manifest` and `latest` exist in the
destination directory. If `latest` points to a valid image, Tako exits
immediately without checking for new versions. In other words, Tako performs a
minimal amount of work while still guaranteeing that an image exists in the
destination directory if the command exits successfully.

This option can be used to provision an clean system with an initial image.
Running `tako fetch` before starting an application that depends on the image
managed by Tako ensures that the image exists when the application starts. When
Tako exits with a zero exit code, the image is guaranteed to exist. However,
without `--init` Tako will always download the manifest, and possibly
download a new image, even if an older compatible image exists that could be
used. This delays application startup, and may prevent startup entirely if
fetching fails (due to connectivity issues, for instance). With `--init`, Tako
only performs any work if required to start the dependent application.
