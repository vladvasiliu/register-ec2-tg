use crate::aws::AwsClient;
use anyhow::Result;
use clap::{ArgAction, Parser, ValueEnum};
use log::{info, LevelFilter};

mod aws;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Config::parse();
    let level = if config.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    log::set_max_level(level);

    if systemd_journal_logger::connected_to_journal() {
        systemd_journal_logger::init_with_extra_fields(vec![(
            "VERSION",
            env!("CARGO_PKG_VERSION"),
        )])?;
    } else {
        simple_logger::SimpleLogger::new()
            .with_level(level)
            .init()?;
    }

    info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let instance_id = "i-0bd1cfe80316e9429";

    let aws_client = AwsClient::new(instance_id, None).await;
    aws_client.register_target(&config.tg_arns[0]).await?;

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Action {
    Register,
    Deregister,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Config {
    #[arg(long("tg-arn"), action = ArgAction::Append, required=true)]
    tg_arns: Vec<String>,
    #[arg(long)]
    deregistration_timeout: Option<u8>,
    #[arg(long, default_value_t = false)]
    debug: bool,
    #[arg(value_enum)]
    action: Action,
}
