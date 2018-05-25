# Manifest Format

The manifest lists all available image versions and their SHA256 hashes.
The format is human-readable for convenience.

## Structure

The manifest starts with a line `Tako Manifest 1` that identifies the file as
a manifest.

After the header is a blank line. Then follows one image version per line,
formatted as the version number, a space, the file size in bytes, a space, and
the hexadecimally encoded SHA256 of the image. This makes it easy to use
`sha256sum` as a sanity check. Versions are sorted by version number.

After the image versions is again a blank line, followed by the base64-encoded
Ed25519 signature of all of the preceding content (including newlines).

Newlines are a single line feed (`\n`). Version numbers should be ascii. Hence
the entire file is valid ascii, and also valid UTF-8.

## Example

    Tako Manifest 1

    1.0.0 10092569 b101acf3c4870594bb4363090d5ab966c193fb329e2f2db2096708e08c4913e2
    1.1.0 11239411 9641a49d02e90cbb6213f202fb632da70cdc59073d42283cfcdc1d786454f17f
    2.0.0 11862029 b7b01c6f6772529c66b945e559cb1f46546ef62063e44c1d1068725157ae1cda

    fQK92C/tPnH0uqxrTEnU+LEE4jnSpQPbOItph4kGAEfWEmn6wPXiQsSdXlDmoneaJkG6KLvInTvB7FlELoeQFg==

## Rationale

The manifest format is inspired by the well-established practice of distributing
a GPG-signed `SHASUMS` file.

* We include a version line, so future versions of Tako can still read older
  manifests.
* The signature is embedded, rather than external, to avoid race conditions when
  uploading a new manifest to the server. One would still need to upload new
  images before uploading a new manifest.
* The signature is an Ed25519 signature rather than a GPG signature, because GPG
  involves a stateful trust store that is difficult to provision in an automated
  way. In practice people use GPG is only for signature verification. For
  authentication, rather than relying on GPG’s web of trust, people announce
  the fingerprint of their key in a trusted location (Twitter, Github, an https-
  protected website). Ed25519 public keys are small enough that the full public
  key can be announced in places where we would normally announce a fingerprint.
* The manifest does not include timestamps, to ensure that it is reproducible.
  Timestamps belong in a changelog or audit log.
* Entries should never be removed from the manifest. There are reasons to stop
  providing an image (for instance because it contained a critical bug that
  causes data loss). In that case the image itself can be removed from the
  server, but it should still be listed in the manifest. This prevents
  accidentally releasing different images under the same version number. It also
  ensures that clients which did download the image can still identify it, so
  they do not end up running a mysterous image without record of existence.
* The size of each image is included, and signed, so malicious mirrors cannot
  cause clients to download large files that would fill up their disks.
