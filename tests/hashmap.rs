#![allow(dead_code)]
use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Outer {
    values: HashMap<String, String>,
    inner: Inner,
}

#[derive(Debug, Deserialize)]
struct Inner {
    more_values: HashMap<String, String>,
}

#[test]
fn parse_hashmap() {
    let vars = [
        ("values__0key", ("first value")),
        ("values__1val", ("second value")),
        ("inner__more_values__0key", ("first inner value")),
    ];

    let config: Outer = envious::Config::new().build_from_iter(vars).unwrap();
    println!("{:#?}", config);
}
