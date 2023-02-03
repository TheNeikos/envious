use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum StaircaseOrientation {
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    target_temp: f32,
    automate_doors: bool,

    staircase_orientation: StaircaseOrientation,
}

#[test]
fn parse_from_env() {
    let vars = [
        ("target_temp", "25.0"),
        ("automate_doors", "true"),
        ("staircase_orientation", "Left"),
    ];

    for (key, val) in vars {
        std::env::set_var(key, val);
    }

    let config: Config = envious::from_env(envious::Prefix::None).unwrap();
    println!("{:#?}", config);
}
