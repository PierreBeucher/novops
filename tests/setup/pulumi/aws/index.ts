import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as docker from "@pulumi/docker";

// Adapted from https://github.com/localstack/localstack/blob/master/docker-compose.yml
const localstackContainer = new docker.Container("localstack", {
    image: "localstack/localstack:3.4.0",
    name: "novops-localstack",
    ports: [
        {
            internal: 4566,
            external: 4566,
            protocol: "tcp",
        },
        {
            internal: 4510,
            external: 4510,
            protocol: "tcp",
        },
        {
            internal: 4559,
            external: 4559,
            protocol: "tcp",
        },
    ],
    envs: [
        // `DEBUG=true`,
        "DOCKER_HOST=unix:///var/run/docker.sock",
    ],
    volumes: [
        {
            volumeName: "novops-localstack",
            containerPath: "/var/lib/localstack",
        },
        {
            hostPath: "/var/run/docker.sock",
            containerPath: "/var/run/docker.sock",
        },
    ],
}, {
    // Pulumi shows a constant diff on image even though it did not change, causing container recreation and loss of all resources
    // No need to change here as this stack is ephemeral anyway
    ignoreChanges: ["image"] 
})

const awsResourceOpts : pulumi.CustomResourceOptions = {
    dependsOn: localstackContainer,
    deletedWith: localstackContainer,
}
// SSM
const ssmParamString = new aws.ssm.Parameter("novops-test-ssm-param-string", {
    name: "novops-test-ssm-param-string",
    type: "String",
    value: "novops-string-test",
}, awsResourceOpts)

const ssmParamSecureString = new aws.ssm.Parameter("novops-test-ssm-param-secureString", {
    name: "novops-test-ssm-param-secureString",
    type: "SecureString",
    value: "novops-string-test-secure",
}, awsResourceOpts)

// Secret Manager
const secretManagerSecretString = new aws.secretsmanager.Secret(`novops-test-secretsmanager-string`, {
    name: "novops-test-secretsmanager-string",
})

const secretManagerSecretStringVersion = new aws.secretsmanager.SecretVersion(`novops-test-secretsmanager-string-version`, {
    secretId: secretManagerSecretString.id,
    secretString: "Some-String-data?1548a~#{[["
}, awsResourceOpts)

const secretManagerSecretBinary = new aws.secretsmanager.Secret(`novops-test-secretsmanager-binary`, {
    name: "novops-test-secretsmanager-binary",
}, awsResourceOpts)

const secretManagerSecretBinaryVersion = new aws.secretsmanager.SecretVersion(`novops-test-secretsmanager-binary-version`, {
    secretId: secretManagerSecretBinary.id,
    secretBinary: "8J+Slg==" // base64 💖 emoji [240, 159, 146, 150]
},  awsResourceOpts)

// IAM
const novopsTestRole = new aws.iam.Role("novops-test-aws-assume-role", {
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
}, awsResourceOpts)

const novopsTestRolePolicy = new aws.iam.RolePolicyAttachment("novops-test-aws-assume-role-policy", {
    role: novopsTestRole.name,
    policyArn: aws.iam.ManagedPolicies.AmazonEC2FullAccess,
}, awsResourceOpts)

// S3
const bucket = new aws.s3.Bucket("novops-test-bucket", {
    bucket: "novops-test-bucket",
}, awsResourceOpts)

new aws.s3.BucketObject("variable-object", {
    bucket: bucket.bucket,
    key: "path/to/var",
    source: new pulumi.asset.StringAsset("variable-content"),
},awsResourceOpts)

new aws.s3.BucketObject("file-object", {
    bucket: bucket.bucket,
    key: "path/to/file",
    source: new pulumi.asset.StringAsset("file-content"),
},awsResourceOpts)