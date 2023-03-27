#![allow(dead_code)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
enum Material {
    Wood { kind: String },
    Plastic(f32),
    Unknown,
}

#[derive(Deserialize, Debug)]
struct Door {
    material: Material,
}

#[derive(Deserialize, Debug)]
struct UpstairsConfig {
    doors: Vec<Door>,
}

#[derive(Deserialize, Debug)]
struct Config {
    upstairs: UpstairsConfig,
}

#[test]
fn parse_from_env() {
    let vars = [
        ("upstairs__doors__0__material__Wood__kind", Some("Mahagony")),
        ("upstairs__doors__1__material__Plastic", Some("25")),
        ("upstairs__doors__foo__material", Some("Unknown")),
    ];

    let config: Config =
        temp_env::with_vars(vars, || envious::Config::new().build_from_env().unwrap());
    println!("{:#?}", config);
}
