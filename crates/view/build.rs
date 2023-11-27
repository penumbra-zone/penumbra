fn main() {
    // Changes to the SQL schema should trigger a rebuild:
    println!("cargo:rerun-if-changed=src/storage/schema.sql");
}
