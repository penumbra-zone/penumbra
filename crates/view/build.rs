use vergen::{vergen, Config};

fn main() {
    vergen(Config::default()).expect("able to instantiate vergen");

    // Changes to the SQL schema should trigger a rebuild:
    println!("cargo:rerun-if-changed=src/storage/schema.sql");
}
