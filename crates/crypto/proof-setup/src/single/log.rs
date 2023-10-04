/// The number of bytes in a contribution hash.
pub const CONTRIBUTION_HASH_SIZE: usize = 32;

/// Represents the hash of a contribution.
///
/// This is also used as the output of hashing CRS elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContributionHash(pub [u8; CONTRIBUTION_HASH_SIZE]);

impl ContributionHash {
    pub(crate) fn dummy() -> Self {
        Self([0x1; CONTRIBUTION_HASH_SIZE])
    }
}

impl AsRef<[u8]> for ContributionHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for ContributionHash {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != CONTRIBUTION_HASH_SIZE {
            anyhow::bail!(
                "Failed to read ContributionHash from slice of len {}",
                value.len()
            );
        }
        Ok(Self(value.try_into()?))
    }
}

/// Represents an item that can be hashed.
pub trait Hashable {
    fn hash(&self) -> ContributionHash;
}

/// A trait abstracting common behavior between both setup phases.
///
/// This trait provides us with enough functionality to verify the contribution
/// logs produced by both phases.
pub trait Phase {
    /// The type of the elements constituting this phase.
    ///
    /// This elements will be internally valid, although possibly not correctly linked
    /// with respect to other contributions.
    type CRSElements: Hashable;
    /// The type of a contribution before any kind of validation.
    type RawContribution: Hashable;
    /// A contribution after having been validated.
    type Contribution: Hashable;

    /// The hash of the parent element for a raw contribution.
    fn parent_hash(contribution: &Self::RawContribution) -> ContributionHash;

    /// The elements in a contribution.
    fn elements(contribution: &Self::Contribution) -> &Self::CRSElements;

    /// Validate a contribution relative to some root elements.
    ///
    /// This might fail, hence the Option.
    fn validate(
        root: &Self::CRSElements,
        contribution: &Self::RawContribution,
    ) -> Option<Self::Contribution>;

    /// Check if a contribution is linked to some parent elements.
    ///
    /// If a contribution is linked to those elements, then it builds upon those elements
    /// correctly.
    fn is_linked_to(contribution: &Self::Contribution, elements: &Self::CRSElements) -> bool;
}

/// A log of contributions.
///
/// This is just an ordered list of contributions, which should usually
/// contain all contributions, starting from some root elements.
pub struct ContributionLog<P: Phase> {
    contributions: Vec<P::RawContribution>,
}

impl<P: Phase> ContributionLog<P> {
    pub fn new(contributions: impl IntoIterator<Item = P::RawContribution>) -> Self {
        Self {
            contributions: contributions.into_iter().collect(),
        }
    }

    /// Scan this log for the most recent valid contribution.
    ///
    /// This requires the root elements which these contributions all build upon.
    ///
    /// This might fail to find any good contributions, in which case None will be returned.
    pub fn last_good_contribution(
        &self,
        root: &P::CRSElements,
        checkpoint: Option<ContributionHash>,
    ) -> Option<P::Contribution> {
        // We start by assuming that the root is correct, and the first good elements
        // the log should start building on. From there, we keep considering new contributions,
        // each of which can become the most recent good contribution if all conditions are met.
        // By the end, we'll have the last good contribution for the log as a whole.
        let root_hash = root.hash();
        // By default, we start with 0, and no last known good contribution
        let mut start = 0;
        let mut last_good: Option<P::Contribution> = None;
        // If we have a checkpoint, we might want to start at a different place, however:
        if let Some(checkpoint) = checkpoint {
            if let Some((i, matching)) = self
                .contributions
                .iter()
                .enumerate()
                .find(|(_, c)| c.hash() == checkpoint)
            {
                if let Some(valid) = P::validate(root, matching) {
                    start = i + 1;
                    last_good = Some(valid);
                }
            }
        }

        for contribution in &self.contributions[start..] {
            // 1. Check if the parent we're building off of is the last good contribution.
            let expected_parent_hash = last_good.as_ref().map(|c| c.hash()).unwrap_or(root_hash);
            if P::parent_hash(contribution) != expected_parent_hash {
                continue;
            }
            // 2. Check if this contribution is internally valid.
            let contribution = match P::validate(root, contribution) {
                None => continue,
                Some(c) => c,
            };
            // 3. Check if we're linked to the parent elements.
            let linked = match last_good.as_ref() {
                None => P::is_linked_to(&contribution, root),
                Some(c) => P::is_linked_to(&contribution, P::elements(c)),
            };
            if !linked {
                continue;
            }
            // 4. At this point, this is the next good contribution
            last_good = Some(contribution)
        }

        last_good
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn hash_from_u8(x: u8) -> ContributionHash {
        let mut out = [0u8; CONTRIBUTION_HASH_SIZE];
        out[0] = x;
        ContributionHash(out)
    }

    // Our strategy here is to have a dummy phase setup where the validity
    // and linkedness of elements is self-evident.

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct DummyElements {
        id: u8,
    }

    impl Hashable for DummyElements {
        fn hash(&self) -> ContributionHash {
            hash_from_u8(self.id)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct DummyContribution {
        parent: u8,
        elements: DummyElements,
        valid: bool,
        linked: bool,
    }

    impl Hashable for DummyContribution {
        fn hash(&self) -> ContributionHash {
            self.elements.hash()
        }
    }

    struct DummyPhase;

    impl Phase for DummyPhase {
        type CRSElements = DummyElements;

        type RawContribution = DummyContribution;

        type Contribution = DummyContribution;

        fn parent_hash(contribution: &Self::RawContribution) -> ContributionHash {
            hash_from_u8(contribution.parent)
        }

        fn elements(contribution: &Self::Contribution) -> &Self::CRSElements {
            &contribution.elements
        }

        fn validate(
            _root: &Self::CRSElements,
            contribution: &Self::RawContribution,
        ) -> Option<Self::Contribution> {
            if contribution.valid {
                Some(*contribution)
            } else {
                None
            }
        }

        fn is_linked_to(contribution: &Self::Contribution, _elements: &Self::CRSElements) -> bool {
            contribution.linked
        }
    }

    // Quick contribution creation, since we need many such examples
    macro_rules! contribution {
        ($parent: expr, $id: expr, $valid: expr, $linked: expr) => {
            DummyContribution {
                parent: $parent,
                elements: DummyElements { id: $id },
                valid: $valid,
                linked: $linked,
            }
        };
    }

    #[test]
    fn test_empty_log_search_returns_none() {
        let log: ContributionLog<DummyPhase> = ContributionLog::new(vec![]);
        assert!(log
            .last_good_contribution(&DummyElements { id: 0 }, None)
            .is_none());
    }

    #[test]
    fn test_single_valid_log_search() {
        let log: ContributionLog<DummyPhase> =
            ContributionLog::new(vec![contribution!(0, 1, true, true)]);
        assert_eq!(
            log.last_good_contribution(&DummyElements { id: 0 }, None),
            Some(log.contributions[0])
        );
    }

    #[test]
    fn test_multi_valid_log_search() {
        let log: ContributionLog<DummyPhase> = ContributionLog::new(vec![
            contribution!(0, 1, true, true),
            contribution!(1, 2, true, true),
        ]);
        assert_eq!(
            log.last_good_contribution(&DummyElements { id: 0 }, None),
            Some(log.contributions[1])
        );
    }

    #[test]
    fn test_multi_valid_log_search_bad_root() {
        let log: ContributionLog<DummyPhase> = ContributionLog::new(vec![
            contribution!(0, 1, true, true),
            contribution!(1, 2, true, true),
        ]);
        assert_eq!(
            log.last_good_contribution(&DummyElements { id: 2 }, None),
            None
        );
    }

    #[test]
    fn test_multi_log_with_skips_search() {
        let log: ContributionLog<DummyPhase> = ContributionLog::new(vec![
            contribution!(0, 1, true, true),
            // Invalid
            contribution!(1, 2, false, true),
            // Valid
            contribution!(1, 3, true, true),
            // Valid, but wrong parent
            contribution!(1, 4, true, true),
            // Valid
            contribution!(3, 5, true, true),
            // Bad linking
            contribution!(5, 6, true, false),
            // Valid
            contribution!(5, 7, true, true),
        ]);
        assert_eq!(
            log.last_good_contribution(&DummyElements { id: 0 }, None),
            Some(log.contributions[6])
        );
    }

    #[test]
    fn test_multi_log_with_skips_and_resuming_search() {
        let log: ContributionLog<DummyPhase> = ContributionLog::new(vec![
            contribution!(0, 1, true, true),
            // Invalid
            contribution!(1, 2, false, true),
            // Valid
            contribution!(1, 3, true, true),
            // Valid, but wrong parent
            contribution!(1, 4, true, true),
            // Valid
            contribution!(3, 5, true, true),
            // Bad linking
            contribution!(5, 6, true, false),
            // Valid
            contribution!(5, 7, true, true),
        ]);
        assert_eq!(
            log.last_good_contribution(&DummyElements { id: 0 }, Some(hash_from_u8(5))),
            Some(log.contributions[6]),
        );
    }

    #[test]
    fn test_multi_log_with_skips_and_bad_checkpoint() {
        let log: ContributionLog<DummyPhase> = ContributionLog::new(vec![
            contribution!(0, 1, true, true),
            // Invalid
            contribution!(1, 2, false, true),
            // Valid
            contribution!(1, 3, true, true),
            // Valid, but wrong parent
            contribution!(1, 4, true, true),
            // Valid
            contribution!(3, 5, true, true),
            // Bad linking
            contribution!(5, 6, true, false),
            // Valid
            contribution!(5, 7, true, true),
        ]);
        assert_eq!(
            log.last_good_contribution(&DummyElements { id: 0 }, Some(hash_from_u8(100))),
            Some(log.contributions[6]),
        );
        // Should work because we validate internal consistency
        assert_eq!(
            log.last_good_contribution(&DummyElements { id: 0 }, Some(hash_from_u8(2))),
            Some(log.contributions[6]),
        );
    }
}
