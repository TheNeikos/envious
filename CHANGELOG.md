# Changelog `envious`

## v0.2.0

The 0.2.0 version makes the following API changes:

- `Config` is the new entrypoint to `envious`, with all interactions happening via method calls.
- Case sensitivity is now configurable, see `Config::case_sensitive`
    - NB: The default is case insensitive, which is a change from the original behaviour.
- The separator is now configurable, using the same default as `0.1.x`, see `Config::with_separator`
- Add `from_iter` allowing to deserialize from any source of `(String, String)` pairs

## v0.1.1

The 0.1.1 version adds more and better documentation to the main readme.

## v0.1.0

The 0.1.0 version is the initial public release of the `envious` crate.

Added:

- Initial implementation to deserialize structs with serde from environment variables
