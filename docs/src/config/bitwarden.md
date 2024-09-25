# BitWarden

## Authentication & Configuration

To use BitWarden module:

- Ensure BitWarden CLI `bw` is available in the same context `novops` runs in
- Set environment variable `BW_SESSION_TOKEN`

```yaml
environments:
  dev:
    files: 
      - variable: ssh-key
        content:
          bitwarden:
            # Name of the entry to load
            entry: Some SSH Key entry
            # Field to read from BitWarden objects. Maps directly to JSON field from 'bw get item' command
            # See below for details
            field: notes
```

Novops will load items using `bw get item` as JSON. `field` must be set to expected field. Separate sub-field with `.`. Examples:

- Secure Note item
  ```yaml
  field: notes
  ```
- Login item
  ```yaml
  field: login.username
  field: login.password
  field: login.totp
  ```
- Identity item:
  ```yaml
  field: identity.title
  field: identity.firstName
  # field: identity.xxx
  ```
- Card item:
  ```yaml
  field: card.cardholderName
  field: card.number
  field: card.expMonth
  field: card.expYear
  field: card.code
  field: card.brand 
  ```

To get full output from BitWarden, use `bw get`or `bw get template`
