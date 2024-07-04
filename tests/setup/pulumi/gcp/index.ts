import * as gcp from "@pulumi/gcp";

const secret = new gcp.secretmanager.Secret("test-secret", {
    secretId: "novops-test-secret",
    replication: {
        auto: {}
    }
})

const secretVersion = new gcp.secretmanager.SecretVersion("test-secret-version", {
    secret: secret.id,
    secretData: "very!S3cret",
})