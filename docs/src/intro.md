# Introduction

Novops is a platform-agnostic secret manager for local development and CI.

- [Features](#features)
- [Getting Started](#getting-started)
- [🔐 Security](#-security)
- [Why Novops?](#why-novops)
  - [A story you've probably heard before...](#a-story-youve-probably-heard-before)
  - [Use Novops to reduce CI/CD drift and load secrets securely](#use-novops-to-reduce-cicd-drift-and-load-secrets-securely)
- [How is Novops different than Docker, Nix an alike?](#how-is-novops-different-than-docker-nix-an-alike)


## Features

![novops-features](assets/novops-features.jpg)

- Securely load secrets and generate temporary credentials directly in memory as environment variables or temporary files
- Fetch secrets at their source. No more syncing secrets between local tool, CI/CD, and Cloud secret service
- Fetch secrets from anywhere: Hashicorp Vault, AWS, Google Cloud, Azure...
- Reproduce the same workflow locally and on CI using shell, Docker, Gitlab, GitHub and more
- Manage multi-environments setup
- Easy installation with fully static binary or Nix 

## Getting Started

[Go Get Started !](getting-started.md)

## 🔐 Security

Secrets are loaded temporarily as environment variables or in a protected `tmpfs` directory and kept only for as long as they are needed.. See [Novops Security Model](./security.html) for details

## Why Novops?

### A story you've probably heard before...

Consider a typical Infra as Code project:
- Deployment tool such as Terraform, Ansible or Pulumi
- CI/CD with GitLab CI, GitHub Action or Jenkins
- Multiple environments (dev, prod...)

Secrets are managed by either:
- A secret / config manager like Hashicorp Vault or AWS Secret Manager
- Vendor-specific CI/CD secret storage provided as environment variables or files
- Secrets stored locally on developer machines, often synced manually from one of the above

![novops-before](assets/novops-before.jpg)

Your team hits the typical pain points:

- You're not able to reproduce locally what happens on CI, spending hours debugging CI bugs
- Even with Docker, Nix, GitPod or other tools providing a reproducible environment, you still have significant drift because of environment variables and config files
- Developer spends way too much time setting up their local environment, including the tons of secrets & credentials required to run your project
- Important secrets are kept locally on developers laptop and forgotten about
- Your developers want to access Production and sensible environments but they're locked out as per lack of possibility to provide scoped, temporary and secure credentials

### Use Novops to reduce CI/CD drift and load secrets securely 

Novops allow your team to manage secrets & configs from a single Git-versioned file. Reproducing CI/CD context locally becomes easier and more secure:
- Files and environment variables are loaded from the same source of truth
- Secrets are stored securely and can be cleaned-up easily
- It's then easy to reproduce the same context locally and on CI/CD

![novops-after](assets/novops-after.jpg)

## How is Novops different than Docker, Nix an alike?

Novops doesn't intend to replace tools like [Docker](https://www.docker.com/) (and other containerization system) or [Nix](https://nixos.org/), but to complete them. 

Docker, Nix or Vagrant are great to ensure **reproducibility it term and binaries, packages and tooling**. You're sure your CI is not using Python 3.7 while a developer runs Python 3.9 locally. 

However configurations and secrets are typically hard to reproduce, often locked in a vendor-specific CI config and/or hidden a way in a Secret Manager developers end-up copying permanently on their machines. `novops` help reduce the drift by providing a single source of truth for configs and secrets. 