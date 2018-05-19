# Downloading Images

To download images with Tako, the image distributor should have provided you
with two things:

 * A remote url, such as `https://images.example.com/app-foo`.
 * A public key, such as `l0D28J2fiIXvWPbeZP7wkaq+dB55Gl2ysigl9mQH29k=`.

The next step is to create a [configuration file](configuration.md) to tell Tako
what to fetch from that remote, and where to store it. For example, write the
following to `example.tako`:

    Origin=https://images.example.com/app-foo
    PublicKey=l0D28J2fiIXvWPbeZP7wkaq+dB55Gl2ysigl9mQH29k=
    Destination=/tmp/app-foo
    Version=*

Download the latest available image with [`tako fetch`](tako-fetch.md):

    $ tako fetch example.tako
    $ file /tmp/app-foo/latest

Now `/tmp/app-foo/latest` is a symlink to the latest image, which itself is
stored as a readonly file in `/tmp/app-foo/store`.

## Automating Updates

To use Tako to keep the image up to date, run Tako periodically, for example
using a [systemd timer][systemd-timer]. Using [`RandomizedDelaySec=`][delay] is
recommended to avoid overloading the remote server.

<!-- TODO: Elaborate, make more beginner-friendly. -->

## Initial Provisioning

See [`tako fetch --init`](tako-fetch.md#-init).

<!-- TODO: Elaborate. -->

[systemd-timer]: https://www.freedesktop.org/software/systemd/man/systemd.timer.html
[delay]:         https://www.freedesktop.org/software/systemd/man/systemd.timer.html#RandomizedDelaySec=
