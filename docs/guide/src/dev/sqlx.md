# SQLite compilation setup

The view server uses SQLite via `sqlx` as its backing store.  The type-safe
query macros require compile-time information about the database schemas.
Normally, this information is cached in the crate's `sqlx-data.json`, and
nothing extra is required to build.

However, when editing the view server's database code, it's necessary to work
with a development database:

1. You'll need `sqlx-cli` installed with the correct features:
`cargo install sqlx-cli --features sqlite`
2. The database structure is defined in the `migrations/` directory of the
`view` crate.
3. Set the `DATABASE_URL` environment variable to point to the SQLite location.
   For instance,

    ```shell
    export DATABASE_URL="sqlite:///tmp/pclientd-dev-db.sqlite"
    ```

    will set the shell environment variable to the same one set in the project's
    `.vscode/settings.json`.
4. From the `view` directory, run `cargo sqlx database setup` to create
the database and run migrations.
5. From the `view` directory, run
`cargo sqlx prepare -- --lib`
to regenerate the `sqlx-data.json` file that allows offline compilation.
