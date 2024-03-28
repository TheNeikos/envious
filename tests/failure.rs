#![allow(dead_code)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Simple {
    test: bool,
}

#[test]
fn wrongly_nested_fields() {
    let vars = [("test", ("true")), ("test__bar", ("true"))];

    let config: Result<Simple, _> = envious::Config::new().build_from_iter(vars);

    println!("{:?}", config.unwrap_err());
}

#[test]
fn wrongly_nested_prefixed_fields() {
    let vars = [("PRE_test", ("true")), ("PRE_test__bar", ("true"))];

    let config: Result<Simple, _> = envious::Config::new().build_from_iter(vars);

    println!("{:?}", config.unwrap_err());
}
