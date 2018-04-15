# Manifest Format

The manifest lists all available image versions and their SHA256 hashes. The
manifest is signed, and the signature is embedded, rather than external. This
prevents race conditions when uploading a new manifest to a server. The format
is human-readable for convenience.

## Structure

The manifest starts with a line `Tako Manifest 1` that identifies the file as a
manifest. It includes a version number, so future versions of Tako can still
read older manifests.

After the header is a blank line. Then follows one image version per line,
formatted as the version number, a space, and the hexadecimally encoded SHA256
of the image. This makes it easy to use `sha256sum` as a sanity check. Versions
are sorted by version number.

After the image versions is again a blank line, followed by the base64-encoded
Ed25519 signature of all of the preceding content (including newlines).

Newlines are a single line feed (`\n`). Version numbers should be ascii. Hence
the entire file is valid ascii, and also valid UTF-8.

## Example

    Tako Manifest 1

    1.0.0 b101acf3c4870594bb4363090d5ab966c193fb329e2f2db2096708e08c4913e2
    1.1.0 9641a49d02e90cbb6213f202fb632da70cdc59073d42283cfcdc1d786454f17f
    2.0.0 b7b01c6f6772529c66b945e559cb1f46546ef62063e44c1d1068725157ae1cda

    fQK92C/tPnH0uqxrTEnU+LEE4jnSpQPbOItph4kGAEfWEmn6wPXiQsSdXlDmoneaJkG6KLvInTvB7FlELoeQFg==
