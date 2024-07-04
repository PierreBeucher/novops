import * as az from "@pulumi/azure-native";

const resourceGroup = new az.resources.ResourceGroup("novops-testing", {
    resourceGroupName: "novops-testing",
})

const currentConfig = az.authorization.getClientConfigOutput()

const keyVault = new az.keyvault.Vault("novops-test", {
    resourceGroupName: resourceGroup.name,
    vaultName: "novops-test",
    properties: {
        sku: {
            family: "A",
            name: "standard"
        },
        tenantId: currentConfig.tenantId,
        enableSoftDelete: false,
        
        // Allow self to manage secrets
        accessPolicies: [{
            objectId: currentConfig.objectId,
            permissions: {
                secrets: [az.keyvault.KeyPermissions.All]
            },
            tenantId: currentConfig.tenantId,
        }],
    },
})

const secret = new az.keyvault.Secret("novops-test-secret", {
    secretName: "novops-test-kv",
    resourceGroupName: resourceGroup.name,
    vaultName: keyVault.name,
    properties: {
        value: "v3rySecret!",
    },
})