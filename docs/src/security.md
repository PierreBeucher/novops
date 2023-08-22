# Novops Security Model

- [Overview](#overview)
- [In-memory temporary secrets](#in-memory-temporary-secrets)
  - [Wait... Novops may create files but does not write to disk? 🤔](#wait-novops-may-create-files-but-does-not-write-to-disk-)
  - [With XDG_RUNTIME_DIR](#with-xdg_runtime_dir)
  - [Without XDG_RUNTIME_DIR](#without-xdg_runtime_dir)
- [External libraries and CVEs](#external-libraries-and-cves)

## Overview

Novops load secrets safely. In short:
- Secrets are loaded directly in-memory so they are kept only for as long as they are needed
- Novops does not persist any secret. `.novops.yml` config file does not contain any secret and can be safely versionned with Git or version control tool.
- Libraries used are carefully chosen and regularly updated.

## In-memory temporary secrets

Novops load secrets in-memory, mainly as environment variables but also as files. By sourcing them into your current shell session or using `novops run` to run a sub-process with variables, you ensure variables will only persist for as long as they're needed and no other process or user can access them.

### Wait... Novops may create files but does not write to disk? 🤔

Novops may generate files in some situations - but they're written to a [`tmpfs` file system](https://www.kernel.org/doc/html/latest/filesystems/tmpfs.html) (in-memory file system), not on hard drive disk ! Furthermore, Novops uses a secure directory only user running Novops can access (`XDG_RUNTIME_DIR` or secure directory in `/tmp`, see below).

Novops may generate files when:
- Using `novops load -s SYMLINK` creates an exportable `dotenv` file in s secure directory
- Using the [`files`](config/files-variables.md) module

In short:
- If `XDG_RUNTIME_DIR` variable exists, Novops will save files in this secure directory
- Otherwise files are saved under a user-specific `/tmp` directory
- Alternatively you can specify `novops load -w PATH` to point to a custom secure directory

_Note: using environment variables is still safer than files, so prefer environment variables if you can !_

### With XDG_RUNTIME_DIR

If `XDG_RUNTIME_DIR` variable is set, secrets are stored as files under a subdirectory of `XDG_RUNTIME_DIR`. In short, this directory is:
- Owned and read/writable only by current user
- Bound to lifecycle of current user session (i.e. removed on logout or shutdown)
- Usually mounted as a [`tmpfs`](https://www.kernel.org/doc/html/latest/filesystems/tmpfs.html) volume, but not necessarily (spec do not mention it)

This ensures loaded secrets are **securely stored while being used and not persisted unnecessarily.**

To read more about XDG Runtime Dir, see:

- [Official XDG specifications](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [This stack exchange post](https://askubuntu.com/questions/872792/what-is-xdg-runtime-dir)

### Without XDG_RUNTIME_DIR

If `XDG_RUNTIME_DIR` is not available, Novops will issue a warning and try to emulate a XDG-lke behavior under a `/tmp` sub-folder. There's not guarantee it will fully implement [XDG specs](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html), but directory is created such as:

- Owned and read/writable only by current user
- By using a `/tmp` sub-folder, we reasonably assume content won't persist between reboot and logout

See `prepare_working_directory()` in [`src/lib.rs`](https://github.com/PierreBeucher/novops/blob/main/src/lib.rs)

## External libraries and CVEs

Novops uses open source libraries and update them regularly to latest version to get security patches and CVE fixes. 