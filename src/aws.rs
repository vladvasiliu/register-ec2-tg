use anyhow::{anyhow, Context, Result};
use aws_sdk_elasticloadbalancingv2::model::TargetDescription;

pub async fn get_instance_id() -> Result<String> {
    let imds_client = aws_config::imds::Client::builder()
        .build()
        .await
        .context("Failed to build imds client")?;

    imds_client
        .get("/latest/meta-data/instance-id")
        .await
        .context("Failed to retrieve instance id")
}

pub struct AwsClient {
    client: aws_sdk_elasticloadbalancingv2::Client,
    target_description: TargetDescription,
}

impl AwsClient {
    pub async fn new(instance_id: &str, port: Option<i32>) -> Self {
        let shared_config = aws_config::load_from_env().await;
        let client = aws_sdk_elasticloadbalancingv2::Client::new(&shared_config);
        let target_description = TargetDescription::builder()
            .id(instance_id)
            .set_port(port)
            .build();
        Self {
            client,
            target_description,
        }
    }

    pub async fn register_target(&self, tg_arn: &str) -> Result<()> {
        self.client
            .register_targets()
            .target_group_arn(tg_arn)
            .targets(self.target_description.clone())
            .send()
            .await
            .context("Failed to register target")?;
        Ok(())
    }

    pub async fn deregister_target(&self, tg_arn: &str) -> Result<()> {
        self.client
            .deregister_targets()
            .target_group_arn(tg_arn)
            .targets(self.target_description.clone())
            .send()
            .await
            .context("Failed to deregister target")?;
        Ok(())
    }

    pub async fn get_tg_deregistration_timeout(&self, tg_arn: &str) -> Result<u8> {
        let attributes_result = self
            .client
            .describe_target_group_attributes()
            .target_group_arn(tg_arn)
            .send()
            .await
            .context("Failed to describe target group attributes")?;
        let attributes = attributes_result
            .attributes()
            .ok_or_else(|| anyhow!("No attributes retrieved"))?;

        let mut timeouts = attributes
            .iter()
            .filter_map(|a| a.key().and(a.value()))
            .collect::<Vec<&str>>();

        if timeouts.len() != 1 {
            return Err(anyhow!("Deregistration delay not found"));
        }

        timeouts
            .pop()
            .unwrap()
            .parse::<u8>()
            .context("Invalid deregistration timeout")
    }
}
