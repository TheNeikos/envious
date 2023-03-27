#![allow(dead_code)]
use envious::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Simple {
    test: bool,
}

#[test]
fn wrongly_nested_fields() {
    let vars = [("test", Some("true")), ("test__bar", Some("true"))];

    let config: Result<Simple, _> = temp_env::with_vars(vars, || Config::new().build_from_env());

    println!("{:?}", config.unwrap_err());
}

#[test]
fn wrongly_nested_prefixed_fields() {
    let vars = [("PRE_test", Some("true")), ("PRE_test__bar", Some("true"))];

    let config: Result<Simple, _> =
        temp_env::with_vars(vars, || Config::new().with_prefix("PRE_").build_from_env());

    println!("{:?}", config.unwrap_err());
}
