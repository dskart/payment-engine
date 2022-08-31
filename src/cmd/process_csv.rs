use crate::{
    app::{self, App},
    cmd::Config,
    store, Result,
};
use simple_error::bail;

pub const ARG_NAME: &str = "input-file";
pub fn arg<'a>() -> clap::Arg<'a> {
    clap::Arg::new("input-file")
}

pub async fn run(logger: slog::Logger, _config: Config, matches: &clap::ArgMatches) -> Result<()> {
    let app = App::new_with_config(app::Config {
        store: store::Config {
            in_memory: true,
            ..Default::default()
        },
    })
    .await?;
    let sess = app.new_session(logger);

    if let Some(file_path) = matches.get_one::<String>(ARG_NAME) {
        info!(sess.logger(), "Processing {:} ...", file_path);
        sess.process_csv(file_path.clone()).await?;
        sess.output_all_accounts().await?;
        info!(sess.logger(), "DONE");
    } else {
        bail!("no csv file given")
    }

    return Ok(());
}
