# Plain shell (sh, bash, zsh...)

For local usage with plain shell, [`direnv`](https://direnv.net/) usage is recommended for security reasons:
- Novops provide a sourceable environment variable file, but does not provide facility to load/unload. You can use `source` but...
- [`direnv`](https://direnv.net/) does provide a better way to load/unload variables in current shell as it will prevent unwanted variables or secrets to remain in active environment

If you use Novops without `direnv`, some variable may not be unloaded properly when switching context, for example:

```sh
novops load -e prod -s .env && source .env
# Do something in prod...
# ...

# In the same shell, move somewhere else
cd ../my-other-project
novops load -e dev -s .env && source .env
# Some prod variable from previous context may still be loaded!
# Direnv would have automatically loaded/unloaded env for you
```

Instead, with `direnv` installed, you'll just have to run:

```sh
# Generate env file at .envrc
# direnv will automatically load it
novops load -s .envrc
```