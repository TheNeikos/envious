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
        ("target_temp", Some("25.0")),
        ("automate_doors", Some("true")),
        ("staircase_orientation", Some("Left")),
    ];

    let config: Config = temp_env::with_vars(vars, || envious::Config::new().from_env().unwrap());
    println!("{:#?}", config);
}
