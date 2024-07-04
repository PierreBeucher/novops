import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";

// SSM
const ssmParamString = new aws.ssm.Parameter("novops-test-ssm-param-string", {
    name: "novops-test-ssm-param-string",
    type: "String",
    value: "novops-string-test",
})

const ssmParamSecureString = new aws.ssm.Parameter("novops-test-ssm-param-secureString", {
    name: "novops-test-ssm-param-secureString",
    type: "SecureString",
    value: "novops-string-test-secure",
})

// Secret Manager
const secretManagerSecretString = new aws.secretsmanager.Secret(`novops-test-secretsmanager-string`, {
    name: "novops-test-secretsmanager-string",
})

const secretManagerSecretStringVersion = new aws.secretsmanager.SecretVersion(`novops-test-secretsmanager-string-version`, {
    secretId: secretManagerSecretString.id,
    secretString: "Some-String-data?1548a~#{[["
})

const secretManagerSecretBinary = new aws.secretsmanager.Secret(`novops-test-secretsmanager-binary`, {
    name: "novops-test-secretsmanager-binary",
})

const secretManagerSecretBinaryVersion = new aws.secretsmanager.SecretVersion(`novops-test-secretsmanager-binary-version`, {
    secretId: secretManagerSecretBinary.id,
    secretBinary: "8J+Slg==" // base64 💖 emoji [240, 159, 146, 150]
})

// IAM
const novopsTestRole = new aws.iam.Role("novopsTestAwsAssumeRole", {
    assumeRolePolicy: JSON.stringify({
        Version: "2012-10-17",
        Statement: [{
            Action: "sts:AssumeRole",
            Principal: {
                Service: "ec2.amazonaws.com",
            },
            Effect: "Allow",
            Sid: "",
        }],
    }),
})

const novopsTestRolePolicy = new aws.iam.RolePolicyAttachment("novopsTestAwsAssumeRolePolicy", {
    role: novopsTestRole.name,
    policyArn: aws.iam.ManagedPolicies.AmazonEC2FullAccess,
})

// S3
const bucket = new aws.s3.Bucket("novops-test-bucket", {
    bucket: "novops-test-bucket",
})

new aws.s3.BucketObject("variable-object", {
    bucket: bucket.bucket,
    key: "path/to/var",
    source: new pulumi.asset.StringAsset("variable-content"),
})

new aws.s3.BucketObject("file-object", {
    bucket: bucket.bucket,
    key: "path/to/file",
    source: new pulumi.asset.StringAsset("file-content"),
})