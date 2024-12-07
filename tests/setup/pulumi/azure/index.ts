import * as az from "@pulumi/azure-native"
import * as azuread from "@pulumi/azuread"
import * as pulumi from "@pulumi/pulumi"

const resourceGroup = new az.resources.ResourceGroup("novops-testing", {
    resourceGroupName: "novops-testing",
})

const currentConfig = az.authorization.getClientConfigOutput()

// Key Vault secrets from testing
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

// Entra ID app to test Google Workload Identity Federation auth

const appDisplayName = "novops-test-google-workload-id-fed"
export const identifierUri = "api://novops-test/google-workload-id-fed"

const app = new azuread.Application("novops-test-google-workload-id-fed-app", {
    displayName: appDisplayName,
    owners: [currentConfig.objectId],
    identifierUris: [identifierUri]
})

const servicePrincipal = new azuread.ServicePrincipal("novops-test-google-workload-id-fed-service-principal", {
    clientId: app.clientId,
    owners: [currentConfig.objectId]
})
const servicePrincipalPassword = new azuread.ServicePrincipalPassword("novops-test-google-workload-id-fed-sp-password", {
    servicePrincipalId: servicePrincipal.id,
    endDate: "2099-01-01T01:01:42Z"
})

const roleReader = "acdd72a7-3385-48ef-bd42-f606fba81ae7"
const readerRoleAssignment = new az.authorization.RoleAssignment("novops-test-google-workload-id-fed-role-assignment", {
    principalId: servicePrincipal.objectId,
    principalType: "ServicePrincipal",
    roleDefinitionId: pulumi.interpolate`/subscriptions/${currentConfig.subscriptionId}/providers/Microsoft.Authorization/roleDefinitions/${roleReader}`, 
    scope: pulumi.interpolate`/subscriptions/${currentConfig.subscriptionId}`,
})

export const servicePrincipalClientId = servicePrincipal.clientId
export const servicePrincipalObjectId = servicePrincipal.objectId
export const tenantId = servicePrincipal.applicationTenantId
export const applicationObjectId = app.objectId
export const applicationClientId = app.clientId
export const password = servicePrincipalPassword.value