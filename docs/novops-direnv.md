# Why is Novops + direnv strongly advised

- Novops provide a sourceable environment variable file, but does not provide facility to load/unload
- [`direnv`](https://direnv.net/) does provide facility to load/unload in current shell

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