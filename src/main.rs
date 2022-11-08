use crate::aws::{get_instance_id, AwsClient};
use anyhow::Result;
use clap::{ArgAction, Parser, ValueEnum};
use log::{info, warn, LevelFilter};
use std::sync::Arc;
use tokio::task::JoinSet;

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

    let instance_id = get_instance_id().await?;
    info!("Running on instance id {}", instance_id);

    let aws_client = Arc::new(AwsClient::new(&instance_id, None).await);

    let mut task_set = JoinSet::new();
    for tg_arn in config.tg_arns {
        let aws_client = aws_client.clone();
        let tg_arn = tg_arn.clone();
        match config.action {
            Action::Deregister => {
                task_set
                    .spawn(async move { aws_client.deregister_target(&tg_arn, config.wait).await });
            }
            Action::Register => {
                task_set.spawn(async move { aws_client.register_target(&tg_arn).await });
            }
        }
    }

    while let Some(res) = task_set.join_next().await {
        if let Err(err) = res {
            warn!("Task error: {:?}", err);
        }
    }

    info!("Done");

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
    #[arg(long, default_value_t = true)]
    wait: bool,
}
