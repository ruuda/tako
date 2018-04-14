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
