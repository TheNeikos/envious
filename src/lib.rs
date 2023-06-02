#![doc = include_str!("../README.md")]
#![deny(missing_docs, unreachable_pub)]

use value::Value;

mod config;
mod error;
mod value;

pub use config::Config;
pub use error::EnvDeserializationError;

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

        let actual: Simple = Config::new()
            .build_from_iter([(String::from("allowed"), "true")].into_iter())
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

        let actual: Nested = Config::new()
            .build_from_iter(
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

        let actual: Simple = Config::new()
            .build_from_iter([("ALLOWED-SIMPLY", String::from("true"))].into_iter())
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

        let actual: SimpleEnum = Config::new()
            .build_from_iter([("simple", Cow::Borrowed("No"))].into_iter())
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

        let actual: ComplexEnum = Config::new()
            .build_from_iter(
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
