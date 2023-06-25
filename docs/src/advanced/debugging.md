# Debugging and log verbosity

`novops` is a Rust compiled binary. You can use environment variable to set logging level and enable tracing:

```sh
# Set debug level for all rust modules
export RUST_LOG=debug # or other level :info, warn, error

# Enable debug for novops only
export RUST_LOG=novops=debug
```

Show stack traces on error:

```sh
export RUST_BACKTRACE=1
# or 
export RUST_BACKTRACE=full
```

See [Rust Logging configuration](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html) and [Rust Error Handling](https://rust-lang-nursery.github.io/rust-cookbook/errors/handle.html?highlight=rust_back#obtain-backtrace-of-complex-error-scenarios). 