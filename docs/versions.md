# Versions

Tako does not enforce any particular versioning scheme. Instead it implements a
simple version parser and constraint resolver that is compatible with most
versioning schemes, such as [Semantic Versioning][semver].

## Patterns

Compatible versions are selected by matching versions against a pattern.
A pattern is either a **wildcard pattern** or a **bounds pattern**.

 * A wildcard pattern is a version that may end in `*`.
 * A bounds pattern specifies an inclusive lower bound and an exclusive upper
   bound separated by ` <= v < `.

Some examples:

 * `*` matches any version.
 * `1.*` matches `1.0`, `1.1`, `1.2`, etc., but not `2.0`, or `0.1`.
   It does match `1` and `1.0.0`, which are both equivalent to `1.0`.
 * `1.13.7` matches `1.13.7`. It also matches `1.13.7.0`, `1.13.7.0.0`,
   etc., which are equivalent to `1.13.7`. Such a pattern can be used for
   version pinning.
 * `1.0 <= v < 2.0` is the same as `1.*`.
 * `2.3 <= v < 3.0` matches versions that are [semver-compatible][semver]
   with 2.3.

## Syntax

Versions consist of **parts** separated by **separators**.

 * A part is either numeric or a string. A numeric part is an unsigned integer.
 * A separator is one of `.`, `-`, and `_`.

Versions map to a list of parts. Some examples:

 * `1.0.0` ⇒ `[1, 0, 0]`
 * `1.0.0-beta` ⇒ `[1, 0, 0, "beta"]`
 * `1.0.0-beta-25856-ge63d979e22` ⇒ `[1, 1, 0, "beta", 25856, "ge63d979e22"]`
 * `1.0.0-beta.2` ⇒ `[1, 0, 0, "beta", 2]`
 * `1.1.0.h-1` ⇒ `[1, 1, 0, "h", 1]`
 * `66.0.3359.117-1` ⇒ `[66, 0, 3359, 117, 1]`
 * `1.2a` ⇒ `[1, "2a"]`

## Ordering

Versions are ordered conventionally. They are compared part by part. String
parts order before numeric parts, and string parts order lexicographically.
Versions are implicitly padded with zero parts: when two versions with a
different number of parts are compared, the shorter one is padded with zeros.
It is generally a bad idea to use a versioning scheme with a variable number
of parts though. Separator characters do not affect ordering.

Some examples:

 * `1.0.0` &lt; `2.0.0`
 * `1.0.0` &lt; `1.1.0`
 * `1.0.0` &lt; `1.0.1`
 * `1` = `1.0`
 * `1-0` = `1.0`
 * `1.a` &lt; `1.0` (string parts order before numeric parts)
 * `1.0.0.a` &lt; `1.0.0` (string parts order before the implicit zero part)
 * `1.0.a` &lt; `1.0.b`
 * `1.0.0-beta.1` &lt; `1.0.0` (string parts order before the implicit zero part)
 * `1.0.0-beta` &lt; `1.0.0-beta.1` (implicit zero orders before 1)
 * `1.0.0-beta.1` &lt; `1.0.0-beta.2`
 * `1.2a` &lt; `1.1` (string parts order before numeric parts)

Some of these might be counterintuitive. Unfortunately we cannot have both of
the following be true without complicating the comparison rules.

 * `1.0-beta.1` &lt; `1.0`.
 * `1.0` &lt; `1.0.a`

Tako implements the first choice, and hence `1.0.a` &lt; `1.0`. To avoid
confusion, use a versioning scheme that has a fixed number of parts.

[semver]: https://semver.org/
