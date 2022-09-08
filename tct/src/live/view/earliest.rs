use crate::index;

use super::*;

/// Query parameter used in the [`view`] endpoint to specify the earliest version of a tree to
/// return (otherwise the query long-polls until it is available).
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
pub struct DotQuery {
    #[serde(default)]
    epoch: u16,
    #[serde(default)]
    block: u16,
    #[serde(default)]
    commitment: u16,
    /// If `true`, override the specified position by forcing the position to be interpreted as
    /// being full.
    #[serde(default)]
    full: bool,
    #[serde(default)]
    forgotten: Forgotten,
    /// If `true`, force the next thing to be returned to be greater than either the position or
    /// forgotten index specified (it doesn't matter which).
    #[serde(default)]
    pub next: bool,
    /// If `false`, don't return a rendered graph, just send back the other parameters.
    #[serde(default = "default_true")]
    pub graph: bool,
}

fn default_true() -> bool {
    true
}

impl DotQuery {
    pub fn position(&self) -> Option<Position> {
        if self.full {
            return None;
        }

        Some(
            u64::from(index::within::Tree {
                epoch: self.epoch.into(),
                block: self.block.into(),
                commitment: self.commitment.into(),
            })
            .into(),
        )
    }

    pub fn not_too_late_for(&self, tree: &Tree) -> bool {
        let position = if let Some(position) = tree.position() {
            position
        } else {
            // If there is no position, the tree is full, so the only way to be earlier than the
            // tree is for the forgotten index to be earlier
            return if self.next {
                self.forgotten < tree.forgotten()
            } else {
                self.forgotten <= tree.forgotten()
            };
        };

        // Otherwise, one of the forgotten index or the position must be earlier (strictly
        // earlier if the next parameter is specified)
        #[allow(clippy::collapsible_else_if)]
        if let Some(earliest_position) = self.position() {
            if self.next {
                earliest_position < position || self.forgotten < tree.forgotten()
            } else {
                earliest_position <= position || self.forgotten <= tree.forgotten()
            }
        } else {
            if self.next {
                self.forgotten < tree.forgotten()
            } else {
                self.forgotten <= tree.forgotten()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_earliest() {
        let parsed = serde_urlencoded::from_str::<DotQuery>(
            "epoch=1&block=2&commitment=3&forgotten=4&next=true&graph=false",
        )
        .unwrap();
        assert_eq!(
            parsed,
            DotQuery {
                epoch: 1,
                block: 2,
                commitment: 3,
                forgotten: 4.into(),
                full: false,
                next: true,
                graph: false,
            }
        );
    }
}
