# Ansible

Leverage [Ansible built-in environments variables](https://docs.ansible.com/ansible/latest/reference_appendices/config.html#environment-variables) to setup your environments, e.g:

- `ANSIBLE_PRIVATE_KEY_FILE` - SSH key used to connect on managed hosts
- `ANSIBLE_VAULT_PASSWORD_FILE` - Path to Ansible vault password
- `ANSIBLE_INVENTORY` - Inventory to use

Your workflow will look like

```sh
# Inventory, vault passphrase and SSH keys 
# are set by environment variables
novops run -- ansible-playbook my-playbook.yml
```

Use a `.novops.yml` such as:

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
      # Ansible will use this key to connect via SSH on managed hosts
      - variable: ANSIBLE_PRIVATE_KEY_FILE
        content: 
          hvault_kv2:
            path: myapp/dev
            key: ssh_key

      # Ansible use this file to decrypt local Ansible vault
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

