use serde::de::value::{MapAccessDeserializer, MapDeserializer, SeqDeserializer};
use serde::de::IntoDeserializer;
use serde::Deserializer;

use crate::error::EnvDeserializationError;
use crate::Key;

#[derive(Debug, PartialEq)]
pub(crate) enum Value {
    Simple(String),
    Map(Vec<(Key, Value)>),
}

const SEPERATOR: &str = "__";

impl Value {
    fn insert_at(&mut self, path: &[&str], value: Value) -> Result<(), EnvDeserializationError> {
        match self {
            Value::Simple(_) => Err(EnvDeserializationError::InvalidEnvNesting(
                path.iter().map(|s| s.to_string()).collect(),
            )),
            Value::Map(values) => {
                let val = if let Some((_key, val)) =
                    values.iter_mut().find(|(key, _)| key.as_ref() == path[0])
                {
                    match val {
                        Value::Simple(_) => {
                            return Err(EnvDeserializationError::InvalidEnvNesting(
                                path.iter().map(|s| s.to_string()).collect(),
                            ))
                        }
                        Value::Map(_) => val,
                    }
                } else {
                    let val = Value::Map(vec![]);
                    values.push((Key::new(path[0].to_string()), val));
                    &mut values.last_mut().unwrap().1
                };

                let path = &path[1..];

                if path.len() > 1 {
                    val.insert_at(path, value)
                } else {
                    match val {
                        Value::Simple(_) => {
                            return Err(EnvDeserializationError::InvalidEnvNesting(
                                path.iter().map(|s| s.to_string()).collect(),
                            ))
                        }
                        Value::Map(values) => values.push((Key::new(path[0].to_string()), value)),
                    }
                    Ok(())
                }
            }
        }
    }

    pub(crate) fn from_list(
        list: impl IntoIterator<Item = (Key, Self)>,
    ) -> Result<Self, EnvDeserializationError> {
        let mut base = Value::Map(vec![]);

        for (key, value) in list.into_iter() {
            let path = key.original.split(SEPERATOR).collect::<Vec<_>>();

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

        Ok(base)
    }
}

macro_rules! forward_to_deserializer {
    ($($ty:ident => $method:ident),* $(,)?) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: serde::de::Visitor<'de>
            {
                match self {
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

impl<'de> IntoDeserializer<'de, EnvDeserializationError> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = crate::error::EnvDeserializationError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Simple(val) => val.into_deserializer().deserialize_any(visitor),
            Value::Map(_) => self.deserialize_map(visitor),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            val @ Value::Simple(_) => {
                SeqDeserializer::new(std::iter::once(val)).deserialize_seq(visitor)
            }
            Value::Map(values) => {
                let values = values.into_iter().map(|(_, val)| val);

                SeqDeserializer::new(values).deserialize_seq(visitor)
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
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Simple(val) => visitor.visit_enum(val.into_deserializer()),
            Value::Map(values) => visitor.visit_enum(MapAccessDeserializer::new(
                MapDeserializer::new(values.into_iter()),
            )),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Simple(_) => Err(EnvDeserializationError::UnsupportedValue),
            Value::Map(values) => visitor.visit_map(MapDeserializer::new(values.into_iter())),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
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
    use crate::Key;

    use super::Value;
    use serde::Deserialize;


    impl Value {
        /// Used to simplify the creation of a `Simple` variant of [`Value`].
        fn simple(s: impl Into<String>) -> Self {
            Self::Simple(s.into())
        }
    }

    #[test]
    fn simple_values() {
        assert_eq!(Ok(true), <_>::deserialize(Value::simple("true")));

        assert_eq!(Ok(25u32), <_>::deserialize(Value::simple("25")));
        assert_eq!(
            Ok(String::from("foobar")),
            <_>::deserialize(Value::simple("foobar"))
        );
        assert_eq!(
            Ok(Some(String::from("foobar"))),
            <_>::deserialize(Value::simple("foobar"))
        );
    }

    #[test]
    fn simple_sequence() {
        assert_eq!(Ok(vec![125u32]), <_>::deserialize(Value::simple("125")));
        assert_eq!(
            Ok(vec![125u32, 200, 300]),
            <_>::deserialize(Value::Map(vec![
                (Key::new(""), Value::simple("125")),
                (Key::new(""), Value::simple("200")),
                (Key::new(""), Value::simple("300"))
            ]))
        );
    }

    #[test]
    fn simple_map() {
        assert_eq!(
            Ok(std::collections::HashMap::from([(
                String::from("foo"),
                123
            )])),
            <_>::deserialize(Value::Map(vec![(Key::new("foo"), Value::simple("123")),]))
        );

        assert_eq!(
            Ok(std::collections::HashMap::from([(
                String::from("foo"),
                std::collections::HashMap::from([(String::from("bar"), 123)]),
            )])),
            <_>::deserialize(Value::Map(vec![(
                Key::new("foo"),
                Value::Map(vec![(Key::new("bar"), Value::simple("123")),])
            ),]))
        );
    }

    #[test]
    fn convert_list_of_key_vals_to_tree() {
        let input = vec![
            (Key::new("FOO"), Value::simple("bar")),
            (Key::new("BAZ"), Value::simple("124")),
            (Key::new("NESTED__FOO"), Value::simple("true")),
            (Key::new("NESTED__BAZ"), Value::simple("Hello")),
        ];

        let expected = Value::Map(vec![
            (Key::new("FOO"), Value::simple("bar")),
            (Key::new("BAZ"), Value::simple("124")),
            (
                Key::new("NESTED"),
                Value::Map(vec![
                    (Key::new("FOO"), Value::simple("true")),
                    (Key::new("BAZ"), Value::simple("Hello")),
                ]),
            ),
        ]);

        let actual = Value::from_list(input).unwrap();

        assert_eq!(actual, expected);
    }
}
