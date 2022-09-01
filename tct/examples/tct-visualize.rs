#![recursion_limit = "256"]

use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

use anyhow::Result;
use clap::Parser;
use rand::Rng;

use penumbra_tct::{Commitment, Tree, Witness};

/// Visualize the structure of the Tiered Commitment Tree.
#[derive(Parser, Debug)]
struct Args {
    /// The number of epochs to simulate.
    #[clap(long, default_value = "8")]
    epochs: u16,
    /// The probability that any commitment is remembered.
    #[clap(long, default_value = "0.1")]
    remember: f64,
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
    /// Don't write DOT files.
    #[clap(long)]
    no_dot: bool,
    /// Only write the final tree, not the intermediate stages.
    #[clap(long)]
    only_final: bool,
    /// Force the root every iteration.
    #[clap(long)]
    strict: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut rng = rand::thread_rng();
    let schedule = schedule(&args);

    let mut tree = Tree::new();

    for ((epoch, block), count) in schedule {
        // Finish all empty blocks and epochs leading up to this one
        while (epoch, block)
            > (
                tree.position().unwrap().epoch(),
                tree.position().unwrap().block(),
            )
        {
            if !args.only_final {
                write_to_file(&tree, &args)?;
            }
            tree.end_block()?;
            if tree.position().unwrap().block() == args.epoch_size {
                tree.end_epoch()?;
            }
        }
        assert_eq!(epoch, tree.position().unwrap().epoch());
        assert_eq!(block, tree.position().unwrap().block());
        assert!(count > 0);

        for _ in 0..count {
            if !args.only_final {
                write_to_file(&tree, &args)?;
            }
            // Insert a random commitment into the tree with probability `args.remember` that it
            // will be kept when inserted
            let witness = if rng.gen_range(0.0..=1.0) < args.remember {
                Witness::Keep
            } else {
                Witness::Forget
            };
            tree.insert(witness, gen_commitment())?;
        }
    }

    // Write the final tree
    write_to_file(&tree, &args)?;

    Ok(())
}

/// Generate a Poisson distribution across the blocks to be simulated, binned by block.
fn schedule(args: &Args) -> BTreeMap<(u16, u16), u16> {
    let mut rng = rand::thread_rng();
    let mut schedule = BTreeMap::new();

    let total_commitments = args.epochs * args.epoch_size * args.block_mean;

    for _ in 0..total_commitments {
        let epoch = rng.gen_range(0u16..args.epochs);
        let block = rng.gen_range(0u16..args.epoch_size);
        *schedule.entry((epoch, block)).or_insert(0) += 1;
    }

    schedule
}

fn write_to_file(tree: &Tree, args: &Args) -> Result<()> {
    let position = tree.position().unwrap();

    // Evaluate the root if in strict mode
    if args.strict {
        tree.root();
    }

    println!(
        "visualizing epoch {}, block {}, commitment {} ...",
        position.epoch(),
        position.block(),
        position.commitment()
    );

    let base_path = args.output.join(format!(
        "{:0>5}-{:0>5}-{:0>5}",
        position.epoch(),
        position.block(),
        position.commitment(),
    ));

    let svg_path = base_path.with_extension("svg");
    let dot_path = base_path.with_extension("dot");

    if args.no_svg {
        if !args.no_dot {
            // Serialize the dot representation directly to the dot file
            println!("Writing {} ...", dot_path.display());
            let mut dot_file = File::create(dot_path)?;
            tree.render_dot(&mut dot_file)?;
        }
    } else if args.no_dot {
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
    let mut child = Command::new("dot")
        .args(&["-Tsvg"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    thread::scope(|scope| {
        let render_thread = scope.spawn(move || {
            stdin.write_all(dot)?;
            stdin.flush()?;
            Ok::<_, io::Error>(())
        });
        io::copy(&mut stdout, writer)?;
        render_thread.join().unwrap()?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

fn write_svg_direct<W: Write>(tree: &Tree, writer: &mut W) -> Result<()> {
    let mut child = Command::new("dot")
        .args(&["-Tsvg"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    thread::scope(|scope| {
        let render_thread = scope.spawn(move || {
            tree.render_dot(&mut stdin)?;
            stdin.flush()?;
            Ok::<_, io::Error>(())
        });
        io::copy(&mut stdout, writer)?;
        render_thread.join().unwrap()?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

/// Generate a random valid commitment by rejection sampling
fn gen_commitment() -> Commitment {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];

    loop {
        rng.fill(&mut bytes);
        if let Ok(commitment) = Commitment::try_from(bytes) {
            return commitment;
        }
    }
}
