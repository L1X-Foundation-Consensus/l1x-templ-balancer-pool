use clap::{App, Arg, SubCommand};
use serde_json::{from_str, Value};

#[derive(Debug)]
enum ConfigError {
    FileNotFound,
    InvalidJson,
}

impl ConfigError {
    fn from_io_error(err: std::io::Error) -> Self {
        log::error!("from_io_error :: {:?}", err);
        Self::FileNotFound
    }
}

fn read_config_file(config_file_path: &str) -> Result<Value, ConfigError> {
    let config_file_contents = std::fs::read_to_string(config_file_path)
        .map_err(ConfigError::from_io_error)?;

    let config_value = from_str(&config_file_contents).map_err(|err| {
        log::error!("JSON Parse Error :: {:?}", err);
        ConfigError::InvalidJson
    })?;

    Ok(config_value)
}

fn set_config_value(
    config: &mut Value,
    key: &str,
    value: &str,
) -> Result<(), ConfigError> {
    config[key] = Value::String(value.to_string());
    Ok(())
}

fn get_config_value(config: &Value, key: &str) -> Result<String, ConfigError> {
    let config_value = config.get(key).ok_or(ConfigError::InvalidJson)?;
    match config_value {
        Value::String(value) => Ok(value.to_string()),
        _ => Err(ConfigError::InvalidJson),
    }
}

fn main() -> Result<(), ConfigError> {
    tracing_subscriber::fmt::init();
    let matches = App::new("l1x-bpool-conf")
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("cnf-key").required(true))
                .arg(Arg::with_name("cnf-value").required(true)),
        )
        .subcommand(
            SubCommand::with_name("get")
                .arg(Arg::with_name("cnf-key").required(true)),
        )
        .get_matches();

    let cfg_ws_home = std::env::var("L1X_CFG_WS_HOME")
        .expect("The L1X_CFG_WS_HOME environment variable must be set");

    let config_file_path = format!(
        "{}/{}",
        cfg_ws_home,
        "l1x-eth-contracts/Balancer-v2/pkg/pool-stable/deploy/input.json"
    );

    match matches.subcommand() {
        ("set", Some(set_matches)) => {
            let key = set_matches.value_of("cnf-key").unwrap();
            let value = set_matches.value_of("cnf-value").unwrap();

            let mut config = read_config_file(&config_file_path)
                .expect("Failed to read config file");

            set_config_value(&mut config, key, value)
                .expect("Failed to set config value");

            std::fs::write(
                &config_file_path,
                serde_json::to_string_pretty(&config).unwrap(),
            )
            .expect("Failed to write config file");

            log::info!(
                "Bpool Deploy Config File :: `{}` got updated with ...",
                config_file_path
            );
            log::info!("Cnf-Key {}", key);
            log::info!("Cnf-Value {}", value);
        }
        ("get", Some(get_matches)) => {
            let key = get_matches.value_of("cnf-key").unwrap();

            let config = read_config_file(&config_file_path)
                .expect("Failed to read config file");

            let value = get_config_value(&config, key)
                .expect("Failed to get config value");

            log::info!("{:#?}", value);
        }
        _ => {
            log::error!("Unknown command");
        }
    }

    Ok(())
}
