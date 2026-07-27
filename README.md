# cliptown-cli

Rust CLI for ClipTown. Command-line flags are normalized by [`ORESoftware/flags-2-env`](https://github.com/ORESoftware/flags-2-env) using `.cli-flags.toml`; the Rust core consumes the resulting environment contract.

```bash
bin/cliptown auth login
bin/cliptown clip list --limit=50
bin/cliptown clip add --text='deploy checklist' --pin
bin/cliptown clip search --query='postgres migration'
bin/cliptown sync pull
bin/cliptown doctor
```

The CLI stores refresh/session material in the operating-system keyring. It never writes the account master key or plaintext clip history into config files. Clipboard reads require an explicit `clip add --from-clipboard` command.
