use vergen::{vergen, Config};

fn main() {
    vergen(Config::default()).unwrap();
    println!("cargo:rerun-if-changed=migrations");
}
