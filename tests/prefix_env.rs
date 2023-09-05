#![allow(dead_code)]
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
enum Material {
    Wood,
    Plastic,
    Glass,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct Door {
    material: Material,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct UpstairsConfig {
    doors: Vec<Door>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct Config {
    upstairs: UpstairsConfig,
}

#[test]
fn parse_from_env() {
    let vars = [
        ("ENVIOUS_upstairs__doors__0__material", Some("Wood")),
        ("ENVIOUS_upstairs__doors__2__material", Some("Glass")),
        ("ENVIOUS_upstairs__doors__1__material", Some("Plastic")),
    ];

    let config: Config = temp_env::with_vars(vars, || {
        envious::Config::new()
            .with_prefix("ENVIOUS_")
            .build_from_env()
            .unwrap()
    });

    assert_eq!(
        config,
        Config {
            upstairs: UpstairsConfig {
                doors: vec![
                    Door {
                        material: Material::Wood
                    },
                    Door {
                        material: Material::Plastic
                    },
                    Door {
                        material: Material::Glass
                    }
                ]
            }
        }
    );

    // When case insensitive, the same test should succeed with a lowercase prefix.
    let config: Config = temp_env::with_vars(vars, || {
        envious::Config::new()
            .case_sensitive(false)
            .with_prefix("envious_")
            .build_from_env()
            .unwrap()
    });

    println!("{:#?}", config);

    // However when case sensitive, it will fail.
    let result: Result<Config, _> = temp_env::with_vars(vars, || {
        envious::Config::new()
            .case_sensitive(true)
            .with_prefix("envious_")
            .build_from_env()
    });
    let err = result.unwrap_err();

    println!("{:#?}", err);
}
