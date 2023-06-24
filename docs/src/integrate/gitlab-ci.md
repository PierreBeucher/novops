# GitLab CI

GitLab uses YAMl to define jobs. You can either:

## Use a Docker image packaging Novops

See [Docker integration](docker.md) to build a Docker image packaging Novops, then use it in on CI such as:

```yaml
job-with-novops:
  image: your-image-with-novops
  stage: test
  script:
    # Load config
    # Specify environment to avoid input prompt
    - novops load -s .envrc -e $CI_ENVIRONMENT_NAME && source .envrc
    
    # Environment is now loaded!
    # Run others commands...
    - pulumi up -yrf
```

## Install novops on-the-fly

_This method is not recommended. Prefer using an image packaging Novops to avoid unnecessary network load._

You can download `novops` binary on the fly:

```yaml
job-with-novops:
  image: some-image
  stage: test
  variables:
    NOVOPS_VERSION: "0.6.0"
  script:
    # Download novops
    - >-
      curl -L "https://github.com/novadiscovery/novops/releases/download/v${NOVOPS_VERSION}/novops-X64-Linux.zip" -o novops.zip &&
      unzip novops.zip &&
      mv novops /usr/local/bin/novops
    
    # Load config
    # Specify environment to avoid input prompt
    - novops load -s .envrc -e $CI_ENVIRONMENT_NAME && source .envrc
    
    # Environment is now loaded!
    # Run others commands...
    - pulumi up -yrf
```


## Authenticating to external provider on CI

GitLab provides facility to [authenticate with external party services via OIDC tokens](https://docs.gitlab.com/ee/ci/secrets/id_token_authentication.html). You can leverage it to authenticate on Hashicorp Vault, AWS, or another provider before.

Alternatively, you can use CI environment variables to authenticate directly (see module Authentication docs for details)

More examples will be provided soon. 