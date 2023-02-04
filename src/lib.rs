#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use error::EnvDeserializationError;
use serde::de::{value::StringDeserializer, DeserializeOwned, IntoDeserializer};
use value::Value;

mod error;
mod value;

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
    fn new(original: String) -> Self {
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
    let env_values = std::env::vars();

    from_primitive(env_values.flat_map(|(key, value)| {
        if let Prefix::Some(prefix) = prefix {
            let stripped_key = key.strip_prefix(prefix).map(|s| s.to_string())?;
            Some((Key::new(stripped_key), value))
        } else {
            Some((Key::new(key), value))
        }
    }))
}

fn from_primitive<T: DeserializeOwned, I: Iterator<Item = (Key, String)>>(
    values: I,
) -> Result<T, error::EnvDeserializationError> {
    let deserializer =
        Value::from_list(values.map(|(key, val)| (key, Value::Simple(val))).collect())?;
    T::deserialize(deserializer)
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use crate::{from_primitive, Key};

    #[test]
    fn check_simple_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Simple {
            allowed: bool,
        }

        let expected = Simple { allowed: true };

        let actual: Simple =
            from_primitive([(Key::new(String::from("allowed")), String::from("true"))].into_iter())
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

        let actual: Nested = from_primitive(
            [
                (Key::new(String::from("temp")), String::from("15")),
                (
                    Key::new(String::from("inner__smoothness")),
                    String::from("32.0"),
                ),
                (
                    Key::new(String::from("inner__extra__allowed")),
                    String::from("false"),
                ),
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

        let actual: Simple = from_primitive(
            [(
                Key::new(String::from("ALLOWED-SIMPLY")),
                String::from("true"),
            )]
            .into_iter(),
        )
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

        let actual: SimpleEnum =
            from_primitive([(Key::new(String::from("simple")), String::from("No"))].into_iter())
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

        let actual: ComplexEnum = from_primitive(
            [
                (
                    Key::new(String::from("complex__Access__password")),
                    String::from("hunter2"),
                ),
                (
                    Key::new(String::from("complex__Access__foo")),
                    String::from("42.0"),
                ),
            ]
            .into_iter(),
        )
        .unwrap();

        assert_eq!(actual, expected);
    }
}
