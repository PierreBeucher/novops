use aws_sdk_sts::Client as StsClient;
use uuid::Uuid;
use serde::Deserialize;
use crate::novops::ResolvedNovopsVariable;

#[derive(Debug, Deserialize, Clone)]
pub struct NovopsAws {
    pub assume_role: AwsAssumeRole
}

#[derive(Debug, Deserialize, Clone)]
pub struct AwsAssumeRole {
    pub role_arn: String,
    pub source_profile: String
}

/**
 * Parse aws.assume_role cond and return a Vec of variables to epxort with credentials
 */
pub async fn parse_aws_assumerole(aws_assume_role: &AwsAssumeRole, app_name: &String, env_name: &String) -> Vec<ResolvedNovopsVariable>{
    
    let config = aws_config::from_env()
        .credentials_provider(
            aws_config::profile::ProfileFileCredentialsProvider::builder()
            .profile_name(&aws_assume_role.source_profile)
            .build()
        )
        .load()
        .await;
    
    // let config = aws_config::from_env().credentials_provider(cred)
    let sts_client = StsClient::new(&config);
    let assumed_role = sts_client.assume_role()
        .role_arn(&aws_assume_role.role_arn)
        .role_session_name(&format!("novops-{:}-{:}-{:}", env_name, app_name, Uuid::new_v4()))
        .send().await;
    let aso = match assumed_role {
        Ok(c) => {c},
        Err(e) => panic!("Coudn't assume role {:} (source profile {:}: {:?}", 
            aws_assume_role.role_arn, aws_assume_role.source_profile, e),
    };

    let creds = &aso.credentials.unwrap();
    let access_key = creds.access_key_id.as_ref().unwrap();
    let secret_key = creds.secret_access_key.as_ref().unwrap();
    let session_token = creds.session_token.as_ref().unwrap();

    return vec![
        ResolvedNovopsVariable{name: "AWS_ACCESS_KEY_ID".into(), value: access_key.clone()},
        ResolvedNovopsVariable{name: "AWS_SECRET_ACCESS_KEY".into(), value: secret_key.clone()},
        ResolvedNovopsVariable{name: "AWS_SESSION_TOKEN".into(), value: session_token.clone()} 
    ]

}