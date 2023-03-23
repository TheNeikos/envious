use serde::de::DeserializeOwned;

use crate::{error, Key};

/// Temp
#[derive(Debug, Default)]
pub struct Config<'a> {
    prefix: Option<&'a str>,
    case_sensitive: bool,
}

impl<'a> Config<'a> {
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
    pub fn from_iter<T, K, V, I>(&mut self, iter: I) -> Result<T, error::EnvDeserializationError>
    where
        T: DeserializeOwned,
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        let values = iter.into_iter().map(|(k, v)| (k.into(), v.into()));
        super::from_primitive(values.flat_map(|(key, value)| {
            if let Some(prefix) = self.prefix {
                let stripped_key = key.strip_prefix(prefix)?.to_owned();
                Some((Key::new(stripped_key), value))
            } else {
                Some((Key::new(key), value))
            }
        }))
    }
}
