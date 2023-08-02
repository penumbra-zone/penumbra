/// The number of bytes in a contribution hash.
pub const CONTRIBUTION_HASH_SIZE: usize = 32;

/// Represents the hash of a contribution.
///
/// This is also used as the output of hashing CRS elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContributionHash([u8; CONTRIBUTION_HASH_SIZE]);

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
    type RawContribution;
    /// A contribution after having been validated.
    type Contribution: Hashable;

    /// The hash of the parent element for a raw contribution.
    fn parent_hash(contribution: &Self::RawContribution) -> ContributionHash;

    /// The elements in a contribution.
    fn elements(contribution: &Self::Contribution) -> Self::CRSElements;

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
    pub fn last_good_contribution(&self, root: &P::CRSElements) -> Option<P::Contribution> {
        // We start by assuming that the root is correct, and the first good elements
        // the log should start building on. From there, we keep considering new contributions,
        // each of which can become the most recent good contribution if all conditions are met.
        // By the end, we'll have the last good contribution for the log as a whole.
        let root_hash = root.hash();
        let mut last_good: Option<P::Contribution> = None;

        for contribution in &self.contributions {
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
                Some(c) => P::is_linked_to(c, root),
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
