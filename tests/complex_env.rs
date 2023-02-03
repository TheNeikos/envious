use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
enum Material {
    Wood { kind: String },
    Plastic(f32),
    Unknown,
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
        ("upstairs__doors__0__material__Wood__kind", "Mahagony"),
        ("upstairs__doors__1__material__Plastic", "25"),
        ("upstairs__doors__foo__material", "Unknown"),
    ];

    for (key, val) in vars {
        std::env::set_var(key, val);
    }

    let config: Config = envious::from_env(envious::Prefix::None).unwrap();
    println!("{:#?}", config);
}
