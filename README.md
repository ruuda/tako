# Tako

Tako: take container image.

## Goals

Goals:

 * Securely downloading signed images.
 * Implement a versioning policy, to be able to download the latest compatible
   version of an image. Automatic security updates, without installing new
   versions with breaking changes.

Non-goals:

 * Reinvent apt packaging. In particular: no scriptable install steps or
   extensive metadata. Just a signed filesystem image. Not even systemd unit
   files.
 * Be a container runtime. Systemd is a decent container runtime.

## Overview

Tako is a short-lived process that downloads images specified in its
configuration and then exits. Optionally Tako restarts configured systemd units
when it downloads a newer version of an image. Take runs on two occasions:

 * Periodically, triggered by a systemd timer. Tako will poll for new compatible
   versions of a configured image. If one exists, Tako downloads it and restarts
   the systemd unit that uses the image.
 * As a dependency of the systemd unit that uses the image, to provision a clean
   system with an initial image.

## Usage

Command-line interface:

    # Initially fetch an image, but do nothing if any image exists already.
    tako --if-not-exists /etc/tako/yourapp

    # Check for, download, and apply available updates.
    tako /etc/tako/yourapp

    # Update multiple images at once.
    tako /etc/tako/app-foo /etc/tako/app-bar

Configuration file example:

    Origin=https://images.example.com/app-foo
    PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=
    Destination=/var/lib/images/app-foo
    RestartUnit=app-foo.service

If multiple units share the same image, it is possible to specify multiple units
to restart:

    Origin=https://images.example.com/app-foo
    PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=
    Destination=/var/lib/images/app-foo
    RestartUnit=app-foo.service
    RestartUnit=app-bar.service

The `RestartUnit=` key is optional.

## Building

    rustup target add x86_64-unknown-linux-musl
    cargo build --release
    target/x86_64-unknown-linux-musl/release/tako

## Server

A Tako server is a regular http server, with a particular directory layout. The
origin uri points to a directory where we can find the manifest file, that lists
all available versions and their SHA256 digests. The manifest is signed.

The manifest file is a text file, one image version per line (separated by
`\n`). Every line contains the version number, a space, and then the SHA256
of the image (encoded hexadecimally). The first line contains the base64-encoded
Ed25519 signature of the remainder of the file (newline not included).

    fQK92C/tPnH0uqxrTEnU+LEE4jnSpQPbOItph4kGAEfWEmn6wPXiQsSdXlDmoneaJkG6KLvInTvB7FlELoeQFg==
    1.0.0 b101acf3c4870594bb4363090d5ab966c193fb329e2f2db2096708e08c4913e2
    1.1.0 9641a49d02e90cbb6213f202fb632da70cdc59073d42283cfcdc1d786454f17f
    2.0.0 b7b01c6f6772529c66b945e559cb1f46546ef62063e44c1d1068725157ae1cda

## Local Store

Tako downloads images into a destination directory. It creates the following
files there (`//` indicates the destination directory path).

    //store/<hexdigest>  # Raw image file.
    //manifest           # A copy of the manifest served by the origin.
    //latest             # Symlink to the latest image.

## Future work

 * GC'ing the local store.
