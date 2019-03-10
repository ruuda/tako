mono_title: true

# tako gen-key

Generate a key pair for signing manifests.

## Synopsis

    tako gen-key
    tako gen-key -h | --help

## Description

This command generates an [Ed25519][ed25519] key pair. Both are printed to
stdout, base64 encoded. The secret key should be kept in a secure location,
for example in a password manager, or in a secret store such as [Vault][vault]
if automated signing is desired. Tako avoids writing the secret key to disk:
the secret key is not protected by a passphrase (like SSH keys can be), so
if you do want to store the secret key in a plain file, be sure to use full-disk
encryption. The public key (the shorter one) should be announced to end users.

The secret key is prefixed with `SECRET+`, to reduce the risk of mistaking the
secret key for public data. The prefix is part of the key, it must always be
included.

[ed25519]: https://ed25519.cr.yp.to/
[vault]:   https://www.vaultproject.io/
