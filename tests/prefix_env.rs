#![allow(dead_code)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
enum Material {
    Wood,
    Plastic,
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
        ("ENVIOUS_upstairs__doors__0__material", Some("Wood")),
        ("ENVIOUS_upstairs__doors__1__material", Some("Plastic")),
        ("ENORMUS_upstairs__doors__2__material", Some("Plastic")),
    ];

    let config: Config = temp_env::with_vars(vars, || {
        envious::Config::new()
            .with_prefix("ENVIOUS_")
            .build_from_env()
            .unwrap()
    });

    println!("{:#?}", config);
}
