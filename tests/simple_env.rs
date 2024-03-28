#![allow(dead_code)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
enum StaircaseOrientation {
    Left,
    Right,
}

#[derive(Deserialize, Debug)]
struct Config {
    target_temp: f32,
    automate_doors: bool,

    staircase_orientation: StaircaseOrientation,
}

#[test]
fn parse_from_env() {
    let vars = [
        ("target_temp", ("25.0")),
        ("automate_doors", ("true")),
        ("staircase_orientation", ("Left")),
    ];

    let config: Config = envious::Config::new().build_from_iter(vars).unwrap();
    println!("{:#?}", config);
}
