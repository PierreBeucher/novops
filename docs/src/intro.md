# Introduction

Novops, the universal secret and configuration manager for development, applications and CI.

- [Features](#features)
- [Getting Started](#getting-started)
- [🔐 Security](#-security)
- [Why Novops?](#why-novops)
- [How is Novops different than other secret management tools?](#how-is-novops-different-than-other-secret-management-tools)

## Features

![novops-features](assets/novops-features.jpg)

- Securely load secrets in protected in-memory files and environment variables
- Generate temporary credentials and secrets
- Fetch secrets from anywhere: Hashicorp Vault, AWS, Google Cloud, Azure, SOPS [and more](https://novops.dev/config/index.html). Avoid syncing secrets between local tool, CI/CD, and Cloud secret services.
- Feed secrets directly to command or process with `novops run`, easing usage of tools like Terraform, Pulumi, Ansible...
- Manage multiple environments: `dev`, `preprod`, `prod`... and configure them as you need.
- Easy installation with fully static binary or Nix

## Getting Started

[Go Get Started !](getting-started.md)

## 🔐 Security

Secrets are loaded temporarily as environment variables or in a protected `tmpfs` directory and kept only for as long as they are needed.. See [Novops Security Model](./security.html) for details

## Why Novops?

Novops help manage secrets and configurations to avoid keeping them (often insecurely) in gitignored folders, forgotten on your machine and spread around CI and server configs. 

See [Why Novops?](./why-novops.md) for a detailed explanation and history. 

## How is Novops different than other secret management tools? 

- **Universal**: unlike platform-specific tools like `aws-vault`, Novops is designed to be versatile and flexible, meeting a wide range of secret management needs across different platforms and tools.
- **Free and Open Source**, Novops is not trying to sell you a platform or subscription. 
- **Generate temporary credentials** for Clouders like AWS, where most tools only manage static key/value secrets.
- **Manages multi-environment** natively without requiring complex setup like `teller`.
- **Fetch secrets from source** avoiding need for syncing manually with some encrypted file like.
