use anyhow::Context;
use std::io::Read;

fn main() -> anyhow::Result<()> {
    let f = "src/gen/proto_descriptor.bin";
    check_file_is_not_lfs_pointer(f)?;
    Ok(())
}

// Support for gRPC server reflection requires that we load
// a `proto_descriptor.bin` file at build time. We store that file
// in git-lfs, so if git-lfs isn't installed, it'll be a plaintext pointer
// file, rather than a larger binary blob. Let's just check for a filesize
// greater than ~500bytes, that's a good enough check for whether the file
// has been properly checked out from lfs.
pub fn check_file_is_not_lfs_pointer(file: &str) -> anyhow::Result<()> {
    let mut bytes = Vec::new();
    {
        let f = std::fs::File::open(file).with_context(|| "can open proto descriptor file")?;
        let mut reader = std::io::BufReader::new(f);
        reader
            .read_to_end(&mut bytes)
            .with_context(|| "can read proto descriptor file")?;
    }
    // We expect ~300Kb for the descriptor file, so 500b is a conservative minimum.
    if bytes.len() < 500 {
        let msg = format!(
            "Error: the protobuf descriptor file is too small.
Check that you have git-lfs installed, then run:

   git lfs fetch
   git lfs checkout

and retry the build."
        );

        return Err(anyhow::anyhow!(msg));
    }
    return Ok(());
}
