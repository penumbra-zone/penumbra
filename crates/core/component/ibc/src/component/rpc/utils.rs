use std::str::FromStr;

use anyhow::bail;
use anyhow::Context as _;
use cnidarium::Snapshot;
use cnidarium::Storage;
use ibc_proto::ibc::core::client::v1::Height;

pub(in crate::component::rpc) fn determine_snapshot_from_height_header<T>(
    storage: Storage,
    request: &tonic::Request<T>,
) -> anyhow::Result<Snapshot> {
    let height_entry = request
        .metadata()
        .get("height")
        .context("no `height` header")?
        .to_str()
        .context("value of `height` header was not ASCII")?;

    'state: {
        let height = match parse_as_ibc_height(height_entry)
            .context("failed to parse value as IBC height")
        {
            Err(err) => break 'state Err(err),
            Ok(height) => height.revision_height,
        };
        if height == 0 {
            Ok(storage.latest_snapshot())
        } else {
            storage
                .snapshot(height)
                .with_context(|| format!("could not open snapshot at revision height `{height}`"))
        }
    }
    .with_context(|| {
        format!("failed determine snapshot from `\"height\": \"{height_entry}` header")
    })
}

/// Utility to implement
#[derive(Debug)]
struct TheHeight(Height);

impl TheHeight {
    fn zero() -> Self {
        Self(Height {
            revision_number: 0,
            revision_height: 0,
        })
    }
    fn into_inner(self) -> Height {
        self.0
    }
}

impl FromStr for TheHeight {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        const FORM: &str = "input was not of the form '0' or '<number>-<height>'";

        let mut parts = input.split('-');

        let revision_number = parts
            .next()
            .context(FORM)?
            .parse::<u64>()
            .context("failed to parse revision number as u64")?;
        let revision_height = match parts.next() {
            None if revision_number == 0 => return Ok(Self::zero()),
            None => bail!(FORM),
            Some(rev_height) => rev_height
                .parse::<u64>()
                .context("failed to parse revision height as u64")?,
        };

        Ok(TheHeight(Height {
            revision_number,
            revision_height,
        }))
    }
}

fn parse_as_ibc_height(input: &str) -> anyhow::Result<Height> {
    let height = input
        .trim()
        .parse::<TheHeight>()
        .context("failed to parse as IBC height")?
        .into_inner();

    Ok(height)
}

#[cfg(test)]
mod tests {
    use ibc_proto::ibc::core::client::v1::Height;

    use super::TheHeight;

    fn zero() -> Height {
        Height {
            revision_number: 0,
            revision_height: 0,
        }
    }

    fn height(revision_number: u64, revision_height: u64) -> Height {
        Height {
            revision_number,
            revision_height,
        }
    }

    #[track_caller]
    fn assert_ibc_height_is_parsed_correctly(input: &str, expected: Height) {
        let actual = input.parse::<TheHeight>().unwrap().into_inner();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_ibc_height() {
        assert_ibc_height_is_parsed_correctly("0", zero());
        assert_ibc_height_is_parsed_correctly("0-0", zero());
        assert_ibc_height_is_parsed_correctly("0-1", height(0, 1));
        assert_ibc_height_is_parsed_correctly("1-0", height(1, 0));
        assert_ibc_height_is_parsed_correctly("1-1", height(1, 1));
    }
}
