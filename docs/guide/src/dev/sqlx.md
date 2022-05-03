# SQLite compilation setup

The wallet server uses SQLite via `sqlx` as its backing store.  The type-safe
query macros require compile-time information about the database schemas.
Normally, this information is cached in the crate's `sqlx-data.json`, and
nothing extra is required to build.

However, when editing the wallet server's database code, it's necessary to work
with a development database:

1.  The database structure is defined in the `migrations/` directory of the
wallet server crate.
2.  Set the `DATABASE_URL` environment variable to point to the SQLite location.
    For instance,
    ```
    export DATABASE_URL="sqlite:///tmp/pwalletd-dev-db.sqlite"
    ```
    will set the shell environment variable to the same one set in the project's
    `.vscode/settings.json`.
3.  From the `wallet-next` directory, run `cargo sqlx database setup` to create
the database and run migrations.
4.  From the `wallet-next` directory, run
`cargo sqlx prepare -- --lib=penumbra-wallet-next`
to regenerate the `sqlx-data.json` file that allows offline compilation.
