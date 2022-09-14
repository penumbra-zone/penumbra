#![recursion_limit = "256"]

use std::{
    collections::BTreeMap,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
    thread,
};

use anyhow::Result;
use clap::Parser;
use rand::{seq::SliceRandom, Rng, RngCore, SeedableRng};
use rand_distr::Binomial;

use penumbra_tct::{self as tct, Commitment, Tree, Witness};
use regex::Regex;
use tct::structure::Hash;

/// Visualize the structure of the Tiered Commitment Tree.
#[derive(Parser, Debug)]
struct Args {
    /// The number of epochs to simulate.
    #[clap(long, default_value = "8")]
    epochs: u16,
    /// The probability that any commitment is remembered.
    #[clap(long, default_value = "0.1")]
    remember: f64,
    /// The probability that any remebered commitment is forgotten by the end.
    #[clap(long, default_value = "0.0")]
    forget: f64,
    /// The mean number of commitments in each block.
    #[clap(long, default_value = "8")]
    block_mean: u16,
    /// The number of blocks in each epoch.
    #[clap(long, default_value = "8")]
    epoch_size: u16,
    /// The directory to place the output SVG files.
    #[clap(long)]
    output: PathBuf,
    /// Don't write SVG files.
    #[clap(long)]
    no_svg: bool,
    /// Write intermediate DOT files (slower than just going directly to SVG).
    #[clap(long)]
    dot: bool,
    /// Only write the final tree, not the intermediate stages.
    #[clap(long)]
    only_final: bool,
    /// Force the root every iteration.
    #[clap(long)]
    strict: bool,
    /// Use the given seed for random generation.
    #[clap(long)]
    seed: Option<Seed>,
    /// Accelerate tree construction, skipping frames for epochs and blocks with no remembered commitments.
    #[clap(long)]
    fast: bool,
}

#[derive(Debug)]
struct Seed([u8; 32]);

impl FromStr for Seed {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Seed(
            hex::decode(s)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("seed is not 32 bytes"))?,
        ))
    }
}

impl Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    let mut rng = if let Some(Seed(seed)) = args.seed {
        rand_chacha::ChaCha20Rng::from_seed(seed)
    } else {
        let mut seed = [0; 32];
        rand::rngs::OsRng.fill_bytes(&mut seed);
        println!("Using random seed: {}", Seed(seed));
        rand_chacha::ChaCha20Rng::from_seed(seed)
    };

    println!("Simulating {} epochs of activity ...", args.epochs);
    let schedule = schedule(&mut rng, &args);

    println!("Generating tree states and writing files ...");
    let mut tree = Tree::new();

    for epoch in 0..args.epochs {
        if let Some(epoch) = schedule.get(&epoch) {
            // (count, kept)
            let mut epoch_schedule = BTreeMap::<u16, (u16, u16)>::new();
            let mut total_kept = 0;
            let mut total_count = 0;
            for (&block, &count) in epoch {
                let kept = rng.sample(Binomial::new(count.into(), args.remember).unwrap());
                total_kept += kept;
                total_count += count;
                epoch_schedule.insert(block, (count, kept.try_into().unwrap()));
            }

            // Shortcut when going fast and the epoch won't contain anything we'll remember
            if args.fast && total_kept == 0 {
                write_to_file(&tree, &args)?;

                if total_count == 0 {
                    // If the total count in the epoch is zero, it's the empty epoch
                    tree.end_epoch().unwrap();
                } else {
                    // Otherwise it's some random non-empty epoch
                    tree.insert_epoch(tct::builder::epoch::Root(gen_hash(&mut rng)))
                        .unwrap();
                }

                // Skip doing anything more for this epoch
                continue;
            }

            // For each block, generate its contents
            for block in 0..args.epoch_size {
                let (count, kept) = *epoch_schedule.get(&block).unwrap_or(&(0, 0));

                if count == 0 {
                    // End the block immediately because the count is zero
                    write_to_file(&tree, &args)?;
                    tree.end_block().unwrap();
                } else if args.fast && kept == 0 {
                    // Shortcut when going fast and the block won't contain anything we'll remember
                    write_to_file(&tree, &args)?;
                    tree.insert_block(tct::builder::block::Root(gen_hash(&mut rng)))
                        .unwrap();
                } else {
                    // Otherwise we should construct the block explicitly, keeping as many commitments
                    // as previously randomly decided
                    let mut witnesses = Vec::new();
                    for _ in 0..kept {
                        witnesses.push(Witness::Keep);
                    }
                    for _ in kept..count {
                        witnesses.push(Witness::Forget);
                    }
                    witnesses.shuffle(&mut rng);
                    for witness in witnesses {
                        write_to_file(&tree, &args)?;
                        tree.insert(witness, gen_commitment(&mut rng)).unwrap();
                    }

                    // End the block now we're done with it
                    write_to_file(&tree, &args)?;
                    tree.end_block().unwrap();
                }
            }
        }

        // End the epoch now we're done with it
        write_to_file(&tree, &args)?;
        tree.end_epoch().unwrap();
    }

    // Write the final tree
    args.only_final = false;
    write_to_file(&tree, &args)?;

    Ok(())
}

/// Generate a Poisson distribution across the blocks to be simulated, binned by block.
fn schedule<R: Rng>(rng: &mut R, args: &Args) -> BTreeMap<u16, BTreeMap<u16, u16>> {
    let mut schedule: BTreeMap<u16, BTreeMap<u16, u16>> = BTreeMap::new();

    let total_commitments: u64 =
        args.epochs as u64 * args.epoch_size as u64 * args.block_mean as u64;

    for _ in 0..total_commitments {
        let epoch = rng.gen_range(0..args.epochs);
        let block = rng.gen_range(0..args.epoch_size);
        *schedule.entry(epoch).or_default().entry(block).or_default() += 1;
    }

    schedule
}

fn write_to_file(tree: &Tree, args: &Args) -> Result<()> {
    if args.only_final {
        return Ok(());
    }

    let position = tree.position().unwrap();

    // Evaluate the root if in strict mode
    if args.strict {
        tree.root();
    }

    let base_path = args.output.join(format!(
        "{:0>5}-{:0>5}-{:0>5}",
        position.epoch(),
        position.block(),
        position.commitment(),
    ));

    let svg_path = base_path.with_extension("svg");
    let dot_path = base_path.with_extension("dot");

    if args.no_svg {
        if args.dot {
            // Serialize the dot representation directly to the dot file
            println!("Writing {} ...", dot_path.display());
            let mut dot_file = File::create(dot_path)?;
            tree.render_dot(&mut dot_file)?;
        }
    } else if !args.dot {
        // Serialize the dot representation directly into the dot subprocess
        println!("Writing {} ...", svg_path.display());
        let mut svg_file = File::create(svg_path)?;
        write_svg_direct(tree, &mut svg_file)?;
    } else {
        // Allocate an intermediate dot file in memory
        let mut dot = Vec::new();
        tree.render_dot(&mut dot)?;

        // Write the dot file
        println!("Writing {} ...", dot_path.display());
        let mut dot_file = File::create(dot_path)?;
        dot_file.write_all(&dot)?;

        // Generate an svg from the dot file
        println!("Writing {} ...", svg_path.display());
        let mut svg_file = File::create(svg_path)?;
        write_svg(&dot, &mut svg_file)?;
    }

    Ok(())
}

fn write_svg<W: Write>(dot: &[u8], writer: &mut W) -> Result<()> {
    let (mut stdin, mut stdout) = dot_command()?;
    thread::scope(|scope| {
        let render_thread = scope.spawn(move || {
            stdin.write_all(dot)?;
            stdin.flush()?;
            Ok::<_, io::Error>(())
        });
        add_keys(&mut stdout, writer)?;
        render_thread.join().unwrap()?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

fn write_svg_direct<W: Write>(tree: &Tree, writer: &mut W) -> Result<()> {
    let (mut stdin, mut stdout) = dot_command()?;
    thread::scope(|scope| {
        let render_thread = scope.spawn(move || {
            tree.render_dot(&mut stdin)?;
            stdin.flush()?;
            Ok::<_, io::Error>(())
        });
        add_keys(&mut stdout, writer)?;
        render_thread.join().unwrap()?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

fn dot_command() -> io::Result<(impl Write, impl Read)> {
    let mut child = Command::new("dot")
        .args(&["-Tsvg"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    Ok((stdin, stdout))
}

fn add_keys<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<()> {
    // Add "key" SVG attributes to anything with an "id" attribute
    let mut out = BufReader::new(reader);
    let mut line = String::new();
    let svg_id = Regex::new(r#"id="([a-zA-Z][a-zA-Z0-9_\-]*)""#).unwrap();
    loop {
        if let 0 = out.read_line(&mut line)? {
            break Ok(());
        }
        write!(writer, "{}", svg_id.replace(&line, "id=\"$1\" key=\"$1\""))?;
        line.clear();
    }
}

/// Generate a random valid commitment by rejection sampling
fn gen_commitment<R: Rng>(rng: &mut R) -> Commitment {
    let mut bytes = [0u8; 32];

    loop {
        rng.fill(&mut bytes);
        if let Ok(commitment) = Commitment::try_from(bytes) {
            return commitment;
        }
    }
}

/// Generate a random valid hash by rejection sampling
fn gen_hash<R: Rng>(rng: &mut R) -> Hash {
    // Rejection sample until finding a valid hash value
    loop {
        let mut bytes = [0; 32];
        rng.fill_bytes(&mut bytes);
        if let Ok(hash) = Hash::from_bytes(bytes) {
            break hash;
        }
    }
}
