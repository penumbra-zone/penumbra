use anyhow::anyhow;
use ibc_proto::ibc::core::client::v1::Height;

pub(crate) fn height_from_str(value: &str) -> anyhow::Result<Height> {
    let split: Vec<&str> = value.split('-').collect();

    if split.len() != 2 {
        return Err(anyhow!("invalid height string"));
    }

    let revision_number = split[0]
        .parse::<u64>()
        .map_err(|e| anyhow!("failed to parse revision number"))?;

    let revision_height = split[1]
        .parse::<u64>()
        .map_err(|e| anyhow!("failed to parse revision height"))?;

    if revision_number == 0 && revision_height == 0 {
        return Err(anyhow!("height is zero"));
    }

    Ok(Height {
        revision_number,
        revision_height,
    })
}
