# Changelog `envious`

## v0.3.0

The 0.3.0 version makes the following API changes:

- `Config` is the new entrypoint to `envious`, with all interactions happening via method calls.
- Existing functions are now wrappers around `Config` and will persist until `0.4.0` at which point they will be removed. This is indicated by deprecation warnings on the functions.

The 0.2.0 version adds the following features:

- Case sensitivity is now configurable, see `Config::case_sensitive`
- The separator is now configurable, using the same default as `0.2.0`, see `Config::with_separator`

## v0.2.0

The 0.2.0 version adds the following features

- Add `from_iter` allowing to deserialize from any source of `(String, String)` pairs

## v0.1.1

The 0.1.1 version adds more and better documentation to the main readme.

## v0.1.0

The 0.1.0 version is the initial public release of the `envious` crate.

Added:

- Initial implementation to deserialize structs with serde from environment variables
