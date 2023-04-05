use vergen::{vergen, Config};

fn main() -> anyhow::Result<()> {
    vergen(Config::default()).unwrap();
    Ok(())
}
