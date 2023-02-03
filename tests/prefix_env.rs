use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum Material {
    Wood,
    Plastic,
}

#[derive(Serialize, Deserialize, Debug)]
struct Door {
    material: Material,
}

#[derive(Serialize, Deserialize, Debug)]
struct UpstairsConfig {
    doors: Vec<Door>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    upstairs: UpstairsConfig,
}

#[test]
fn parse_from_env() {
    let vars = [
        ("ENVIOUS_upstairs__doors__0__material", "Wood"),
        ("ENVIOUS_upstairs__doors__1__material", "Plastic"),
        ("ENORMUS_upstairs__doors__2__material", "Plastic"),
    ];

    for (key, val) in vars {
        std::env::set_var(key, val);
    }

    let config: Config = envious::from_env(envious::Prefix::Some("ENVIOUS_")).unwrap();
    println!("{:#?}", config);
}
