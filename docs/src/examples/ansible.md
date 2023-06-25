# Ansible

A typical Ansible workflow requires developers and CI to provide per environment:

- Ansible inventory
- Private SSH key to connect on target hosts
- Ansible Vault password

Instead of duplicating configs and spreading secrets on CI and your local environment, you can use Novops and [Ansible built-in variables](https://docs.ansible.com/ansible/latest/reference_appendices/config.html#environment-variables).

Your workflow will then look like:

```sh
novops load -s .envrc && source .envrc
# Select environment: dev, prod (default: dev)

# No need to specify inventory, vault password or ssh key
# They've all been loaded as environment variables and files
ansible-playbook my-playbook.yml
```

Create a `.novops.yml` such as:

```yaml
environments:

  dev:

    variables:
      # Comma separated list of Ansible inventory sources
      # Ansible will automatically use these inventories
      - name: ANSIBLE_INVENTORY
        value: inventories/dev
      
      # Add more as needed
      # - name: ANSIBLE_*
      #   value: ...

    files:
      # Built-in variable for Ansible secret ssh key
      # Ansible will use this key to connect via SSH on hosts
      # Load from some external source
      - variable: ANSIBLE_PRIVATE_KEY_FILE
        content: 
          hvault_kv2:
            path: myapp/dev
            key: ssh_key

      # Built-in variable for Ansible Vault password file
      # Ansible will automatically use this file to decrypt vault
      # Load from some external source
      - variable: ANSIBLE_VAULT_PASSWORD_FILE
        content: 
          hvault_kv2:
            path: myapp/dev
            key: inventory_password
  
  # Another environment
  prod:
    variables:
      - name: ANSIBLE_INVENTORY
        value: inventories/prod
    files:
      - variable: ANSIBLE_PRIVATE_KEY_FILE
        content: 
          hvault_kv2:
            path: myapp/prod
            key: ssh_key
      - variable: ANSIBLE_VAULT_PASSWORD_FILE
        content: 
          hvault_kv2:
            path: myapp/prod
            key: inventory_password
```

Your workflow will now be the same on CI or for local development, and you'll be able to switch environments seamlessly !