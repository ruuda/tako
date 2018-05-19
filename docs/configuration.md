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

## Keys

  * `Origin=`: TODO
  * `PublicKey=`: TODO
  * `Destination=`: TODO
  * `Version=`: TODO
  * `RestartUnit=`: TODO

## Comments

Like systemd unit files, lines starting with `#` or `;` are ignored. Empty lines
are ignored as well.
