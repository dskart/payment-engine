use crate::{api::API, app, cmd::Config, Result};
use clap::Arg;

pub const CMD_NAME: &str = "serve";

pub fn cmd<'a>() -> clap::Command<'a> {
    let port_arg = Arg::new("port")
        .long("port")
        .short('p')
        .default_value("8080")
        .takes_value(true)
        .help("the port for the http api to listen on");

    return clap::Command::new(CMD_NAME).arg(port_arg);
}

pub async fn run(logger: slog::Logger, config: Config, matches: &clap::ArgMatches) -> Result<()> {
    let port = clap::value_t!(matches, "port", u16)?;

    let app = app::App::new_with_config(config.app).await?;
    let api = API::new(logger, app);

    let r = api.rocket(port)?;
    let _ = r.launch().await?;

    return Ok(());
}
