use crate::Result;
use clap::{value_t, Arg};
use simple_error::bail;
use slog::Drain;
use std::{os::unix::io::AsRawFd, path::Path};

pub mod config;
use config::*;
mod ascii_art;
pub mod process_csv;
pub mod serve;

pub async fn exec(logger: slog::Logger, matches: &clap::ArgMatches) -> Result<()> {
    let config_path = if let Ok(config_path) = value_t!(matches, "config", String) {
        if !Path::new(&config_path).exists() {
            bail!("config file {} not found", config_path)
        }
        Some(config_path)
    } else {
        const DEFAULT_CONFIG_FILE: &str = "config.yaml";
        if Path::new(DEFAULT_CONFIG_FILE).exists() {
            Some(String::from(DEFAULT_CONFIG_FILE))
        } else {
            None
        }
    };

    let mut config = if let Some(config_path) = config_path {
        let config_yaml = std::fs::read_to_string(config_path)?;
        serde_yaml::from_str(&config_yaml)?
    } else {
        Config::default()
    };

    config.load_from_env("PS_").await?;

    match matches.subcommand() {
        Some((serve::CMD_NAME, sub_match)) => serve::run(logger, config, sub_match).await,
        None => process_csv::run(logger, config, matches).await,
        Some(_) => unreachable!("match arms should cover all the possible cases"),
    }
}

pub async fn set_up_logger_and_exec() -> i32 {
    let matches = clap::Command::new("payment-engine")
        .about("This is a toy payment engine.")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::new("debug").short('d').long("debug").help("makes the logs more verbose with debug"))
        .arg(Arg::new("verbose").short('v').long("verbose").help("makes the logs more verbose with infor"))
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("read configuration from this file")
                .takes_value(true),
        )
        .arg(process_csv::arg())
        .subcommand(serve::cmd())
        .get_matches();

    let stderr = std::io::stderr();
    let drain: Box<dyn Drain<Ok = (), Err = std::io::Error> + Send> = match termios::Termios::from_fd(stderr.as_raw_fd() as _) {
        Ok(_) => {
            let decorator = slog_term::TermDecorator::new().build();
            Box::new(slog_term::FullFormat::new(decorator).build())
        }
        Err(_) => Box::new(slog_json::Json::default(stderr)),
    };

    let drain = slog_async::Async::new(drain.fuse())
        .build()
        .filter_level(if matches.is_present("debug") {
            slog::Level::Debug
        } else if matches.is_present("verbose") {
            slog::Level::Info
        } else {
            slog::Level::Error
        })
        .fuse();

    let logger = slog::Logger::root(drain, o!());

    if let Err(e) = exec(logger.clone(), &matches).await {
        error!(logger, "{}", e);
        return 1;
    } else {
        return 0;
    }
}
