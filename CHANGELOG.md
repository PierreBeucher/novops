# Changelog

## [0.13.0](https://github.com/PierreBeucher/novops/compare/v0.12.1...v0.13.0) (2024-04-20)


### Features

* added plain install script ([02839fd](https://github.com/PierreBeucher/novops/commit/02839fdd69f037f7c01c59e5fe553528d0030b4e))
* update Cargo deps (security issues) ([2778930](https://github.com/PierreBeucher/novops/commit/2778930aeeb81d1db34bda3abdd2c88c7b885f10))
* update Cargo deps (security issues) ([10c6db5](https://github.com/PierreBeucher/novops/commit/10c6db5463246140a33fa34ff480244ce7de9d1e))


### Bug Fixes

* install script checksum verify ([77c59bf](https://github.com/PierreBeucher/novops/commit/77c59bffff60fe69ea6eb18b937bcb27319e6d9e))

## [0.12.1](https://github.com/PierreBeucher/novops/compare/v0.12.0...v0.12.1) (2024-03-21)


### Bug Fixes

* truncate file output before writing ([72756ac](https://github.com/PierreBeucher/novops/commit/72756ac657ae48bcdba59ae961e68c6d80d78f7e))

## [0.12.0](https://github.com/PierreBeucher/novops/compare/v0.11.0...v0.12.0) (2024-02-18)


### Features

* macOS (Darwin) support ([4daa2df](https://github.com/PierreBeucher/novops/commit/4daa2dfc22bbe55dfb53c7e1f8a2ae9960126a69))
* multi-arch support (x86_64 / aarch64) ([b673e30](https://github.com/PierreBeucher/novops/commit/b673e30e66ce7cb21418c8299a36f921ebdf59c5))


### Bug Fixes

* read both .yaml and .yml config file by default ([3e3080c](https://github.com/PierreBeucher/novops/commit/3e3080cd70dc54fc795f3d50ae9efc3ff2cec728))

## [0.11.0](https://github.com/PierreBeucher/novops/compare/v0.10.0...v0.11.0) (2023-12-27)


### Features

* console drop-down for environment selection ([d908e51](https://github.com/PierreBeucher/novops/commit/d908e518f3d529ed3388f6d2b650a12ce2077bee))
* safety check for custom workdir permissions and ownership ([dce1c02](https://github.com/PierreBeucher/novops/commit/dce1c02783a3799c73b6c10d4c536430c3917d3c))
* SOPS (Secrets OPerationS) module ([a19861c](https://github.com/PierreBeucher/novops/commit/a19861c2a929106f7c5ab520220f9bb862d63953))

## [0.10.0](https://github.com/PierreBeucher/novops/compare/v0.9.0...v0.10.0) (2023-12-23)


### Features

* list environments and outputs ([c2b9787](https://github.com/PierreBeucher/novops/commit/c2b97878081927c8e72941c4bff473e54c1b9f0a))
* ordered list of environments ([93acf7f](https://github.com/PierreBeucher/novops/commit/93acf7fb2d64d71df817df8e3c3c290b17da5607))

## [0.9.0](https://github.com/PierreBeucher/novops/compare/v0.8.0...v0.9.0) (2023-12-09)


### Features

* Discord community channel ([b55c4be](https://github.com/PierreBeucher/novops/commit/b55c4bea40660648d1b1a57895f49a624c0ba98b))
* expose NOVOPS_ENVIRONMENT variable by default ([9fdfb85](https://github.com/PierreBeucher/novops/commit/9fdfb85dc60d1fb8e757fbf8e5dec40314f83406))
* provide Cachix cache to Nix Flake users ([b5fe8f4](https://github.com/PierreBeucher/novops/commit/b5fe8f427764a7e7fb4090d71e3255e6bee7c0a7))
* support for NOVOPS_* environment variables ([36dc800](https://github.com/PierreBeucher/novops/commit/36dc800b95858f4b4fa965c1939229bcab93fdde))


### Bug Fixes

* tty check logic always failing despite symlink set ([6d49959](https://github.com/PierreBeucher/novops/commit/6d49959184fb7d8c915eb6329b53ee99092d0a3f))

## [0.8.0](https://github.com/PierreBeucher/novops/compare/v0.7.0...v0.8.0) (2023-08-27)


### Features

* check if stdout is tty to prevent secret leak on screen ([4894031](https://github.com/PierreBeucher/novops/commit/48940311e8b15aee7e5fdf41d05add15457cf209))

## [0.7.0](https://github.com/PierreBeucher/novops/compare/v0.6.1...v0.7.0) (2023-08-23)


### Features

* multiple export format, default to stdout and only use stderr for logging ([78b17c4](https://github.com/PierreBeucher/novops/commit/78b17c4e0ef32bde7740c21216cbf4b20a99104a))
* novops run: subprocess with loaded environment ([fede6cb](https://github.com/PierreBeucher/novops/commit/fede6cba0a51d74e5f02c95b968b6fd782a6415c))
* remove required top-level name ([a60aec7](https://github.com/PierreBeucher/novops/commit/a60aec7b77d6a1e2198e395bd7bf11806319a955))


### Bug Fixes

* incorrect permission for /tmp subdirectory when fallbacking from XDG_RUNTIME_DIR ([1056c60](https://github.com/PierreBeucher/novops/commit/1056c60a9cacb1ac72f6e17b507ef0103831048f))

## [0.6.1](https://github.com/PierreBeucher/novops/compare/v0.6.0...v0.6.1) (2023-06-25)


### Bug Fixes

* Google client Application Default Credentials ([4c16437](https://github.com/PierreBeucher/novops/commit/4c1643796e9db7488119ffc98e2fc7da654972d0))

## [0.6.0](https://github.com/PierreBeucher/novops/compare/v0.5.0...v0.6.0) (2023-06-19)


### Features

* load Hashivault token file from default location or provide path in config ([08f2e05](https://github.com/PierreBeucher/novops/commit/08f2e058c6508954fe4f018e052bf69d90061f51))


### Bug Fixes

* Cargo.nix sync ([bf08ecd](https://github.com/PierreBeucher/novops/commit/bf08ecd97e631bd6317359da563a30ce8b3d7e7e))
* Nix ref for vaultrs ([d9d4f30](https://github.com/PierreBeucher/novops/commit/d9d4f30540e65fd2380e45be8021f829296f8b87))
* Nix ref for vaultrs ([fa41126](https://github.com/PierreBeucher/novops/commit/fa411267e25dedef4ef3be0c0f29553a6759ea20))

## [0.5.0](https://github.com/PierreBeucher/novops/compare/v0.4.0...v0.5.0) (2023-05-17)


### Features

* Hashivault module with AWS Secret Engine ([141e282](https://github.com/PierreBeucher/novops/commit/141e282394cad8d7c2cece9077113861c366e986))



## [0.4.0](https://github.com/PierreBeucher/novops/compare/v0.3.0...v0.4.0) (2023-03-11)


### Bug Fixes

* assume_role profile is ignored ([0073b51](https://github.com/PierreBeucher/novops/commit/0073b514345b27a5c9b7004baa7f445ad5915920))
* fully static build with Docker BuildKit ([c8d2a42](https://github.com/PierreBeucher/novops/commit/c8d2a42c412c7b92847d436a0387b1aafb026593))


### Features

* better error handling and context messages ([e9e083f](https://github.com/PierreBeucher/novops/commit/e9e083f587aa2219a84a92f30aadbf40a4e6af18))



## [0.3.0](https://github.com/PierreBeucher/novops/compare/v0.2.0...v0.3.0) (2023-01-21)


### Features

* Azure Keyvault Secret module ([f392182](https://github.com/PierreBeucher/novops/commit/f392182fe4ebb15ee54cdc32dbad40b8e87f6622))
* GCloud Secret Manager module ([44c8c88](https://github.com/PierreBeucher/novops/commit/44c8c880657da777a59854bb7f61f858975370a9))
* Hashicorp Vault KV1 module ([d35aa55](https://github.com/PierreBeucher/novops/commit/d35aa5597fb614f31129f7d0e7e79f03f66be66f))



## [0.2.0](https://github.com/PierreBeucher/novops/compare/v0.1.19...v0.2.0) (2023-01-06)


### Features

* added dry-run flag to ease testing ([0a7cb34](https://github.com/PierreBeucher/novops/commit/0a7cb3463fa9f2c4a0c24b2e5dfb23c4fc3685a6))
* AWS Secrets Manager module ([157ac13](https://github.com/PierreBeucher/novops/commit/157ac1324005fba464e8ccc3619ece8725139393))
* AWS SSM Parameter Store module ([d866f04](https://github.com/PierreBeucher/novops/commit/d866f04754503b44c353428d2e003e0cce1abe73))


## [0.1.19](https://github.com/PierreBeucher/novops/compare/v0.1.18...v0.1.19) (2022-11-30)


### Features

* licensed under GNU LGPLv3 ([927f000](https://github.com/PierreBeucher/novops/commit/927f000e5282cc5de70709879494526c90c1ded8))
