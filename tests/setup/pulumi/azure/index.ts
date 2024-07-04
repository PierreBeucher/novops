import * as azure from "@pulumi/azure-native";

const resourceGroup = new azure.resources.ResourceGroup("novops-testing", {
    resourceGroupName: "novops-testing",
})

const currentConfig = azure.authorization.getClientConfigOutput()

const keyVault = new azure.keyvault.Vault("novopsTest", {
    resourceGroupName: resourceGroup.name,
    vaultName: "novops-test",
    properties: {
        sku: {
            family: "A",
            name: "standard"
        },
        tenantId: currentConfig.tenantId
    },
})

const secret = new azure.keyvault.Secret("novopsTestSecret", {
    secretName: "novops-test-kv",
    resourceGroupName: resourceGroup.name,
    vaultName: keyVault.name,
    properties: {
        value: "v3rySecret!",
    },
})