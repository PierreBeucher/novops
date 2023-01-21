# Security - how safe is Novops?

## Data storage

Novops, in-fine, generate files with sensitive data. We want these sensitive data to be protected and remain on disk only as long as they are needed. 

In short:
- If `XDG_RUNTIME_DIR` variable exists, Novops will save files in this secure directory
- Otherwise files are saved under a user-specific `/tmp` directory
- Alternatively you can specify `-w PATH` to point to a secure directory

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

See `prepare_working_directory()` in [`src/lib.rs`](../src/lib.rs)

## External libraries and CVEs

We do our best to choose secured open source libraries and update them regularly to latest version to get latest security patches. 