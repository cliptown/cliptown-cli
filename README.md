# cliptown-cli

Rust CLI for ClipTown. Command-line flags are normalized by [`ORESoftware/flags-2-env`](https://github.com/ORESoftware/flags-2-env) using `.cli-flags.toml`; the Rust core consumes the resulting environment contract.

```bash
bin/cliptown auth login
bin/cliptown clip list --limit=50
bin/cliptown clip add --stdin --pin < deploy-checklist.txt
bin/cliptown clip add --file deploy-checklist.txt
bin/cliptown clip search --query='postgres migration'
bin/cliptown sync pull
bin/cliptown --json doctor
```

Clip payload text is intentionally not accepted as a command-line flag, where it
could remain in shell history or process listings. Use `--stdin`, `--file`, or
the explicit `--from-clipboard` mode; exactly one input source is required.
Successful `--json` responses use a versioned `{schema_version, ok, result}`
envelope. Errors use `{schema_version, ok, error}` on standard error and stable
exit codes: 2 for invalid arguments, 3 for configuration, 4 for clipboard
access, 5 for local I/O, and 6 for client/service failures.
The machine-readable contract is published as
[`schemas/cli-envelope.schema.json`](schemas/cli-envelope.schema.json).

The CLI stores refresh/session material in the operating-system keyring. It never writes the account master key or plaintext clip history into config files. Clipboard reads require an explicit `clip add --from-clipboard` command.

CI resolves the merged `main` branches of `cliptown-clients` and `cliptown-interfaces`, validates one cross-platform `Cargo.lock` on Linux, macOS, and Windows, and audits the default project `.cli-flags.toml` through the pinned `flags2env` source build. The compatibility run includes the additive DEN-42 device, recovery, and encrypted-object interface models without enabling unfinished auth or cryptographic operations.
