use crate::core::{ResolveTo, NovopsContext};
use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;
use log::debug;
use crate::modules::aws::client::get_client;

/// Reference an S3 object
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsS3ObjectInput {
    aws_s3_object: AwsS3Object
}

/// Reference an S3 object
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsS3Object {
    /// S3 bucket name
    pub bucket: String,

    /// S3 object key
    pub key: String,
}

#[async_trait]
impl ResolveTo<String> for AwsS3ObjectInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {
        let client = get_client(&ctx).await;
        let result = client.get_s3_object(&self.aws_s3_object.bucket, &self.aws_s3_object.key).await?;
        
        debug!("Got file {:} from S3 bucket {:}", &self.aws_s3_object.key, &self.aws_s3_object.bucket);
        
        Ok(String::from_utf8(result.body.collect().await?.into_bytes().to_vec())?)
    }
}
