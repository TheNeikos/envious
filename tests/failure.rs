#![allow(dead_code)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Simple {
    test: bool,
}

#[test]
fn wrongly_nested_fields() {
    let vars = [("test", "true"), ("test__bar", "true")];

    for (key, val) in vars {
        std::env::set_var(key, val);
    }

    let config: Result<Simple, _> = envious::from_env(envious::Prefix::None);

    for (key, _) in vars {
        std::env::remove_var(key);
    }

    println!("{:?}", config.unwrap_err());
}

#[test]
fn wrongly_nested_prefixed_fields() {
    let vars = [("PRE_test", "true"), ("PRE_test__bar", "true")];

    for (key, val) in vars {
        std::env::set_var(key, val);
    }

    let config: Result<Simple, _> = envious::from_env(envious::Prefix::Some("PRE_"));

    for (key, _) in vars {
        std::env::remove_var(key);
    }

    println!("{:?}", config.unwrap_err());
}
