
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

`envious` also supports the ability to only take in prefixed environment
variables by passing in a `envious::Prefix::Some("<prefix string>")`. This will
strip it before processing them further.

## Getting Started

To use `envious` simply add it to your `Cargo.toml` with:

```bash
cargo add envious
```

and deserialize from your environment with `envious::from_env`!


⚠️ **Current Shortcomings**

- Tuple Enum Variants can currently _not_ be longer than one element!
- Ordering of arrays is highly sensitive to environment order
    - No ordering is currently done, and the ordering depends on how the
      operating system propagates variables



## How deserialization works

The mapping between environment variables and the serde model is as follows:

#### Nested fields are seperated by `__` in their names

For example, if you have the following struct:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Radiator {
    min_temp: f32,
    max_temp: f32,
}

#[derive(Serialize, Deserialize)]
struct Config {
    target_temp: f32,
    radiator: Option<Radiator>
}
```

You can deserialize `Config` with the following variables:

```bash
export target_temp=21.0
export radiator__min_temp=15.0
export radiator__max_temp=30.0
```

#### Arrays are serialized using nested fields with the individual keys being discarded

Arrays are represented as anonymous structs, with the 'fields' being the individual elements.

A more complex example could look like this:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Material {
    Wood,
    Plastic,
}

#[derive(Serialize, Deserialize)]
struct Door {
    width: f32,
    height: f32,
    material: Material,
}

#[derive(Serialize, Deserialize)]
struct House {
    age: u32,
    entrance_doors: Vec<Door>,
}
```

Now, to deserialize a `House` we can set the following variables:

```bash
export age=120
export entrance_doors__0__width=100
export entrance_doors__0__height=100
export entrance_doors__0__material="Wood"
export entrance_doors__1__width=200
export entrance_doors__1__height=120
export entrance_doors__1__material="Plastic"
export entrance_doors__foo__width=400
export entrance_doors__foo__height=20
export entrance_doors__foo__material="Plastic"
```

As you can see, the individual 'keys' of the array do not matter! The same key refers to the same object though.

#### Unit enums variants (without fields), are serialized from strings

As you can see in the example above, the `Material` enum gets simply deserialized from the name of the variant. __Be careful about upper/lower case__ Serde expects per-default that the case is _exactly_ the same!

#### Complex enum variants are serialzed just like structs

Per default `serde` uses external tagging for more complicated enum variants.
Tuple enums are currently only supported with a single value.

To see what this means, lets take this enum as an example:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Shape {
    Rectangle { width: f32, height: f32 },
    Circle(f32),
    Nothing,
}

#[derive(Serialize, Deserialize)]
struct Config {
    expected_shape: Shape,
}
```

To deserialize `Config` here, we can use the following variables:

```bash
export expected_shape__Rectangle__width=50.0
export expected_shape__Rectangle__height=10.0

// OR

export expected_shape__Circle=15.0

// OR

export expected_shape=Nothing
```

Any of these sets of variables would give you the expected outcome.

__Should you change the tagging of your struct, be sure to adapt the given variables.__


## License

`envious` is licensed under MIT _or_ Apache 2.0, as you wish.

## Contributing

To contribute to `envious` you can:

- Open up issues with ideas, remarks, bug reports, etc...
- Fork and implement new features and send them in as pull requests
- Leave it a Star and spread the word! ;)
