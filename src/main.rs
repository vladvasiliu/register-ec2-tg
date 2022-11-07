use anyhow::Result;
use clap::{ArgAction, Parser};
use log::{info, LevelFilter};

mod aws;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Config::parse();
    if systemd_journal_logger::connected_to_journal() {
        systemd_journal_logger::init_with_extra_fields(vec![(
            "VERSION",
            env!("CARGO_PKG_VERSION"),
        )])?;
    } else {
        simple_logger::SimpleLogger::new().init()?;
    }

    let level = if config.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    log::set_max_level(level);
    info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let instance_id = "i-0bd1cfe80316e9429";

    let aws_client = aws::AwsClient::new(instance_id, None).await;
    aws_client
        .register_target(&config.target_group_arns[0])
        .await?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Config {
    #[arg(long("target-group-arn"), action = ArgAction::Append, required=true)]
    target_group_arns: Vec<String>,
    #[arg(long)]
    deregistration_timeout: Option<u8>,
    #[arg(long, default_value_t = false)]
    debug: bool,
}
