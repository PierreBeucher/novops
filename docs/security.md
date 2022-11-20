# Security - how safe is Novops?

## Local storage of loaded secrets

### With `XDG_RUNTIME_DIR` 

If possible, secrets are stored as files under secure directory `XDG_RUNTIME_DIR` . In short, this directory is:
- Owned and read/writable only by current user
- Bound to lifecycle of current user session (i.e. removed on logout or shutdown)
  - Most implementation relies on a [`tmpfs` fileystem](https://www.kernel.org/doc/html/latest/filesystems/tmpfs.html)

This ensures loaded secrets are **securely stored while being used and not persisted unnecessarily.**

To read more about XDG Runtime Dir, see:

- [Official XDG specifications](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [This stack exchange post](https://askubuntu.com/questions/872792/what-is-xdg-runtime-dir)


### Without `$XDG_RUNTIME_DIR` 

If `XDG_RUNTIME_DIR` is not available, Novops will issue a warning and try to emulate a XDG-lke behavior under a `/tmp` sub-folder. There's not guarantee it will fully implement [XDG speficiations](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html), but directory is created such as:

- Owned and read/writable only by current user
- By using a `/tmp` sub-folder, we reasonnably assume content won't persist between reboot and logout

See `prepare_working_directory()` in [`src/lib.rs`](../src/lib.rs)

## External libraries and CVEs

We do our best to choose secured open source libraries and update them regularly to latest version to get latest security patches. 