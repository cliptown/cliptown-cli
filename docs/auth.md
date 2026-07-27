# CLI authentication

The CLI opens the system browser for Supabase PKCE. Supabase may delegate step-up authentication to 3FA and shared-auth. The final access token must carry a device identifier and authentication-method references (`amr`). A six-digit PIN is entered only into a trusted local/3FA surface and is never placed on the CLI command line, where shell history and process listings could expose it.
