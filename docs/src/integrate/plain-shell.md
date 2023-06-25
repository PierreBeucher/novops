# Plain shell (sh, bash, zsh...)

Ensure [Novops is installed](../install.md) and configure it:

## With `direnv`

[`direnv`](https://direnv.net/) usage is recommended for security and usability.

[Setup `direnv` for your shell](https://direnv.net/docs/hook.html) and create a `.envrc` file at the root of your Git repository:

```sh
eval "novops load -s .env.tmp && source .env.tmp && rm .env.tmp"
```

`direnv` will take care of loading/unloading values as you `cd` in and out of your project's directory.

### Why is `direnv` recommended?

[`direnv`](https://direnv.net/) usage is recommended for security and usability:
- Novops provide a sourceable environment variable file, but does not provide load/unload out of the box. `direnv` helps load Novops config seamlessly.
- `direnv` will prevent unwanted variables or secrets to remain in active environment after loading Novops config. 

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

## Without `direnv`

Run command to load Novops:

```sh
novops load -e dev -s .env && source .env
```

You can also setup an alias such as:

```sh
alias nload="novops load -s .env && source .env"
```