# Introduction

`novops` is a platform agnostic secret and config manager for local development and CI.

## Features

![novops-features](assets/novops-features.jpg)

- Securely load secrets and configs as files or environment variables
- Provide the same set of environment variables & files between your local dev setup and CI/CD
- Integrate with various secret and config providers: Hashicorp Vault, AWS, Google Cloud, Azure...
- Easily integrated within most shells and CI systems: Gitlab, GitHub, Jenkins...
- Implement development and CI workflow for DevOps tooling: Terraform, Pulumi, Ansible...
- Manage multi-environment (dev, preprod, prod...)
- Quick and easy installation with fully static binary

## Install & Get Started

See [Installation](install.md) and [Go Get Started](getting-started.md) for next steps!

## Why Novops?

### A story you've probably heard before...

Consider a typical Infra as Code project:
- Deployment tool such as Terraform, Ansible or Pulumi
- CI/CD with GitLab CI, GitHub Action or Jenkins
- Multiple environments (dev, prod...)

Secrets are managed by either:
- A secret / config manager like Hashicorp Vault or AWS Secret Manager
- Vendor-specific CI/CD secret storage provided as environment variables or files

![novops-before](assets/novops-before.jpg)

Your team hits the typical pain points:

- You're not able to reproduce locally what happens on CI, spending hours debugging the most simple CI bug.
- Even with Docker, Nix, GitPod or another tooling providing a reproducible environment, you still have significant drift as environment variables and config files are still different and hard to retrieve locally
- Developer spends way too much time setting up their local environment, including the tons of secrets & credentials required to run your project
- Important secrets are kept locally on developers laptop and forgotten about.
- Your developers want to access Production and sensible environments but they're locked out as per lack of possibility to provide scopes, temporary and secure secrets

### Use Novops to reduce CI/CD drift and load secrets securely 

Novops allow your team to manage secrets & configs from a single Git-versioned file. Reproducing CI/CD context locally becomes easy and secure:
- Files and environment variables are loaded from the same source of truth
- Secrets are stored securely and can be cleaned-up easily
- It's then easy to reproduce the same context locally and on CI/CD

![novops-after](assets/novops-after.jpg)
