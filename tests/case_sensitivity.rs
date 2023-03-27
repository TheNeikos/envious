#![allow(non_snake_case, non_camel_case_types)]

use envious::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Root {
    field1: usize,
    FIELD2: Variants,
    FiElD3: Leaf,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum Variants {
    Empty,
    ALSO_EMPTY,
    NoTeMpTy(usize),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Leaf {
    field1: usize,
    FIELD2: Variants,
    FiElD3: usize,
}

#[test]
fn parse_from_env() {
    let mut config = Config::new();
    let expected = Root {
        field1: 1,
        FIELD2: Variants::ALSO_EMPTY,
        FiElD3: Leaf {
            field1: 2,
            FIELD2: Variants::NoTeMpTy(3),
            FiElD3: 4,
        },
    };

    let vars = [
        ("field1", "1"),
        ("FIELD2", "ALSO_EMPTY"),
        ("FiElD3__field1", "2"),
        ("FiElD3__FIELD2__NoTeMpTy", "3"),
        ("FiElD3__FiElD3", "4"),
    ];

    // With case sensitivity, this should succeed
    config.case_sensitive(true);
    let root: Root = config.build_from_iter(vars).unwrap();
    assert_eq!(root, expected);

    // Also without case sensitivity
    config.case_sensitive(false);
    let root: Root = config.build_from_iter(vars).unwrap();
    assert_eq!(root, expected);

    // Now make everything uppercase, as might be expected for envrionment variables
    let vars = [
        ("FIELD1", "1"),
        ("FIELD2", "ALSO_EMPTY"),
        ("FIELD3__FIELD1", "2"),
        ("FIELD3__FIELD2__NOTEMPTY", "3"),
        ("FIELD3__FIELD3", "4"),
    ];

    // This should work when case insensitive
    let root: Root = config.build_from_iter(vars).unwrap();
    assert_eq!(root, expected);

    // But fail when we want case sensitivity again
    config.case_sensitive(true);
    let result: Result<Root, _> = config.build_from_iter(vars);
    result.unwrap_err();

    // Now make everything lowercase
    let vars = [
        ("field1", "1"),
        ("field2", "ALSO_EMPTY"),
        ("field3__field1", "2"),
        ("field3__field2__notempty", "3"),
        ("field3__field3", "4"),
    ];

    // Fails when case sensitive
    let result: Result<Root, _> = config.build_from_iter(vars);
    result.unwrap_err();

    // Works when case insensitive
    config.case_sensitive(false);
    let root: Root = config.build_from_iter(vars).unwrap();
    assert_eq!(root, expected);
}
