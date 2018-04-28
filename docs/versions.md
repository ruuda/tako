# Versions

Tako does not enforce any particular versioning scheme. Instead it implements a
simple version parser and constraint resolver that is compatible with most
versioning schemes, such as [Semantic Versioning][semver].

## Syntax

Version numbers consist of **parts** separated by **separators**.

 * A part is either numeric or a string. A numeric part is an unsigned integer.
 * A separator is one of `.`, `-`, and `_`.

Version numbers are mapped to a list of parts. Some examples:

 * `1.0.0` ⇒ `[1, 0, 0]`
 * `1.0.0-beta` ⇒ `[1, 0, 0, "beta"]`
 * `1.0.0-beta-25856-ge63d979e22` ⇒ `[1, 1, 0, "beta", 25856, "ge63d979e22"]`
 * `1.0.0-beta.2` ⇒ `[1, 0, 0, "beta", 2]`
 * `1.1.0.h-1` ⇒ `[1, 1, 0, "h", 1]`
 * `66.0.3359.117-1` ⇒ `[66, 0, 3359, 117, 1]`
 * `1.2a` ⇒ `[1, "2a"]`

## Ordering

Version numbers are ordered in the regular way. Version numbers are compared
by parts. String parts order before numeric parts, and string parts order
lexicographically. Versions with less parts order before versions with more
parts, although it is generally a bad idea to have a versioning scheme with a
variable number of parts. Missing parts are not implicitly zero. Separator
characters do not affect ordering.

Some examples:

 * `1.0.0` &lt; `2.0.0`
 * `1.0.0` &lt; `1.1.0`
 * `1.0.0` &lt; `1.0.1`
 * `1` &lt; `1.0` (note that these are *not* equal)
 * `1-0` = `1.0`
 * `1.a` &lt; `1.0`
 * `1.0.0` &lt; `1.0.0.a`
 * `1.0.a` &lt; `1.0.b`
 * `1.0.0` &lt; `1.0.0-beta.1`
 * `1.0.0-beta` &lt; `1.0.0-beta.1`
 * `1.0.0-beta.1` &lt; `1.0.0-beta.2`
 * `1.2a` &lt; `1.1` (string parts order before numeric parts)

## Patterns

Compatible versions are selected by matching version numbers against a pattern.
A pattern is either a **wildcard pattern** or a **bounds pattern**.

 * A wildcard pattern is a version string that may end in `*`.
 * A bounds pattern specifies an inclusive lower bound and an exclusive upper
   bound separated by ` <= v < `.

Some examples:

 * `1.*` matches `1.0`, `1.1`, `1.2`, etc., but not `2.0`, `0.1`, nor `1`.
 * `*` matches any version.
 * `1.13.7` matches only `1.13.7`. Such a pattern can be used for
   version pinning.
 * `1.0 <= v < 2.0` is the same as `1.*`.
 * `2.3 <= v < 3.0` matches versions that are [semver][semver]-compatible
   with 2.3.

[semver]: https://semver.org/
