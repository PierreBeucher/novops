# Why Novops?

## A story you've probably heard or lived

Consider a typical Infra as Code project:
- Deployment tool (Terraform, Ansible...) managing Cloud infrastructure / machines configurations
- CI/CD (GitLab, Jenkins...) with GitOps implementation for multiple environments: dev, staging, prod, etc.

Typically your team manages configs and secrets in several places:
- A Secret Manager (Hashicorp Vault, AWS Secret Manager...) holding most secrets
- Internal CI/CD configs and secrets (tokens for Secret Manager, per-environment configs, etc.) provided as files or environment variables

![novops-before](assets/novops-before.jpg)

You probably end-up with situations such as:
- Its hard - or even impossible - to reproduce CI/CD behavior for debugging and development purposes
- Reproducing CI/CD behavior requires a bunch of (secret) files and environment variables, often copied from CI/CD itself
- Additional setup is required to reproduce CI/CD context (built-in variables, etc.)
- You must keep local copy of boiler-plate config and secrets up-to-date, risking consequent drifts and secret leaking
- Your may not even test locally altogether, waiting ages for CI/CD workflow to complete after any minor change 

## Use Novops to reduce CI/CD drift

Novops allow your team to manage secrets & configs from a single Git-versioned file. Reproducing CI/CD context locally becomes easy and secure:
- Files and environment variables are loaded from the same source of truth
- Secrets are stored securely and can be cleaned-up easily
- It's then easy to reproduce the same context locally and on CI/CD

![novops-after](assets/novops-after.jpg)