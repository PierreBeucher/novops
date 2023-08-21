# Google Cloud

## Authentication

Authenticating with `gcloud` CLI is enough. Otherwise, provide credentials using [Application Default Credentials](https://cloud.google.com/docs/authentication/application-default-credentials):

- Set `GOOGLE_APPLICATION_CREDENTIALS` to a credential JSON file
- Setup creds using `gcloud` CLI
- Attached service account using VM metadata server to get credentials 

## Secret Manager

Retrieve secrets from [GCloud Secret Manager](https://cloud.google.com/secret-manager/docs) as env var or files:

```yaml
environments:
  dev:
    variables:
      - name: SECRETMANAGER_VAR_STRING
        value:
          gcloud_secret:
            name: projects/my-project/secrets/SomeSecret/versions/latest
            # validate_crc32c: true
  
    files:
      - name: SECRETMANAGER_VAR_FILE
        content:
          gcloud_secret:
            name: projects/my-project/secrets/SomeSecret/versions/latest
```
