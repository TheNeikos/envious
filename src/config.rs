use serde::de::DeserializeOwned;

use crate::{error, Value, Parser, error::EnvDeserializationError};

/// Temp
#[derive(Debug)]
pub struct Config<'a> {
    prefix: Option<&'a str>,
    case_sensitive: bool,
    separator: &'a str,
}

impl<'a> Config<'a> {
    /// Create a new instance of [`Config`] with basic values.
    pub const fn new() -> Self {
        Self {
            prefix: None,
            case_sensitive: false,
            separator: "__",
        }
    }

    /// Temp
    pub fn with_separator(&mut self, separator: &'a str) -> &mut Self {
        self.separator = separator;
        self
    }

    /// Temp
    pub fn with_prefix(&mut self, prefix: &'a str) -> &mut Self {
        self.prefix = Some(prefix);
        self
    }

    /// Temp
    pub fn case_sensitivity(&mut self, case_sensitive: bool) -> &mut Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Temp
    pub fn from_env<T: DeserializeOwned>(&mut self) -> Result<T, error::EnvDeserializationError> {
        let env_values = std::env::vars();
        self.from_iter(env_values)
    }

    /// Temp
    pub fn from_iter<T, K, V, I>(&self, iter: I) -> Result<T, error::EnvDeserializationError>
    where
        T: DeserializeOwned,
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        let values = iter.into_iter().map(|(k, v)| (k.into(), v.into()));

        let values = values.flat_map(|(key, value)| {
            let value = Value::Simple(value);
            if let Some(prefix) = self.prefix {
                let stripped_key = key.strip_prefix(prefix)?.to_owned();
                Some((stripped_key, value))
            } else {
                Some((key, value))
            }
        });

        let parser = self.create_parser(values)?;

        T::deserialize(parser)
    }

    /// Temp
    fn create_parser<I>(&self, iter: I) -> Result<Parser, EnvDeserializationError> where I: IntoIterator<Item = (String, Value)> {
        let mut base = Value::Map(vec![]);

        for (key, value) in iter.into_iter() {
            let path = key.split(self.separator).collect::<Vec<_>>();

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

        Ok(Parser { config: &self, current: base})
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