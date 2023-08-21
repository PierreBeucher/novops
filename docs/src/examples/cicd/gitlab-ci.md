# GitLab CI

GitLab uses YAMl to define jobs. You can either:

## Use a Docker image packaging Novops

See [Docker examples](../docker.md) to build a container image packaging Novops, then use it in on CI such as:

```yaml
job-with-novops:
  image: your-image-with-novops
  stage: test
  script:
    # Load config
    # Specify environment to avoid input prompt
    - source <(novops load -e dev)
    
    # Environment is now loaded!
    # Run others commands...
    - terraform ... 
```

## Install novops on-the-fly

_This method is not recommended. Prefer using an image packaging Novops to avoid unnecessary network load._

You can download `novops` binary on the fly:

```yaml
job-with-novops:
  image: hashicorp/terraform:light
  stage: test
  script:
    # Download novops
    - |-
      curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops-X64-Linux.zip" -o novops.zip
      unzip novops.zip
      mv novops /usr/local/bin/novops
    
    # Load config
    # Specify environment to avoid input prompt
    - source <(novops load -e dev)
    
    # Environment is now loaded!
    # Run others commands...
    - terraform ... 
```

Alternatively, set a specific version:

```yaml
job-with-novops:
  # ...
  variables:
    NOVOPS_VERSION: "0.6.0"
  script:
    # Download novops
    - |-
      curl -L "https://github.com/PierreBeucher/novops/releases/download/v${NOVOPS_VERSION}/novops-X64-Linux.zip" -o novops.zip
      unzip novops.zip
      mv novops /usr/local/bin/novops
```

## Authenticating to external provider on CI

GitLab provides facility to [authenticate with external party services via OIDC tokens](https://docs.gitlab.com/ee/ci/secrets/id_token_authentication.html). You can leverage it to authenticate on Hashicorp Vault, AWS, or another provider before.

Alternatively, you can use CI environment variables to authenticate directly (see module Authentication docs for details)

More examples will be provided soon. 