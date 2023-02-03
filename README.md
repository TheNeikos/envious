
![envious cover image](./cover.png)

[![Bors enabled](https://bors.tech/images/badge_small.svg)](https://app.bors.tech/repositories/61862)
[![Crates.io](https://img.shields.io/crates/v/envious.svg)](https://crates.io/crates/envious)
[![Docs.rs](https://docs.rs/envious/badge.svg)](https://docs.rs/envious)


**`envious` allows you to deserialize your serde enabled structs from
environment variables.**

See it in action:

```rust,no_run
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum StaircaseOrientation {
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    target_temp: f32,
    automate_doors: bool,

    staircase_orientation: StaircaseOrientation,
}

let config: Config = envious::from_env(envious::Prefix::None).expect("Could not deserialize from env");
```

With the following environment variables:

```bash
EXPORT target_temp=25.0
EXPORT automate_doors=true
EXPORT staircase_orientation=Left
```

it will parse it from the environment and give you a Rust struct you can use in
your application.

> _Note:_ The environment variables **are** case sensitive! This is due to how `serde` works internally. 
> If you want your structs to use SCREAMING_SNAKE_CASE, then be sure to use the
> `#[serde(rename_all = "SCREAMING_SNAKE_CASE"]` annotation on all concerned structs.

## License

`envious` is licensed under MIT _or_ Apache 2.0, as you wish.

## Contributing

To contribute to `envious` you can:

- Open up issues with ideas, remarks, bug reports, etc...
- Fork and implement new features and send them in as pull requests
- Leave it a Star and spread the word! ;)
