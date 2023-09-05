use std::collections::BTreeMap;

use serde::de::value::{MapAccessDeserializer, MapDeserializer, SeqDeserializer};
use serde::de::IntoDeserializer;
use serde::Deserializer;

use crate::error::EnvDeserializationError;
use crate::Config;

#[derive(Debug, PartialEq)]
pub(crate) enum Value {
    Simple(String),
    Map(Vec<(String, Value)>),
}

pub(crate) struct Parser<'a> {
    pub(crate) config: &'a Config<'a>,
    pub(crate) current: Value,
}

impl Value {
    pub(crate) fn insert_at(
        &mut self,
        path: &[&str],
        value: Self,
    ) -> Result<(), EnvDeserializationError> {
        match self {
            Self::Simple(_) => Err(EnvDeserializationError::InvalidEnvNesting(
                path.iter().map(|s| s.to_string()).collect(),
            )),
            Self::Map(values) => {
                let val =
                    if let Some((_key, val)) = values.iter_mut().find(|(key, _)| key == path[0]) {
                        match val {
                            Self::Simple(_) => {
                                return Err(EnvDeserializationError::InvalidEnvNesting(
                                    path.iter().map(|s| s.to_string()).collect(),
                                ))
                            }
                            Self::Map(_) => val,
                        }
                    } else {
                        let val = Self::Map(vec![]);
                        values.push((String::from(path[0]), val));
                        &mut values.last_mut().unwrap().1
                    };

                let path = &path[1..];

                if path.len() > 1 {
                    val.insert_at(path, value)
                } else {
                    match val {
                        Self::Simple(_) => {
                            return Err(EnvDeserializationError::InvalidEnvNesting(
                                path.iter().map(|s| s.to_string()).collect(),
                            ));
                        }
                        Self::Map(values) => {
                            values.push((String::from(path[0]), value));
                        }
                    }
                    Ok(())
                }
            }
        }
    }
}

macro_rules! forward_to_deserializer {
    ($($ty:ident => $method:ident),* $(,)?) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: serde::de::Visitor<'de>
            {
                match self.current {
                    Value::Simple(val) => {
                        match val.parse::<$ty>() {
                            Ok(val) => val.into_deserializer().$method(visitor),
                            Err(e) => Err(crate::error::EnvDeserializationError::GenericDeserialization(format!("'{}' could not be deserialized due to: {}", val, e))),
                        }
                    }
                    Value::Map(_) => Err(crate::error::EnvDeserializationError::InvalidNestedValues)
                }
            }
        )*
    };
}

impl<'de> IntoDeserializer<'de, EnvDeserializationError> for Parser<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> Deserializer<'de> for Parser<'de> {
    type Error = EnvDeserializationError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current {
            Value::Simple(val) => val.into_deserializer().deserialize_any(visitor),
            Value::Map(_) => self.deserialize_map(visitor),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current {
            Value::Simple(_) => {
                SeqDeserializer::new(std::iter::once(self)).deserialize_seq(visitor)
            }
            Value::Map(values) => {
                // Convert the key into a two part sorting token and store them in an ordered data structure:
                // 1. An optional numeric prefix
                // 2. A (potentially empty) string suffix
                let values: BTreeMap<_, _> = values
                    .into_iter()
                    .map(|(key, value)| {
                        let mut chars = key.chars().peekable();

                        let mut num = String::new();

                        while chars.peek().map(char::is_ascii_digit).unwrap_or(false) {
                            num.push(chars.next().unwrap());
                        }

                        // This will only be `None` if `num` is the empty string, as we ensured all its contents are ascii digits.
                        let num = num.parse::<usize>().ok();

                        let rest = chars.collect::<String>();

                        (
                            (num, rest),
                            Parser {
                                current: value,
                                config: self.config,
                            },
                        )
                    })
                    .collect();

                SeqDeserializer::new(values.into_values()).deserialize_seq(visitor)
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current {
            Value::Simple(val) => visitor.visit_enum(val.into_deserializer()),
            Value::Map(values) => {
                // Coerce variants into correct casing if requested
                let values = self.config.maybe_coerce_case(values, variants);

                visitor.visit_enum(MapAccessDeserializer::new(MapDeserializer::new(
                    values.map(|(k, v)| {
                        (
                            k,
                            Self {
                                current: v,
                                config: self.config,
                            },
                        )
                    }),
                )))
            }
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current {
            Value::Simple(_) => Err(EnvDeserializationError::UnsupportedValue),
            Value::Map(values) => {
                visitor.visit_map(MapDeserializer::new(values.into_iter().map(|(k, v)| {
                    (
                        k,
                        Self {
                            current: v,
                            config: self.config,
                        },
                    )
                })))
            }
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let parser = match self.current {
            Value::Simple(_) => self,
            Value::Map(values) => {
                // Coerce variants into correct casing if requested
                let values = self.config.maybe_coerce_case(values, fields);
                Self {
                    config: self.config,
                    current: Value::Map(values.collect()),
                }
            }
        };

        parser.deserialize_map(visitor)
    }

    forward_to_deserializer! {
        u8 => deserialize_u8,
        i8 => deserialize_i8,
        u16 => deserialize_u16,
        i16 => deserialize_i16,
        u32 => deserialize_u32,
        i32 => deserialize_i32,
        u64 => deserialize_u64,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
        bool => deserialize_bool,
    }

    serde::forward_to_deserialize_any! {
        char str string bytes byte_buf unit unit_struct tuple_struct
        identifier tuple ignored_any
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Deserialize;

    use super::{Config, EnvDeserializationError, Parser, Value};

    static CONFIG: Config = Config::new();

    impl Value {
        pub(crate) fn simple(s: impl Into<String>) -> Self {
            Self::Simple(s.into())
        }
    }

    impl Parser<'static> {
        fn simple(s: impl Into<String>) -> Self {
            Self {
                config: &CONFIG,
                current: Value::simple(s),
            }
        }
    }

    impl From<Value> for Parser<'static> {
        fn from(value: Value) -> Self {
            Self {
                config: &CONFIG,
                current: value,
            }
        }
    }

    #[test]
    fn simple_values() {
        assert_eq!(
            Result::<_, EnvDeserializationError>::Ok(true),
            <_>::deserialize(Parser::simple("true"))
        );

        assert_eq!(Ok(25u32), <_>::deserialize(Parser::simple("25")));
        assert_eq!(
            Ok(String::from("foobar")),
            <_>::deserialize(Parser::simple("foobar"))
        );
        assert_eq!(
            Ok(Some(String::from("foobar"))),
            <_>::deserialize(Parser::simple("foobar"))
        );
    }

    #[test]
    fn simple_sequence() {
        assert_eq!(
            Result::<_, EnvDeserializationError>::Ok(vec![125u32]),
            <_>::deserialize(Parser::simple("125"))
        );
        assert_eq!(
            Ok(vec![125u32, 200, 300]),
            <_>::deserialize(Parser::from(Value::Map(vec![
                (String::from(""), Value::simple("125")),
                (String::from(""), Value::simple("200")),
                (String::from(""), Value::simple("300"))
            ])))
        );
    }

    #[test]
    fn sorted_sequence() {
        assert_eq!(
            Result::<_, EnvDeserializationError>::Ok(vec![125u32]),
            <_>::deserialize(Parser::simple("125"))
        );
        assert_eq!(
            Ok(vec![200u32, 125, 300]),
            <_>::deserialize(Parser::from(Value::Map(vec![
                (String::from("1"), Value::simple("125")),
                (String::from("0"), Value::simple("200")),
                (String::from("4"), Value::simple("300"))
            ])))
        );
    }

    #[test]
    fn simple_map() {
        assert_eq!(
            Result::<_, EnvDeserializationError>::Ok(HashMap::from([(String::from("foo"), 123)])),
            <_>::deserialize(Parser::from(Value::Map(vec![(
                String::from("foo"),
                Value::simple("123")
            ),])))
        );

        assert_eq!(
            Result::<_, EnvDeserializationError>::Ok(HashMap::from([(
                String::from("foo"),
                HashMap::from([(String::from("bar"), 123)]),
            )])),
            <_>::deserialize(Parser::from(Value::Map(vec![(
                String::from("foo"),
                Value::Map(vec![(String::from("bar"), Value::simple("123")),])
            ),])))
        );
    }
}
