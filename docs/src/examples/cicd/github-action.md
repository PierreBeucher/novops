# GitHub Action

Considering your repository has a `.novops.yml` at root, configure a job such as:

```yaml
jobs:
  job_with_novops_load:
    name: run Novops on GitHub Action job
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: setup Novops
        run: |
          curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops-X64-Linux.zip" -o novops.zip
          unzip novops.zip
          mv novops /usr/local/bin/novops
      
      - name: run Novops
        run: |
          novops load -s .envrc -e dev
          cat .envrc >> "$GITHUB_ENV"
      
      - name: a step with loaded novops environment
        run: env | grep MY_APP_HOST
```

Novops loaded values are appended to `$GITHUB_ENV` file as documented in [Setting environment variables](https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-an-environment-variable). This allow novops values to be passed across job's steps.

Alternatively, set a specific version:

```yaml
- name: setup Novops
  env:
    NOVOPS_VERSION: 0.6.0
  run: |
    curl -L "https://github.com/PierreBeucher/novops/releases/download/v${NOVOPS_VERSION}/novops-X64-Linux.zip" -o novops.zip
    unzip novops.zip
    sudo mv novops /usr/local/bin/novops
```

_Note: roadmap includes a GitHub action to ease setup_