#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use error::EnvDeserializationError;
use serde::de::{value::StringDeserializer, DeserializeOwned, IntoDeserializer};
use value::Value;

mod config;
mod error;
mod value;

pub use config::Config;

#[derive(Debug, PartialEq)]
struct Key {
    original: String,
    current: String,
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        &self.current
    }
}

impl Key {
    fn new(original: impl Into<String>) -> Self {
        let original = original.into();
        Self {
            current: original.clone(),
            original,
        }
    }
}

impl<'de> IntoDeserializer<'de, EnvDeserializationError> for Key {
    type Deserializer = StringDeserializer<EnvDeserializationError>;
    fn into_deserializer(self) -> Self::Deserializer {
        self.current.into_deserializer()
    }
}

/// Whether to use and strip a prefix, and if so which one
pub enum Prefix<'a> {
    /// No prefix, nothing will be stripped
    None,
    /// The given prefix will be stripped
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// enum Material {
    ///     Wood,
    ///     Plastic,
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Door {
    ///     material: Material,
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct UpstairsConfig {
    ///     doors: Vec<Door>,
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Config {
    ///     upstairs: UpstairsConfig,
    /// }
    ///
    ///# #[test]
    ///# fn parse_from_env() {
    ///#     let vars = [
    ///#         ("ENVIOUS_upstairs__doors__0__material", "Wood"),
    ///#         ("ENVIOUS_upstairs__doors__1__material", "Plastic"),
    ///#         ("ENORMUS_upstairs__doors__2__material", "Plastic"),
    ///#     ];
    ///#
    ///#     for (key, val) in vars {
    ///#         std::env::set_var(key, val);
    ///#     }
    ///
    /// let config: Config = envious::from_env(envious::Prefix::Some("ENVIOUS_")).expect("Could not read from environment");
    ///# }
    ///
    /// ```
    Some(&'a str),
}

impl<'a> From<Option<&'a str>> for Prefix<'a> {
    /// Allows to easily convert from a `Option` to a `Prefix`
    ///
    /// For easily readability it should primarily used as `Prefix::from`
    ///
    /// ```rust,no_run
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize, Debug)]
    /// # struct Config {
    /// #     foobar: f32
    /// # }
    /// let maybe_prefix = Some("ENVIOUS_");
    /// let value: Config = envious::from_env(envious::Prefix::from(maybe_prefix)).expect("Could not read from environment");
    /// ```
    fn from(value: Option<&'a str>) -> Self {
        match value {
            Some(v) => Prefix::Some(v),
            None => Prefix::None,
        }
    }
}

/// Parse a given `T: Deserialize` from environment variables.
///
/// You can control whether a given prefix should be stripped or not with [`Prefix`].
///
/// ## Example
///
/// ```rust
///
///# use serde::{Deserialize, Serialize};
///#
/// #[derive(Serialize, Deserialize, Debug)]
/// enum StaircaseOrientation {
///     Left,
///     Right,
/// }
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Config {
///     target_temp: f32,
///     automate_doors: bool,
///
///     staircase_orientation: StaircaseOrientation,
/// }
///#
///# #[test]
///# fn parse_from_env() {
///#     let vars = [
///#         ("target_temp", "25.0"),
///#         ("automate_doors", "true"),
///#         ("staircase_orientation", "Left"),
///#     ];
///#
///#     for (key, val) in vars {
///#         std::env::set_var(key, val);
///#     }
///#
/// let config: Config = envious::from_env(envious::Prefix::None).expect("Could not read from environment");
///# }
/// ```
pub fn from_env<T: DeserializeOwned>(
    prefix: Prefix<'_>,
) -> Result<T, error::EnvDeserializationError> {
    let mut config = Config::default();

    if let Prefix::Some(prefix) = prefix {
        config.with_prefix(prefix);
    }

    config.from_env()
}

/// Parse a given `T: Deserialize` from anything that can be turned into an iterator.
///
/// You can control whether a given prefix should be stripped or not with [`Prefix`].
///
/// This function is useful for static deployments or for testing
///
/// ## Example
///
/// ```rust
///
///# use serde::{Deserialize, Serialize};
///#
/// #[derive(Serialize, Deserialize, Debug)]
/// enum StaircaseOrientation {
///     Left,
///     Right,
/// }
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Config {
///     target_temp: f32,
///     automate_doors: bool,
///
///     staircase_orientation: StaircaseOrientation,
/// }
///
/// let vars = [
///     (String::from("target_temp"), String::from("25.0")),
///     (String::from("automate_doors"), String::from("true")),
///     (String::from("staircase_orientation"), String::from("Left")),
/// ];
///
/// let config: Config = envious::from_iter(vars, envious::Prefix::None).expect("Could not read from environment");
/// ```
pub fn from_iter<T: DeserializeOwned, I: IntoIterator<Item = (String, String)>>(
    iter: I,
    prefix: Prefix<'_>,
) -> Result<T, error::EnvDeserializationError> {
    let mut config = Config::default();

    if let Prefix::Some(prefix) = prefix {
        config.with_prefix(prefix);
    }

    config.from_iter(iter)
}

fn from_primitive<T: DeserializeOwned, I: Iterator<Item = (Key, String)>>(
    values: I,
) -> Result<T, error::EnvDeserializationError> {
    let deserializer = Value::from_list(values.map(|(key, val)| (key, Value::Simple(val))))?;
    T::deserialize(deserializer)
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use serde::Deserialize;

    use crate::Config;

    #[test]
    fn check_simple_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Simple {
            allowed: bool,
        }

        let expected = Simple { allowed: true };

        let actual: Simple = Config::default()
            .from_iter([(String::from("allowed"), "true")].into_iter())
            .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn check_double_nested_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct InnerExtraConfig {
            allowed: bool,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct InnerConfig {
            smoothness: f32,
            extra: InnerExtraConfig,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct Nested {
            temp: u64,
            inner: InnerConfig,
        }

        let expected = Nested {
            temp: 15,
            inner: InnerConfig {
                smoothness: 32.0,
                extra: InnerExtraConfig { allowed: false },
            },
        };

        let actual: Nested = Config::default()
            .from_iter(
                [
                    ("temp", "15"),
                    ("inner__smoothness", "32.0"),
                    ("inner__extra__allowed", "false"),
                ]
                .into_iter(),
            )
            .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn check_renamed_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
        struct Simple {
            allowed_simply: bool,
        }

        let expected = Simple {
            allowed_simply: true,
        };

        let actual: Simple = Config::default()
            .from_iter([("ALLOWED-SIMPLY", String::from("true"))].into_iter())
            .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn check_simple_enum() {
        #[derive(Debug, PartialEq, Deserialize)]
        enum Simple {
            Yes,
            No,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct SimpleEnum {
            simple: Simple,
        }

        let expected = SimpleEnum { simple: Simple::No };

        let actual: SimpleEnum = Config::default()
            .from_iter([("simple", Cow::Borrowed("No"))].into_iter())
            .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn check_complex_enum() {
        #[derive(Debug, PartialEq, Deserialize)]
        enum Complex {
            Access { password: String, foo: f32 },
            No,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct ComplexEnum {
            complex: Complex,
        }

        let expected = ComplexEnum {
            complex: Complex::Access {
                password: String::from("hunter2"),
                foo: 42.0,
            },
        };

        let actual: ComplexEnum = Config::default()
            .from_iter(
                [
                    ("complex__Access__password", "hunter2"),
                    ("complex__Access__foo", "42.0"),
                ]
                .into_iter(),
            )
            .unwrap();

        assert_eq!(actual, expected);
    }
}
