use ibc_types::core::connection::Version;

/// Selects a version from the intersection of locally supported and counterparty versions.
pub fn pick_connection_version(
    supported_versions: &[Version],
    counterparty_versions: &[Version],
) -> anyhow::Result<Version> {
    let mut intersection: Vec<Version> = Vec::new();
    for s in supported_versions.iter() {
        for c in counterparty_versions.iter() {
            if c.identifier != s.identifier {
                continue;
            }
            for feature in c.features.iter() {
                if feature.trim().is_empty() {
                    return Err(anyhow::anyhow!("freatures cannot be empty"));
                }
            }
            intersection.append(&mut vec![s.clone()]);
        }
    }
    intersection.sort_by(|a, b| a.identifier.cmp(&b.identifier));
    if intersection.is_empty() {
        return Err(anyhow::anyhow!("no common version"));
    }
    Ok(intersection[0].clone())
}
