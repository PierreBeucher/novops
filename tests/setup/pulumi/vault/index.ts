import * as pulumi from "@pulumi/pulumi";
import * as vault from "@pulumi/vault";
import * as docker from "@pulumi/docker";
import * as aws from "@pulumi/aws";
import * as k8s from "@pulumi/kubernetes";
import * as fs from "fs";

// KV2
const kv2Engine = new vault.Mount("kv2Engine", {
    type: "kv",
    path: "kv2",
    options: {
        version: "2"
    },
})

const kv2Secret = new vault.kv.SecretV2("kv2Secret", {
    mount: kv2Engine.path,
    name: "test_hashivault_kv2",
    dataJson: JSON.stringify({
        novops_secret: "s3cret_kv2",
    }),
})

// KV1
const kv1Engine = new vault.Mount("kv1Engine", {
    type: "kv",
    path: "kv1",
})

const kv1Secret = new vault.kv.Secret("kv1Secret", {
    path: pulumi.interpolate`${kv1Engine.path}/test_hashivault_kv1`,
    dataJson: JSON.stringify({
        novops_secret: "s3cret_kv1",
    }),
})

const novopsTestRole = new aws.iam.Role("novopsTestAwsAssumeRole", {
    name: "test_role",
    assumeRolePolicy: JSON.stringify({
        Version: "2012-10-17",
        Statement: [{
            Action: "sts:AssumeRole",
            Principal: {
                AWS: "*"
            },
            Effect: "Allow",
        }],
    }),
})

const awsMount = "test_aws"
const awsConfig = new vault.aws.SecretBackend("awsConfig", {
    path: awsMount,
    accessKey: "test_key",
    secretKey: "test_secret",
    stsEndpoint: pulumi.interpolate`http://localstack:4566/`,
    iamEndpoint: pulumi.interpolate`http://localstack:4566/`,
})

const awsRole = new vault.aws.SecretBackendRole("awsRole", {
    roleArns: [novopsTestRole.arn],
    backend: awsMount,
    name: "test_role",
    credentialType: "assumed_role",
}, {
    dependsOn: [awsConfig]
})

// AppRole
const approleAuth = new vault.AuthBackend("approleAuth", {
    type: "approle",
    path: "approle",
})

new vault.approle.AuthBackendRole("roleWithoutSecret", {
    roleName: "without-secret",
    backend: approleAuth.path,
    roleId: "role_id_without_secret",
    bindSecretId: false,
    tokenBoundCidrs: ["0.0.0.0/0"]
})

new vault.approle.AuthBackendRole("roleWithSecret", {
    roleName: "with-secret",
    backend: approleAuth.path,
    roleId: "role_id_with_secret",
    bindSecretId: true,
    tokenBoundCidrs: ["0.0.0.0/0"]
})

// JWT
const publicKey = fs.readFileSync("jwt_public_key.pem", "utf8");
const jwtPath = "jwt"

const jwtAuth = new vault.jwt.AuthBackend("jwtAuth", {
    path: jwtPath,
    jwtValidationPubkeys: [publicKey],    
})

new vault.jwt.AuthBackendRole("jwtAuthConfig", {
    backend: jwtPath,
    roleName: "test-role",
    boundSubject: "novops_test_subject",
    userClaim: "sub",
    roleType: "jwt",
}, {
    dependsOn: [ jwtAuth ]
})

// Kubernetes

const kubCaCert = fs.readFileSync("../../k8s/ca.pem", "utf8");

const kubAuth = new vault.AuthBackend("kubernetesAuth", {
    type: "kubernetes",
    path: "k8s",
})

const kubAuthConfig = new vault.kubernetes.AuthBackendConfig("kubernetesAuthConfig", {
    backend: kubAuth.path,
    kubernetesHost: "https://novops-auth-test-control-plane:6443",
    kubernetesCaCert: kubCaCert,
    disableIssValidation: true,
}, {
    deletedWith: kubAuth
})

const kubRole = new vault.kubernetes.AuthBackendRole("kubernetesRole", {
    backend: kubAuth.path,
    roleName: "test-role",
    boundServiceAccountNames: ["*"],
    boundServiceAccountNamespaces: ["*"],
})

// Service Account for JWT
const kubProvider = new k8s.Provider("kub", {
    kubeconfig: "../../k8s/kubeconfig"
})

const kubResourceOpts: pulumi.CustomResourceOptions  = {
    provider: kubProvider
}
const serviceAccount = new k8s.core.v1.ServiceAccount("vaultJwtTestSA", {
    metadata: {
        name: "vault-jwt-test-sa",
        namespace: "default",
    },
}, kubResourceOpts)

const clusterRoleBinding = new k8s.rbac.v1.ClusterRoleBinding("vaultJwtTestSARoleBinding", {
    metadata: {
        name: "vault-jwt-test-sa-binding",
    },
    subjects: [{
        kind: "ServiceAccount",
        name: serviceAccount.metadata.name,
        namespace: serviceAccount.metadata.namespace,
    }],
    roleRef: {
        kind: "ClusterRole",
        name: "system:auth-delegator",
        apiGroup: "rbac.authorization.k8s.io",
    },
}, kubResourceOpts)


const serviceAccountToken = new k8s.core.v1.Secret("vaultJwtTestSAToken", {
    metadata: {
        name: "vault-jwt-test-sa-token",
        namespace: serviceAccount.metadata.namespace,
        annotations: {
            "kubernetes.io/service-account.name": serviceAccount.metadata.name,
        },
    },
    type: "kubernetes.io/service-account-token",
}, kubResourceOpts)