use std::{borrow::Cow, ops::Not};

use serde::de::DeserializeOwned;

use crate::{error, error::EnvDeserializationError, Parser, Value};

/// Used to configure the behaviour of the environment variable deserialization.
///
/// For information on default behaviours see [`Self::new`].
/// For details on usage see [`Self::from_env`] and [`Self::from_iter`].
#[derive(Debug, Clone)]
#[must_use]
pub struct Config<'a> {
    prefix: Option<Cow<'a, str>>,
    case_sensitive: bool,
    separator: Cow<'a, str>,
}

impl Default for Config<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Config<'a> {
    /// Create a new instance of [`Config`] with basic values. i.e.
    /// - No prefix
    /// - Case insensitive
    /// - A separator of "__" (double underscore)
    pub const fn new() -> Self {
        Self {
            prefix: None,
            case_sensitive: false,
            separator: Cow::Borrowed("__"),
        }
    }

    /// Configures the separator used when parsing the environment variable names.
    ///
    /// Defaults to `__` (double underscore)
    ///
    /// E.g. with an environment variable named `env_variable`, and the default separator,
    /// a field with the name `env_variable` is looked for, whereas with a custom separator of
    /// `_`, a field with the name `env` is looked for, where that field is a struct which in turn has a field
    /// `variable`.
    pub fn with_separator<S>(&mut self, separator: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.separator = separator.into();
        self
    }

    /// Configures the prefix to strip from environment variables names.
    ///
    /// Environments variables without the prefix are discarded.
    ///
    /// Defaults to no prefix being set. The default can be returned to via [`Self::without_prefix`].
    pub fn with_prefix<S>(&mut self, prefix: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.prefix = Some(prefix.into());
        self
    }

    /// Resets the [`Config`] to not look for a specific prefix in environment variables names.
    ///
    /// Used to remove the effect of [`Self::with_prefix`].
    pub fn without_prefix(&mut self) -> &mut Self {
        self.prefix = None;
        self
    }

    /// Configured whether the parsing of environment variables names is case sensitive or not.
    ///
    /// Defaults to case insensitive.
    ///
    /// NB: Only `struct` fields and `enum` variants are affected by case sensitivity.
    pub fn case_sensitive(&mut self, case_sensitive: bool) -> &mut Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Parse a given `T: Deserialize` from environment variables.
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
    /// let config: Config = envious::Config::new().from_env().unwrap();
    ///# }
    /// ```
    pub fn from_env<T: DeserializeOwned>(&self) -> Result<T, error::EnvDeserializationError> {
        let env_values = std::env::vars();
        self.from_iter(env_values)
    }

    /// Parse a given `T: Deserialize` from anything that can be turned into an iterator of key value tuples.
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
    ///     ("target_temp", "25.0"),
    ///     ("automate_doors", "true"),
    ///     ("staircase_orientation", "Left"),
    /// ];
    ///
    /// let config: Config = envious::Config::new().from_iter(vars).unwrap();
    /// ```
    pub fn from_iter<T, K, V, I>(&self, iter: I) -> Result<T, error::EnvDeserializationError>
    where
        T: DeserializeOwned,
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        let values = iter.into_iter().map(|(k, v)| (k.into(), v.into()));

        let values = values.filter_map(|(key, value)| {
            let value = Value::Simple(value);
            if let Some(prefix) = &self.prefix {
                let stripped_key = key.strip_prefix(prefix.as_ref())?.to_owned();
                Some((stripped_key, value))
            } else {
                Some((key, value))
            }
        });

        let parser = self.create_parser(values)?;

        T::deserialize(parser)
    }

    /// Creates a [`Parser`] from its various parts.
    fn create_parser<I>(&self, iter: I) -> Result<Parser, EnvDeserializationError>
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        let mut base = Value::Map(vec![]);

        for (key, value) in iter.into_iter() {
            let path = key.split(self.separator.as_ref()).collect::<Vec<_>>();

            if path.len() == 1 {
                if let Value::Map(base) = &mut base {
                    base.push((key, value));
                } else {
                    unreachable!()
                }
            } else {
                base.insert_at(&path, value)?;
            }
        }

        Ok(Parser {
            config: self,
            current: base,
        })
    }

    /// Given an iterator of keys and values, and a list of keys with corrected casing, converts
    /// the keys to the desired cases, thereby making the process case insensitive.
    ///
    /// NB: This uses [`str::eq_ignore_ascii_case`], and therefore has the same limitations.
    /// Namely it will not be able to handle differently cased non-ascii characters, such as ß and ẞ.
    pub(crate) fn maybe_coerce_case<I, V>(
        &self,
        values: I,
        corrected_cases: &'static [&'static str],
    ) -> impl Iterator<Item = (String, V)>
    where
        I: IntoIterator<Item = (String, V)>,
    {
        let case_sensitive = self.case_sensitive;
        values.into_iter().map(move |(key, value)| {
            if case_sensitive.not() {
                if let Some(coerced_key) = corrected_cases
                    .iter()
                    .find(|item| item.eq_ignore_ascii_case(&key))
                {
                    (coerced_key.to_string(), value)
                } else {
                    (key, value)
                }
            } else {
                (key, value)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, Value};

    #[test]
    fn convert_list_of_key_vals_to_tree() {
        let input = vec![
            (String::from("FOO"), Value::simple("bar")),
            (String::from("BAZ"), Value::simple("124")),
            (String::from("NESTED__FOO"), Value::simple("true")),
            (String::from("NESTED__BAZ"), Value::simple("Hello")),
        ];

        let expected = Value::Map(vec![
            (String::from("FOO"), Value::simple("bar")),
            (String::from("BAZ"), Value::simple("124")),
            (
                String::from("NESTED"),
                Value::Map(vec![
                    (String::from("FOO"), Value::simple("true")),
                    (String::from("BAZ"), Value::simple("Hello")),
                ]),
            ),
        ]);

        let config = Config::new();
        let actual = config.create_parser(input).unwrap();

        assert_eq!(actual.current, expected);
    }

    #[test]
    fn custom_sep() {
        let input = vec![
            (String::from("FOO"), Value::simple("bar")),
            (String::from("BAZ"), Value::simple("124")),
            (String::from("NESTED#FOO"), Value::simple("true")),
            (String::from("NESTED#BAZ"), Value::simple("Hello")),
        ];

        let expected = Value::Map(vec![
            (String::from("FOO"), Value::simple("bar")),
            (String::from("BAZ"), Value::simple("124")),
            (
                String::from("NESTED"),
                Value::Map(vec![
                    (String::from("FOO"), Value::simple("true")),
                    (String::from("BAZ"), Value::simple("Hello")),
                ]),
            ),
        ]);

        let mut config = Config::new();
        let actual = config.with_separator("#").create_parser(input).unwrap();

        assert_eq!(actual.current, expected);
    }
}
