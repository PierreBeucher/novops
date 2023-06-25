# Impersonate IAM Role

A typical workflow to deploy application and infrastructure on AWS is by [impersonating IAM Role](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles_common-scenarios.html).

Scenario:

- You want to deploy an application on AWS
- IAM Roles `my_app_deployment_dev` and `my_app_deployment_prod` are used to deploy Dev and Production environments respectively
- You want to provide a workflow working both locally and on CI for your developers, generating secure temporary credentials for deployment with minimal setup

Using Novops, you can define a setup like this:

Create a `.novops.yml` such as:

```yaml
name: my-app

environments:
  
  dev:
    aws:
      assume_role: 
        role_arn: arn:aws:iam::111122223333:role/my_app_deployment_dev

  prod:
    aws:
      assume_role: 
        role_arn: arn:aws:iam::111122223333:role/my_app_deployment_prod
```

Your developers just need to have local AWS credentials available and permission to impersonate IAM Role

On CI, you can configure AWS authentication before loading Novops config:
- [GitHub Action: Configuring OpenID Connect in Amazon Web Services](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-amazon-web-services)
- [GitLab CI: Configure OpenID Connect in AWS to retrieve temporary credentials](https://docs.gitlab.com/ee/ci/cloud_services/aws/)
- [Jenkins: How To Authenticate to AWS with the Pipeline AWS Plugin](https://docs.cloudbees.com/docs/cloudbees-ci-kb/latest/client-and-managed-controllers/how-to-authenticate-to-aws-with-the-pipeline-aws-plugin)

See [AWS Module documentation](../load/aws.md) for details. Alternatively you can use [Hashicorp Vault AWS Secret Engine](../load/hashicorp-vault.md)