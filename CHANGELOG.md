# Changelog `envious`

Breaking change: Although no code will fail to compile, the following change may break assumptions on current behaviour:

- Arrays read from environment variables with numeric keys (e.g. `array__1__value=5`) will now be sorted by those keys
- Arrays read from environment variables with non-numeric keys (e.g. `array__a__value=5`) will remain in the order we are given the environment variables by the Operating System

## v0.2.1

The 0.2.1 version fixes several unexpected behaviours w.r.t. case sensitivity:

- When case insensitive, environment variables with the same keys but different casing should be collated
  - i.e. previously `database__password` and `DATABASE__username` would have conflicted and only one the username or password would be read
- When case insensitive, the prefix is also treated insensitively

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
