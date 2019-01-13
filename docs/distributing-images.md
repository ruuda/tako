# Distributing Images

A Tako server is a regular http server, with a particular directory layout.
The origin uri points to a directory that contains the manifest file and the
image store.

 * `manifest` is a file that lists all available versions and their SHA256
   digests. The manifest is signed. See [Manifest Format](manifest-format.md)
   for more information about the manifest format.
 * `store` is a directory that contains all images. Files are named after their
   digest.

Images can be added to the directory with [`tako store`](tako-store.md). The
most convenient way to maintain the server directory is to have a local copy
that Tako can act on, and to sync that to a server.

## Immutable Images

Tako is designed as an append-only system where images are immutable. Changing
an image is not possible: publish it as a new version instead. The Tako client
[`tako fetch`](tako-fetch.md) stores a copy of the manifest locally, and when it
downloads a new version of the manifest, it must be a superset of the local
manifest. If the hash of a particular version has changed in the remote
manifest, or if a version was removed, the client rejects the new manifest, even
if it has a valid signature.

In some occasions, it might be necessary to remove a previously published image.
To do so, simply stop serving the image by removing it from the `store`
directory. The manifest will still list the image. If a client selects that
particular image as a candidate to download, the [`tako fetch`](tako-fetch.md)
will fail. This is generally not an issue if a newer compatible version is
available. If this is not the case, you can configure the server to serve “410
Gone” on the url of the removed image. In the future — the following has not yet
been implemented — if [`tako fetch`](tako-fetch.md) encounters a 410 it will not
fail, but instead select an earlier compatible version and try again.
<!-- TODO: Implement handling of 410 at some point. -->
