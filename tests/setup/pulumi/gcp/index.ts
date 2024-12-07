import * as gcp from "@pulumi/gcp";
import * as pulumi from "@pulumi/pulumi";

const gcpConfig = new pulumi.Config("gcp");
const projectId = gcpConfig.require("project")

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

// Workload Identity Federation config to allow Azure authentication and test of Google WIF
// Retrieve outputs from Azure setup stack 
// As documented on https://cloud.google.com/iam/docs/workload-identity-federation-with-other-clouds#azure_2

const azStackRef = new pulumi.StackReference("az-stack-ref", { name: "pierrebeucher/novops-test-infra-azure/test" })
const azServicePrincipalObjectId = azStackRef.requireOutput("servicePrincipalObjectId") as pulumi.Output<string>
const azTenantId = azStackRef.requireOutput("tenantId") as pulumi.Output<string>
const azAppIdentifierUri = azStackRef.requireOutput("identifierUri") as pulumi.Output<string>

const poolId = "novops-test-identity-pool"
const providerId = "azure"

// Use an import-if-exists and retain pattern to avoid deletion on stack down and re-import on stack up
// Otherwise, as resources are soft-deleted, Google API would consider them already existing if Pulumi tried to recreate them
// See https://github.com/pulumi/pulumi-gcp/issues/1149
const identityPool = pulumi.output(gcp.iam.getWorkloadIdentityPool({ 
    workloadIdentityPoolId: poolId,
}).then(existingPool => 
    new gcp.iam.WorkloadIdentityPool("workload-identity-pool", {
        workloadIdentityPoolId: poolId,
    }, {
        import: existingPool.workloadIdentityPoolId,
        retainOnDelete: true,
    })
).catch(e => 
    new gcp.iam.WorkloadIdentityPool("workload-identity-pool", {
        workloadIdentityPoolId: poolId,
    }, { 
        retainOnDelete: true,
    })
))

const workloadIdentityProvider = pulumi.output(gcp.iam.getWorkloadIdentityPoolProvider({ 
    workloadIdentityPoolProviderId: providerId, 
    workloadIdentityPoolId: poolId
}).then(existingWifProvider => 
    new gcp.iam.WorkloadIdentityPoolProvider("workload-identity-pool-provider", {
        workloadIdentityPoolProviderId: providerId,
        workloadIdentityPoolId: poolId,
        oidc: {
            issuerUri: pulumi.interpolate`https://sts.windows.net/${azTenantId}/`,
            allowedAudiences: [azAppIdentifierUri],
        },
        attributeMapping: {
            "google.subject": "assertion.sub",
        },
    }, {
        import: existingWifProvider.name,
        retainOnDelete: true,
    })
).catch(e => 
    new gcp.iam.WorkloadIdentityPoolProvider("workload-identity-pool-provider", {
        workloadIdentityPoolProviderId: providerId,
        workloadIdentityPoolId: poolId,
        oidc: {
            issuerUri: pulumi.interpolate`https://sts.windows.net/${azTenantId}/`,
            allowedAudiences: [azAppIdentifierUri],
        },
        attributeMapping: {
            "google.subject": "assertion.sub",
        },
    }, {
        retainOnDelete: true,
    })
))

const secretManagerIamBinding = new gcp.projects.IAMMember("secret-manager-iam-binding", {
    project: projectId,
    role: "roles/secretmanager.secretAccessor",
    // Service Principal Object ID is the ID Google uses to identify or Azure user
    member: pulumi.interpolate`principal://iam.googleapis.com/projects/${projectId}/locations/global/workloadIdentityPools/${poolId}/subject/${azServicePrincipalObjectId}`,
}, {
    dependsOn: [workloadIdentityProvider]
})

const viewerIamBinding = new gcp.projects.IAMMember("viewer-iam-binding", {
    project: projectId,
    role: "roles/viewer",
    member: pulumi.interpolate`principal://iam.googleapis.com/projects/${projectId}/locations/global/workloadIdentityPools/${poolId}/subject/${azServicePrincipalObjectId}`,
}, {
    dependsOn: [workloadIdentityProvider]
})

export const workloadIdentityPoolName = identityPool.name
export const workloadIdentityPoolProviderName = workloadIdentityProvider.name
export const providerResourceName = workloadIdentityProvider.id
